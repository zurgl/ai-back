// cargo test -p models --test '*' -- --nocapture
use models::diffusion::text_transformer::transformer::ClipTextTransformer;
use models::diffusion::tokenizer::{Tokenizer, TokenizerConfig};
use models::diffusion::unet::model::UNet2DConditionModel;
use models::diffusion::vae::model::AutoEncoderKL;

fn get_config() -> TokenizerConfig {
    TokenizerConfig {
        max_position_embeddings: 77,
        pad_with: Some("!".to_string()),
    }
}

#[test]
fn tokenizer_config() {
    let config = TokenizerConfig::default();
    assert_eq!(config, get_config());
}

#[test]
fn tokenizer_loading() {
    let tokenizer = Tokenizer::create(Default::default());
    assert!(tokenizer.is_ok());
}

#[test]
fn lmts_loading() {
    let device = tch::Device::cuda_if_available();
    let lmts = ClipTextTransformer::new(Default::default(), device);
    assert!(lmts.is_ok());
}

#[test]
#[ignore]
fn unet_loading() {
    let device = tch::Device::Cpu; //cuda_if_available();
    let _unet = UNet2DConditionModel::new(Default::default(), device);
}

#[test]
fn vae_loading() {
    let device = tch::Device::cuda_if_available();
    let _vae = AutoEncoderKL::new(Default::default(), device);
}
