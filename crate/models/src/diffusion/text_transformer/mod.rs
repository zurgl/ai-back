use tch::nn::Module;

use transformer::ClipTextTransformer;

use crate::diffusion::tokenizer::Tokenizer;
use shared::tools;

use self::transformer::LMTSConfig;

pub mod transformer;

#[derive(Debug)]
pub struct Clip {
    pub clip: ClipTextTransformer,
    pub out_device: tch::Device,
}

impl Clip {
    pub fn new(device: Option<tch::Device>) -> Result<Self, &'static str> {
        let out_device = match device {
            None => tch::Device::Cpu,
            Some(device) => device,
        };
        let config = LMTSConfig::default();
        let clip = ClipTextTransformer::new(config, out_device)?;
        Ok(Self { clip, out_device })
    }

    pub fn wait() {
        tools::wait_for_input();
    }

    pub fn clear(&self) {
        self.clip.store.variables().clear()
    }

    pub fn drop_it(&self) {
        for (_path, tensor) in self.clip.store.variables().into_iter() {
            drop(tensor);
        }
    }

    pub fn move_to_cpu(&mut self) {
        self.clip.store.set_device(tch::Device::Cpu)
    }

    pub fn run(&self, prompt: &str, tokenizer: Tokenizer) -> Result<tch::Tensor, &'static str> {
        let Clip {
            out_device, clip, ..
        } = self;

        let tokens = tokenizer.encode_to_tensor(prompt, *out_device)?;
        let text_embeddings = clip.forward(&tokens);
        let uncond_tokens = tokenizer.encode_to_tensor("", *out_device)?;
        let uncond_embeddings = clip.forward(&uncond_tokens);
        let text = tch::Tensor::cat(&[uncond_embeddings, text_embeddings], 0).to(*out_device);

        Ok(text)
    }
}
