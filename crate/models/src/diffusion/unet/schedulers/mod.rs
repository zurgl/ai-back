use super::model::UNet2DConditionModel;
use shared::message::emit::Emit;
use shared::message::Message;
use shared::types::MessageType;
use tch::{Device, IndexOp, Kind, Tensor};

pub mod ddim;
pub mod integrate;
pub mod lms_discrete;
use indicatif::ProgressBar;
use tokio::sync::broadcast;

use crate::diffusion::unet::schedulers::ddim::DDIMScheduler;
use crate::diffusion::unet::schedulers::lms_discrete::LMSDiscreteScheduler;

const GUIDANCE_SCALE: f64 = 8.5;

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub enum BetaSchedule {
    Linear,
    ScaledLinear,
    SquaredcosCapV2,
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub enum PredictionType {
    Epsilon,
    VPrediction,
    Sample,
}

pub(crate) fn betas_for_alpha_bar(num_diffusion_timesteps: usize, max_beta: f64) -> Tensor {
    let alpha_bar = |time_step: usize| {
        f64::cos((time_step as f64 + 0.008) / 1.008 * std::f64::consts::FRAC_PI_2).powi(2)
    };
    let mut betas = Vec::with_capacity(num_diffusion_timesteps);
    for i in 0..num_diffusion_timesteps {
        let t1 = i / num_diffusion_timesteps;
        let t2 = (i + 1) / num_diffusion_timesteps;
        betas.push((1.0 - alpha_bar(t2) / alpha_bar(t1)).min(max_beta));
    }

    Tensor::from_slice(&betas)
}

pub fn interp(x: &Tensor, xp: Tensor, yp: Tensor) -> Tensor {
    assert_eq!(xp.size(), yp.size());
    let sz = xp.size1().unwrap();

    // (yp[1:] - yp[:-1]) / (xp[1:] - xp[:-1])
    let m = (yp.i(1..) - yp.i(..sz - 1)) / (xp.i(1..) - xp.i(..sz - 1));

    // yp[:-1] - (m * xp[:-1])
    let b = yp.i(..sz - 1) - (&m * xp.i(..sz - 1));

    // torch.sum(torch.ge(x[:, None], xp[None, :]), 1) - 1
    let indices = x.unsqueeze(-1).ge_tensor(&xp.unsqueeze(0));
    let indices = indices.sum_dim_intlist([1].as_slice(), false, Kind::Int64) - 1;
    // torch.clamp(indices, 0, len(m) - 1)
    let indices = indices.clamp(0, m.size1().unwrap() - 1);

    m.take(&indices) * x + b.take(&indices)
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]

pub struct SchedulerTick {
    tick: usize,
    steps: usize,
}

#[allow(clippy::too_many_arguments)]
pub trait Scheduler: Send {
    fn schedule(
        &mut self,
        init: &Tensor,
        unet: &UNet2DConditionModel,
        text: &Tensor,
        device: Device,
        steps: usize,
        with_bar: bool,
        tx: Option<broadcast::Sender<Message>>,
    ) -> Tensor {
        let mut latents = init.shallow_clone();
        latents *= self.init_noise_sigma();

        let bar = if with_bar {
            Some(ProgressBar::new(steps as u64))
        } else {
            None
        };

        match &tx {
            Some(tx) => {
                let message = SchedulerTick { tick: 0, steps };
                MessageType::SchedulerStep
                    .emit(
                        tx,
                        Default::default(),
                        Some(&serde_json::json!(message).to_string()),
                    )
                    .map_err(|err| format!("{err}"))
                    .unwrap();
            }
            None => (),
        };

        for (index, timestep) in self.timesteps().iter().enumerate() {
            match &bar {
                Some(bar) => bar.inc(1),
                None => (),
            };

            let latent_model_input = Tensor::cat(&[&latents, &latents], 0).to_device(device);
            let latent_model_input = self.scale_model_input(latent_model_input, *timestep);

            let noise_pred = unet.forward(&latent_model_input, *timestep, text);
            let noise_pred = noise_pred.chunk(2, 0);
            let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
            let noise_pred =
                noise_pred_uncond + (noise_pred_text - noise_pred_uncond) * GUIDANCE_SCALE;

            latents = self.step(&noise_pred, *timestep, &latents);

            match &tx {
                Some(tx) => {
                    let message = SchedulerTick {
                        tick: index + 1,
                        steps,
                    };
                    MessageType::SchedulerStep
                        .emit(
                            tx,
                            Default::default(),
                            Some(&serde_json::json!(message).to_string()),
                        )
                        .map_err(|err| format!("{err}"))
                        .unwrap();
                }
                None => (),
            };
        }
        match &bar {
            Some(bar) => bar.finish(),
            None => (),
        };

        latents
    }

    fn step(&mut self, model_output: &Tensor, timestep: f64, sample: &Tensor) -> Tensor;
    fn timesteps(&self) -> Vec<f64>;
    fn scale_model_input(&self, sample: Tensor, timestep: f64) -> Tensor;
    fn init_noise_sigma(&self) -> f64;
}

pub fn select_scheduler(name: &str, steps: usize) -> Box<dyn Scheduler> {
    match name {
        "dlms" => Box::new(LMSDiscreteScheduler::new(steps)),
        "ddims" => Box::new(DDIMScheduler::new(steps)),
        _ => unimplemented!(),
    }
}
