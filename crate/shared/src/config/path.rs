use std::path::PathBuf;

pub fn workspace() -> PathBuf {
    let current_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    current_path.ancestors().nth(2).unwrap().to_path_buf()
}

pub fn data() -> PathBuf {
    workspace().join(crate::constants::AI_DATA_PATH)
}

pub fn config() -> PathBuf {
    workspace().join("configuration")
}

pub mod model {
    pub const DISTILBERT_SST2: &str = "distilbert_sst2";
    pub const DISTILBART_CNN_6_6: &str = "distilbart_cnn_6_6";
    pub const STABLE_DIFFUSION_2_1: &str = "stable_diffusion_2_1";
    pub const M2M100_418M: &str = "m2m100_418M";
    pub const LLAMA_7B: &str = "llama_7B";
}
