use shared::{
    message::{emit::Emit, Message},
    types::MessageType,
};
use tokio::sync::broadcast;

use crate::llama::{
    model::{precompute_freqs_cis, CONTEXT_SIZE},
    tokenizer::Tokenizer,
};

use self::model::{Config, LlamaModel};

pub mod model;
pub mod tokenizer;

pub struct Llama {
    model: LlamaModel,
    tokenizer: Tokenizer,
    config: Config,
    device: tch::Device,
}

impl Default for Llama {
    fn default() -> Self {
        let tokenizer = Tokenizer::from_file(".ai-data/llama_7B/llama-tokenizer.json")
            .expect("Cannot build the Tokenizer");

        let device = tch::Device::cuda_if_available();
        let mut vs = tch::nn::VarStore::new(device);
        let config = Config::config_7b();

        let llama = LlamaModel::new(vs.root(), &config);

        vs.set_kind(tch::Kind::Half);
        {
            let file = std::fs::File::open(".ai-data/llama_7B/llama.safetensors")
                .expect("Cannot load weight");
            let content = unsafe {
                memmap2::MmapOptions::new()
                    .map(&file)
                    .expect("Cannot mmap weight")
            };
            let safetensors = safetensors::SafeTensors::deserialize(&content)
                .expect("Cannot deserialze the weight");

            let mut variables = vs.variables_.lock().unwrap();
            for (name, var) in variables.named_variables.iter_mut() {
                let src_tensor_view = safetensors.tensor(name).expect("Loading error");
                let src_tensor: tch::Tensor = src_tensor_view.try_into().expect("Loading error");
                var.f_copy_(&src_tensor).expect("Loading error");
            }
        }
        vs.set_kind(tch::Kind::Float);

        Self {
            model: llama,
            tokenizer,
            config,
            device,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LlamaTick {
    tick: usize,
    steps: usize,
}

impl Llama {
    pub fn prediction(
        &self,
        prompt: &str,
        sample_len: usize,
        temperature: f64,
        tx: Option<broadcast::Sender<Message>>,
    ) -> String {
        let _no_grad = tch::no_grad_guard();

        let mut tokens = self
            .tokenizer
            .encode(prompt)
            .expect("Cannot encode the prompt");
        let mut new_tokens = vec![];
        let freqs_cis = precompute_freqs_cis(&self.config).to_device(self.device);

        for index in 0..sample_len {
            let ctxt: Vec<_> = tokens[tokens.len().saturating_sub(CONTEXT_SIZE)..]
                .iter()
                .map(|c| *c as i64)
                .collect();
            let ctxt = tch::Tensor::from_slice(&ctxt).reshape([1, -1]);
            let logits = self.model.forward(&ctxt, &freqs_cis);
            let ctxt = (logits / temperature).softmax(-1, tch::Kind::Float);

            let logits = self.model.forward(&ctxt, &freqs_cis);
            let sampled_y = logits.get(0).get(0).multinomial(1, true);
            let next_token = i64::try_from(&sampled_y).expect("Prediction error") as usize;
            tokens.push(next_token);
            new_tokens.push(next_token);
            match &tx {
                Some(tx) => {
                    let message = LlamaTick {
                        tick: index + 1,
                        steps: sample_len,
                    };
                    MessageType::LlamaTokenGen
                        .emit(
                            tx,
                            Default::default(),
                            Some(&serde_json::json!(message).to_string()),
                        )
                        .map_err(|err| format!("{err}"))
                        .unwrap();
                }
                None => println!(
                    "{} token: {} '{}'",
                    index + 1,
                    next_token,
                    self.tokenizer.decode(&[next_token])
                ),
            };
        }
        self.tokenizer.decode(&new_tokens)
    }

    pub fn try_prediction(
        &self,
        prompt: &str,
        sample_len: usize,
        temperature: f64,
        tx: Option<broadcast::Sender<Message>>,
    ) -> Result<(), &'static str> {
        let _no_grad = tch::no_grad_guard();

        let mut tokens = self
            .tokenizer
            .encode(prompt)
            .expect("Cannot encode the prompt");
        let mut new_tokens = vec![];
        let freqs_cis = precompute_freqs_cis(&self.config).to_device(self.device);

        for index in 0..sample_len {
            let ctxt: Vec<_> = tokens[tokens.len().saturating_sub(CONTEXT_SIZE)..]
                .iter()
                .map(|c| *c as i64)
                .collect();
            let ctxt = tch::Tensor::from_slice(&ctxt).reshape([1, -1]);
            let logits = self.model.forward(&ctxt, &freqs_cis);
            let ctxt = (logits / temperature).softmax(-1, tch::Kind::Float);

            let logits = self.model.forward(&ctxt, &freqs_cis);
            let sampled_y = logits.get(0).get(0).multinomial(1, true);
            let next_token = i64::try_from(&sampled_y).expect("Prediction error") as usize;
            tokens.push(next_token);
            new_tokens.push(next_token);
            match &tx {
                Some(tx) => {
                    let message = LlamaTick {
                        tick: index + 1,
                        steps: sample_len,
                    };
                    MessageType::LlamaTokenGen
                        .emit(
                            tx,
                            Default::default(),
                            Some(&serde_json::json!(message).to_string()),
                        )
                        .map_err(|err| format!("{err}"))
                        .unwrap();
                }
                None => println!(
                    "{} token: {} '{}'",
                    index + 1,
                    next_token,
                    self.tokenizer.decode(&[next_token])
                ),
            };
        }
        let message = self.tokenizer.decode(&new_tokens);
        println!("{message:?}");
        Ok(())
    }
}
