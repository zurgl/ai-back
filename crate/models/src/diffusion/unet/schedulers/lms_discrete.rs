use super::integrate::integrate;
use super::{interp, BetaSchedule, PredictionType, Scheduler};
use tch::{kind, Kind, Tensor};

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct LMSDiscreteSchedulerConfig {
    pub beta_start: f64,
    pub beta_end: f64,
    pub beta_schedule: BetaSchedule,
    pub train_timesteps: usize,
    pub order: usize,
    pub prediction_type: PredictionType,
}

impl Default for LMSDiscreteSchedulerConfig {
    fn default() -> Self {
        Self {
            beta_start: 0.00085,
            beta_end: 0.012,
            beta_schedule: BetaSchedule::ScaledLinear,
            train_timesteps: 1000,
            order: 4,
            prediction_type: PredictionType::VPrediction,
        }
    }
}

pub struct LMSDiscreteScheduler {
    timesteps: Vec<f64>,
    sigmas: Vec<f64>,
    init_noise_sigma: f64,
    derivatives: Vec<Tensor>,
    pub config: LMSDiscreteSchedulerConfig,
}

impl LMSDiscreteScheduler {
    pub fn new(inference_steps: usize) -> Self {
        let config = LMSDiscreteSchedulerConfig::default();
        let betas = match config.beta_schedule {
            BetaSchedule::ScaledLinear => Tensor::linspace(
                config.beta_start.sqrt(),
                config.beta_end.sqrt(),
                config.train_timesteps as i64,
                kind::FLOAT_CPU,
            )
            .square(),
            BetaSchedule::Linear => Tensor::linspace(
                config.beta_start,
                config.beta_end,
                config.train_timesteps as i64,
                kind::FLOAT_CPU,
            ),
            _ => unimplemented!(
                "LMSDiscreteScheduler only implements linear and scaled_linear betas."
            ),
        };

        let alphas: Tensor = 1. - betas;
        let alphas_cumprod = alphas.cumprod(0, Kind::Double);

        let timesteps = Tensor::linspace(
            (config.train_timesteps - 1) as f64,
            0.,
            inference_steps as i64,
            kind::FLOAT_CPU,
        );

        let sigmas = ((1. - &alphas_cumprod) as Tensor / &alphas_cumprod).sqrt();
        let sigmas = interp(
            &timesteps, // x-coordinates at which to evaluate the interpolated values
            Tensor::range(0, sigmas.size1().unwrap() - 1, kind::FLOAT_CPU),
            sigmas,
        );
        let sigmas = Tensor::concat(&[sigmas, Tensor::from_slice(&[0.0])], 0);

        // standard deviation of the initial noise distribution
        let init_noise_sigma: f64 = f64::try_from(sigmas.max()).unwrap();

        Self {
            timesteps: Vec::<f64>::try_from(&timesteps).unwrap(),
            sigmas: Vec::<f64>::try_from(&sigmas).unwrap(),
            init_noise_sigma,
            derivatives: vec![],
            config,
        }
    }

    fn get_lms_coefficient(&mut self, order: usize, t: usize, current_order: usize) -> f64 {
        let lms_derivative = |tau| -> f64 {
            let mut prod = 1.0;
            for k in 0..order {
                if current_order == k {
                    continue;
                }
                prod *= (tau - self.sigmas[t - k])
                    / (self.sigmas[t - current_order] - self.sigmas[t - k]);
            }
            prod
        };

        // Integrate `lms_derivative` over two consecutive timesteps.
        // Absolute tolerances and limit are taken from
        // the defaults of `scipy.integrate.quad`
        let integration_out =
            integrate(lms_derivative, self.sigmas[t], self.sigmas[t + 1], 1.49e-8);
        // integrated coeff
        integration_out.integral
    }
}

impl Scheduler for LMSDiscreteScheduler {
    fn step(&mut self, model_output: &Tensor, timestep: f64, sample: &Tensor) -> Tensor {
        let step_index = self.timesteps.iter().position(|&t| t == timestep).unwrap();
        let sigma = self.sigmas[step_index];

        // 1. compute predicted original sample (x_0) from sigma-scaled predicted noise
        let pred_original_sample = match self.config.prediction_type {
            PredictionType::Epsilon => sample - sigma * model_output,
            PredictionType::VPrediction => {
                model_output * (-sigma / (sigma.powi(2) + 1.).sqrt())
                    + (sample / (sigma.powi(2) + 1.))
            }
            _ => unimplemented!("Prediction type must be one of `epsilon` or `v_prediction`"),
        };

        // 2. Convert to an ODE derivative
        let derivative = (sample - pred_original_sample) / sigma;
        self.derivatives.push(derivative);
        if self.derivatives.len() > self.config.order {
            // remove the first element
            self.derivatives.drain(0..1);
        }

        // 3. compute linear multistep coefficients
        let order = self.config.order.min(step_index + 1);
        let lms_coeffs: Vec<_> = (0..order)
            .map(|o| self.get_lms_coefficient(order, step_index, o))
            .collect();

        // 4. compute previous sample based on the derivatives path
        let deriv_sum: Tensor = lms_coeffs
            .iter()
            .zip(self.derivatives.iter().rev())
            .map(|(coeff, derivative)| *coeff * derivative)
            .sum();

        sample + deriv_sum
    }

    fn init_noise_sigma(&self) -> f64 {
        self.init_noise_sigma
    }

    fn timesteps(&self) -> Vec<f64> {
        // Vec<f64> {
        self.timesteps.clone()
    }

    fn scale_model_input(&self, sample: Tensor, timestep: f64) -> Tensor {
        let step_index = self.timesteps.iter().position(|&t| t == timestep).unwrap();
        let sigma = self.sigmas[step_index];
        sample / (sigma.powi(2) + 1.).sqrt()
    }
}
