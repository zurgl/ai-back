#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum MessageType {
    Health,
    CommandSucess,
    CommandFailed,
    ModelPaused,
    ModelResumed,
    ModelKilled,
    ModelStarted,
    ModelLoaded,
    ModelPrediction,
    ModelError,
    SchedulerStep,
    LlamaTokenGen,
}
