use super::Instruction;
use crate::types::CommandType;
use crate::types::ModelType;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Spawn {
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub timestamp: u128,
    pub owner: String,
}

impl Spawn {
    pub fn new(owner: &str, model_type: ModelType) -> Self {
        Self {
            command_type: CommandType::Spawn,
            model_type,
            timestamp: crate::tools::time(),
            owner: owner.to_string(),
        }
    }
}

impl Instruction for Spawn {
    fn command_type(&self) -> CommandType {
        self.command_type
    }

    fn model_type(&self) -> ModelType {
        self.model_type
    }

    fn task_id(&self) -> Option<String> {
        None
    }

    fn json_input(&self) -> Option<String> {
        None
    }

    fn timestamp(&self) -> u128 {
        self.timestamp
    }

    fn owner(&self) -> String {
        self.owner.clone()
    }
}
