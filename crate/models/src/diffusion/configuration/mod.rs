#[derive(Clone, Copy)]
pub enum Engine {
    Tokenizer,
    LMTS,
    UnetScheduler,
    VAEDecoder,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Engine::Tokenizer => write!(f, "tokenizer"),
            Engine::LMTS => write!(f, "lmts"),
            Engine::UnetScheduler => write!(f, "unet"),
            Engine::VAEDecoder => write!(f, "vae"),
        }
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Engine::Tokenizer => write!(f, "tokenizer"),
            Engine::LMTS => write!(f, "lmts"),
            Engine::UnetScheduler => write!(f, "unet"),
            Engine::VAEDecoder => write!(f, "vae"),
        }
    }
}

pub fn get_config<'a, T: serde::Deserialize<'a>>(
    engine: Engine,
) -> Result<Box<T>, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::from(path::config(engine)))
        .build()?;

    settings.try_deserialize::<Box<T>>()
}

pub mod path {
    use std::path::PathBuf;

    use super::Engine;

    pub fn weights(engine: Engine) -> PathBuf {
        shared::config::path::data()
            .join("stable_diffusion_2_1")
            .join(format!("{engine}.safetensors"))
    }

    pub fn vocab() -> PathBuf {
        shared::config::path::data()
            .join("stable_diffusion_2_1")
            .join("vocab.txt")
    }

    pub fn config(engine: Engine) -> PathBuf {
        shared::config::path::workspace()
            .join("configuration")
            .join("diffusion")
            .join(format!("{engine}.ron"))
    }
}
