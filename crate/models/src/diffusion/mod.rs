pub mod configuration;
pub mod pipe;
pub mod text_transformer;
pub mod tokenizer;
pub mod unet;
pub mod utils;
pub mod vae;

use shared::message::Message;
use tokio::sync::broadcast;

use crate::diffusion::{pipe::Pipe, text_transformer::Clip, tokenizer::Tokenizer};

pub struct Diffusion {
    pipe: Pipe,
    tokenizer: Tokenizer,
    device: tch::Device,
}

// fn to_parameters(device: tch::Device) -> Parameters {
//     Parameters {
//         prompt: None,
//         steps: 30,
//         scheduler: "dlms".to_owned(),
//         n_frame: 0,
//         device,
//     }
// }

impl Default for Diffusion {
    fn default() -> Self {
        let device = tch::Device::cuda_if_available();
        let tokenizer = Tokenizer::create(Default::default()).expect("cannot create tokenizer");
        let pipe = Pipe::new(Default::default()).expect("cannot create pipe");

        Self {
            pipe,
            tokenizer,
            device,
        }
    }
}

impl Diffusion {
    pub fn prediction(
        &mut self,
        prompt: &str,
        seed: i64,
        height: i64,
        width: i64,
        tx: Option<broadcast::Sender<Message>>,
    ) -> Vec<String> {
        tch::manual_seed(seed);
        let no_grad_guard = tch::no_grad_guard();
        let tensor = tch::Tensor::randn(
            [1, 4, height / 8, width / 8],
            (tch::Kind::Float, self.device),
        );
        let text = Clip::new(Some(self.device))
            .and_then(|clip| clip.run(prompt, self.tokenizer.clone()))
            .expect("cannot encode prompt");

        let prediction = self
            .pipe
            .diffuse(&tensor, &text, true, tx)
            .expect("cannot run pipe");
        drop(no_grad_guard);

        prediction
    }
}
