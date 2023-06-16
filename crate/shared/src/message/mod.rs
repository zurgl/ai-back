pub mod emit;

use crate::types::CommandType;
use crate::types::MessageType;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Message {
    Health(HealthT),
    CommandSucess(CommandSucessT),
    CommandFailed(CommandFailedT),
    ModelPaused(ModelPausedT),
    ModelResumed(ModelResumedT),
    ModelKilled(ModelKilledT),
    ModelStarted(ModelStartedT),
    ModelLoaded(ModelLoadedT),
    ModelPrediction(ModelPredictionT),
    ModelError(ModelErrorT),
    SchedulerStep(SchedulerStepT),
    LlamaTokenGen(LlamaTokenGenT),
}

impl Message {
    pub fn is_health(&self) -> bool {
        matches!(self, Message::Health(_))
    }

    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Health(_) => MessageType::Health,
            Message::CommandSucess(_) => MessageType::CommandSucess,
            Message::CommandFailed(_) => MessageType::CommandFailed,
            Message::ModelPaused(_) => MessageType::ModelPaused,
            Message::ModelResumed(_) => MessageType::ModelResumed,
            Message::ModelKilled(_) => MessageType::ModelKilled,
            Message::ModelStarted(_) => MessageType::ModelStarted,
            Message::ModelLoaded(_) => MessageType::ModelLoaded,
            Message::ModelPrediction(_) => MessageType::ModelPrediction,
            Message::ModelError(_) => MessageType::ModelError,
            Message::SchedulerStep(_) => MessageType::SchedulerStep,
            Message::LlamaTokenGen(_) => MessageType::LlamaTokenGen,
        }
    }

    pub fn task_id(&self) -> Option<String> {
        match self {
            Message::ModelPaused(data) => Some(data.task_id.to_string()),
            Message::ModelResumed(data) => Some(data.task_id.to_string()),
            Message::ModelPrediction(data) => Some(data.task_id.to_string()),
            Message::ModelLoaded(data) => Some(data.task_id.to_string()),
            Message::ModelKilled(data) => Some(data.task_id.to_string()),
            Message::ModelStarted(data) => Some(data.task_id.to_string()),
            Message::Health(data) => Some(data.task_id.to_string()),
            Message::ModelError(data) => Some(data.task_id.to_string()),
            _ => None,
        }
    }
}

pub trait Owner {
    fn owner(&self) -> &str;
}

impl Owner for Message {
    fn owner(&self) -> &str {
        match self {
            Message::Health(data) => &data.owner,
            Message::CommandSucess(data) => &data.owner,
            Message::CommandFailed(data) => &data.owner,
            Message::ModelPaused(data) => &data.owner,
            Message::ModelResumed(data) => &data.owner,
            Message::ModelKilled(data) => &data.owner,
            Message::ModelStarted(data) => &data.owner,
            Message::ModelLoaded(data) => &data.owner,
            Message::ModelPrediction(data) => &data.owner,
            Message::ModelError(data) => &data.owner,
            Message::SchedulerStep(data) => &data.owner,
            Message::LlamaTokenGen(data) => &data.owner,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HealthT {
    pub owner: String,
    pub message_type: MessageType,
    pub timestamp: u128,
    pub task_id: String,
    pub value: String,
}

use crate::types::ModelType;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommandSucessT {
    pub owner: String,
    pub message_type: MessageType,
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub timestamp: u128,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CommandFailedT {
    pub owner: String,
    pub timestamp: u128,
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub message_type: MessageType,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SchedulerStepT {
    pub owner: String,
    pub timestamp: u128,
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub message_type: MessageType,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LlamaTokenGenT {
    pub owner: String,
    pub timestamp: u128,
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub message_type: MessageType,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModelPausedT {
    pub owner: String,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub model_type: ModelType,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModelStartedT {
    pub owner: String,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub model_type: ModelType,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModelKilledT {
    pub owner: String,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub model_type: ModelType,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModelResumedT {
    pub owner: String,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub model_type: ModelType,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModelPredictionT {
    pub owner: String,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub model_type: ModelType,
    pub value: String,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModelLoadedT {
    pub owner: String,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub model_type: ModelType,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ModelErrorT {
    pub owner: String,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub model_type: ModelType,
    pub error: String,
    pub task_id: String,
}
