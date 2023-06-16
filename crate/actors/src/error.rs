use thiserror::Error;

use shared::message::emit::EmitSource;

#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Supervisor Model Process; task_id not found; for {owner}")]
    SupervisorModelProcessTaskIdNotFound { owner: String },
    #[error("Supervisor Model Process; echo instruction failed; for {owner}")]
    SupervisorModelProcessEchoFailed { owner: String },

    #[error("Supervisor Run Process; command not implemented")]
    SupervisoRunCommandNotImplemented,

    #[error("Supervisor Model Kill; emit message failed; for {owner}")]
    SupervisorModelEmitKillEmitMessageFailed { owner: String },
    #[error("Supervisor Model Kill; task_id not found; for {owner}")]
    SupervisorModelKillEmitTaskIdNotFound { owner: String },

    #[error("Supervisor Task Stats; task_id not found; for {owner}")]
    SupervisorTaskStatsTaskIdNotFound { owner: String },
    #[error("Supervisor Task Stats; emit message failed; for {owner}")]
    SupervisorEmitTaskStatsEmitMessageFailed { owner: String },

    #[error("Supervisor Model Spawn; insert task_id failed; for {owner}")]
    SupervisorModelSpawnInsertTaskIdFailed { owner: String },
    #[error("Supervisor Model Spawn; cannot spawn model; for {owner}")]
    SupervisorModelSpawnModelSpawnFailed { owner: String },

    #[error("Worker Run; parse json input failed; for {owner} and {source}")]
    WorkerRunParseJsonInputFailed { owner: String, source: EmitSource },
    #[error("Worker Run; model Loaded Failed; for {owner} and {source}")]
    WorkerRunModelLoadedFailed { owner: String, source: EmitSource },
    #[error("Worker Run; emit message model Started Failed; for {owner} and {source}")]
    WorkerRunEmitModelStartedFailed { owner: String, source: EmitSource },
    #[error("Worker Run; model new Failed; for {owner} and {source}")]
    WorkerRunModelInitializeFailed { owner: String, source: EmitSource },
    #[error("Worker Run; emit message model prediction Failed; for {owner}")]
    WorkerRunEmitModelPredictionFailed { owner: String },
    #[error("Worker Run; emit message model error Failed; for {owner} and {source}")]
    WorkerRunEmitModelErrorFailed { owner: String, source: EmitSource },
}
