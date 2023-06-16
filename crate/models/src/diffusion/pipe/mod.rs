use shared::message::Message;
use tch::{Device, Tensor};
use tokio::sync::broadcast;

use super::unet::model::UNet2DConditionModel;
use super::unet::schedulers::{select_scheduler, Scheduler};
use super::vae::model::AutoEncoderKL;

#[derive(Clone, Debug)]
pub struct PipeConfig {
    pub prompt: Option<String>,
    pub device: Device,
    pub scheduler: String,
    pub steps: usize,
    pub n_frame: u32,
    pub height: i64,
    pub width: i64,
    pub seed: i64,
    pub inference: Option<i64>,
}

impl Default for PipeConfig {
    fn default() -> Self {
        Self {
            prompt: None,
            inference: None,
            device: tch::Device::cuda_if_available(),
            scheduler: "dlms".to_string(),
            steps: 30,
            n_frame: 0,
            height: 768,
            width: 768,
            seed: 42,
        }
    }
}

impl PipeConfig {
    pub fn with_device(&self, device: tch::Device) -> Self {
        Self {
            device,
            ..self.clone()
        }
    }

    pub fn with_frame(&self, n_frame: u32) -> Self {
        Self {
            n_frame,
            ..self.clone()
        }
    }

    pub fn inc_n_frame(&mut self) {
        self.n_frame += 1
    }
}

pub struct Pipe {
    unet: UNet2DConditionModel,
    vae: AutoEncoderKL,
    scheduler: Box<dyn Scheduler>,
    pub config: PipeConfig,
}

impl Pipe {
    pub fn new(config: PipeConfig) -> anyhow::Result<Self> {
        let PipeConfig {
            device,
            scheduler,
            steps,
            ..
        } = config.clone();

        let unet = UNet2DConditionModel::new(Default::default(), device);
        let vae = AutoEncoderKL::new(Default::default(), device);
        let scheduler = select_scheduler(&scheduler, steps);

        Ok(Self {
            unet,
            vae,
            scheduler,
            config,
        })
    }

    pub fn diffuse(
        &mut self,
        init: &Tensor,
        text: &Tensor,
        with_bar: bool,
        tx: Option<broadcast::Sender<Message>>,
    ) -> anyhow::Result<Vec<String>> {
        let Pipe {
            unet, vae, config, ..
        } = self;

        let mut images = vec![];

        let latent = {
            self.scheduler
                .schedule(init, unet, text, config.device, config.steps, with_bar, tx)
        };

        let image = {
            let latent = latent.to(config.device);
            let image = vae.decode(&(&latent / 0.18215));
            let image = (image / 2 + 0.5).clamp(0., 1.).to_device(Device::Cpu);
            (image * 255.).to_kind(tch::Kind::Uint8)
        };

        tch::vision::image::save(&image, format!("images/frame{:04}.png", config.n_frame))?;
        images.push(format!("images/frame{:04}.png", config.n_frame));
        config.inc_n_frame();

        Ok(images)
    }
}
