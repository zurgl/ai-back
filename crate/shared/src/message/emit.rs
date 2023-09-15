use std::error::Error;
use tokio::{
    sync::broadcast::{self, error::SendError},
    task::Id,
};

use crate::{
    command::instruction::Instruction,
    constants,
    message::{
        CommandFailedT, CommandSucessT, HealthT, Message, MessageType, ModelErrorT, ModelKilledT,
        ModelLoadedT, ModelPausedT, ModelPredictionT, ModelResumedT, ModelStartedT,
    },
    tools::root,
    types::CommandType,
};

use super::{LlamaTokenGenT, SchedulerStepT};

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EmitSource {
    command_type: Option<CommandType>,
    owner: String,
    task_id: Option<String>,
    model_type: Option<crate::types::model::ModelType>,
}

impl Default for EmitSource {
    fn default() -> Self {
        Self {
            command_type: None,
            owner: root(),
            task_id: None,
            model_type: None,
        }
    }
}

impl std::fmt::Display for EmitSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let EmitSource {
            command_type,
            owner,
            task_id,
            model_type,
        } = self;
        write!(f, "EmitSource {{ command_type: {command_type:?}, owner: {owner}, task_id: {task_id:?}, model_type: {model_type:?} }}")
    }
}

impl Error for EmitSource {}

impl EmitSource {
    pub fn set_task_id(&self, task_id: Id) -> EmitSource {
        let EmitSource {
            command_type,
            owner,
            model_type,
            ..
        } = self;
        EmitSource {
            command_type: *command_type,
            owner: owner.clone(),
            task_id: Some(task_id.to_string()),
            model_type: *model_type,
        }
    }

    pub fn set_owner(&self, owner: &str) -> EmitSource {
        let EmitSource {
            command_type,
            task_id,
            model_type,
            ..
        } = self;
        EmitSource {
            command_type: *command_type,
            owner: owner.to_owned(),
            task_id: task_id.as_deref().map(|value| value.to_owned()),
            model_type: *model_type,
        }
    }

    pub fn health(task_id: Id) -> Self {
        Self {
            command_type: None,
            owner: constants::role::ROOT.to_owned(),
            task_id: Some(task_id.to_string()),
            model_type: None,
        }
    }
}

impl From<Box<dyn Instruction>> for EmitSource {
    fn from(payload: Box<dyn Instruction>) -> Self {
        Self {
            command_type: Some(payload.command_type()),
            owner: payload.owner(),
            task_id: payload.task_id(),
            model_type: Some(payload.model_type()),
        }
    }
}

pub trait Emit {
    fn emit(
        &self,
        tx: &broadcast::Sender<Message>,
        source: EmitSource,
        value: Option<&str>,
    ) -> Result<usize, SendError<Message>>;
}

impl Emit for MessageType {
    fn emit(
        &self,
        tx: &broadcast::Sender<Message>,
        source: EmitSource,
        value: Option<&str>,
    ) -> Result<usize, SendError<Message>> {
        let message_type = self.clone();
        let message = match self {
            MessageType::Health => Message::Health(HealthT {
                timestamp: crate::tools::time(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                message_type,
                value: value.unwrap().to_owned(),
            }),
            MessageType::SchedulerStep => Message::SchedulerStep(SchedulerStepT {
                owner: source.owner,
                timestamp: crate::tools::time(),
                command_type: CommandType::Process,
                model_type: crate::types::ModelType::Diffusion,
                message_type,
                value: value.unwrap().to_owned(),
            }),
            MessageType::LlamaTokenGen => Message::LlamaTokenGen(LlamaTokenGenT {
                owner: source.owner,
                timestamp: crate::tools::time(),
                command_type: CommandType::Process,
                model_type: crate::types::ModelType::Diffusion,
                message_type,
                value: value.unwrap().to_owned(),
            }),
            MessageType::CommandFailed => Message::CommandFailed(CommandFailedT {
                timestamp: crate::tools::time(),
                command_type: source.command_type.unwrap(),
                owner: source.owner,
                error: value.unwrap().to_owned(),
                model_type: source.model_type.unwrap(),
                message_type,
            }),
            MessageType::CommandSucess => Message::CommandSucess(CommandSucessT {
                timestamp: crate::tools::time(),
                command_type: source.command_type.unwrap(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                message_type,
            }),
            MessageType::ModelKilled => Message::ModelKilled(ModelKilledT {
                timestamp: crate::tools::time(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                message_type,
            }),
            MessageType::ModelPaused => Message::ModelPaused(ModelPausedT {
                timestamp: crate::tools::time(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                message_type,
            }),
            MessageType::ModelResumed => Message::ModelResumed(ModelResumedT {
                timestamp: crate::tools::time(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                message_type,
            }),
            MessageType::ModelStarted => Message::ModelStarted(ModelStartedT {
                timestamp: crate::tools::time(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                message_type,
            }),
            MessageType::ModelLoaded => Message::ModelLoaded(ModelLoadedT {
                timestamp: crate::tools::time(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                message_type,
            }),
            MessageType::ModelPrediction => Message::ModelPrediction(ModelPredictionT {
                timestamp: crate::tools::time(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                value: value.unwrap().to_owned(),
                message_type,
            }),
            MessageType::ModelError => Message::ModelError(ModelErrorT {
                timestamp: crate::tools::time(),
                model_type: source.model_type.unwrap(),
                task_id: source.task_id.unwrap(),
                owner: source.owner,
                error: value.unwrap().to_owned(),
                message_type,
            }),
        };
        println!("{message:?}");

        tx.send(message)
    }
}
