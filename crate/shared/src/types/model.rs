#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ModelType {
    Sentiment,
    Summarize,
    Translation,
    Diffusion,
    Llama,
}
