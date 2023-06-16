use super::Instruction;
use crate::types::CommandType;
use crate::types::ModelType;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Kill {
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub task_id: String,
    pub timestamp: u128,
    pub owner: String,
}

impl Kill {
    pub fn new(owner: &str, model_type: ModelType, task_id: &str) -> Self {
        Self {
            command_type: CommandType::Kill,
            model_type,
            task_id: task_id.to_string(),
            timestamp: crate::tools::time(),
            owner: owner.to_string(),
        }
    }
}

impl Instruction for Kill {
    fn command_type(&self) -> CommandType {
        self.command_type
    }

    fn model_type(&self) -> ModelType {
        self.model_type
    }

    fn task_id(&self) -> Option<String> {
        Some(self.task_id.clone())
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
