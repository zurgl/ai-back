use super::{betas_for_alpha_bar, BetaSchedule, PredictionType, Scheduler};
use tch::{kind, Kind, Tensor};

#[derive(Debug, Clone, Copy)]
pub struct DDIMSchedulerConfig {
    pub beta_start: f64,
    pub beta_end: f64,
    pub beta_schedule: BetaSchedule,
    pub eta: f64,
    pub steps_offset: usize,
    pub prediction_type: PredictionType,
    pub train_timesteps: usize,
}

impl Default for DDIMSchedulerConfig {
    fn default() -> Self {
        Self {
            beta_start: 0.00085f64,
            beta_end: 0.012f64,
            beta_schedule: BetaSchedule::ScaledLinear,
            eta: 0.,
            steps_offset: 1,
            prediction_type: PredictionType::VPrediction,
            train_timesteps: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DDIMScheduler {
    timesteps: Vec<f64>,
    alphas_cumprod: Vec<f64>,
    step_ratio: usize,
    init_noise_sigma: f64,
    pub config: DDIMSchedulerConfig,
}

impl Scheduler for DDIMScheduler {
    fn step(&mut self, model_output: &Tensor, timestep: f64, sample: &Tensor) -> Tensor {
        let timestep = timestep as usize;
        let timestep = if timestep >= self.alphas_cumprod.len() {
            timestep - 1
        } else {
            timestep
        };
        let prev_timestep = if timestep > self.step_ratio {
            timestep - self.step_ratio
        } else {
            0
        };

        let alpha_prod_t = self.alphas_cumprod[timestep];
        let alpha_prod_t_prev = self.alphas_cumprod[prev_timestep];
        let beta_prod_t = 1. - alpha_prod_t;
        let beta_prod_t_prev = 1. - alpha_prod_t_prev;

        let (pred_original_sample, model_output) = match self.config.prediction_type {
            PredictionType::Epsilon => {
                let pred_original_sample =
                    (sample - beta_prod_t.sqrt() * model_output) / alpha_prod_t.sqrt();
                (pred_original_sample, model_output.shallow_clone())
            }
            PredictionType::VPrediction => {
                let pred_original_sample =
                    alpha_prod_t.sqrt() * sample - beta_prod_t.sqrt() * model_output;
                let model_output = alpha_prod_t.sqrt() * model_output + beta_prod_t.sqrt() * sample;
                (pred_original_sample, model_output)
            }
            PredictionType::Sample => {
                let pred_original_sample = model_output.shallow_clone();
                (pred_original_sample, model_output.shallow_clone())
            }
        };

        let variance = (beta_prod_t_prev / beta_prod_t) * (1. - alpha_prod_t / alpha_prod_t_prev);
        let std_dev_t = self.config.eta * variance.sqrt();

        let pred_sample_direction =
            (1. - alpha_prod_t_prev - std_dev_t * std_dev_t).sqrt() * model_output;
        let prev_sample = alpha_prod_t_prev.sqrt() * pred_original_sample + pred_sample_direction;
        if self.config.eta > 0. {
            &prev_sample + Tensor::randn_like(&prev_sample) * std_dev_t
        } else {
            prev_sample
        }
    }

    fn timesteps(&self) -> Vec<f64> {
        self.timesteps.clone()
    }

    fn scale_model_input(&self, sample: Tensor, _timestep: f64) -> Tensor {
        sample
    }

    fn init_noise_sigma(&self) -> f64 {
        self.init_noise_sigma
    }
}

impl DDIMScheduler {
    pub fn new(inference_steps: usize) -> Self {
        let config = DDIMSchedulerConfig::default();
        let step_ratio = config.train_timesteps / inference_steps;
        let timesteps: Vec<f64> = (0..(inference_steps))
            .map(|s| (s * step_ratio + config.steps_offset) as f64)
            .rev()
            .collect();
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
            BetaSchedule::SquaredcosCapV2 => betas_for_alpha_bar(config.train_timesteps, 0.999),
        };
        let alphas: Tensor = 1.0 - betas;
        let alphas_cumprod = Vec::<f64>::try_from(alphas.cumprod(0, Kind::Double)).unwrap();
        Self {
            alphas_cumprod,
            timesteps,
            step_ratio,
            init_noise_sigma: 1.,
            config,
        }
    }
}
