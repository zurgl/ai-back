use super::Instruction;
use crate::types::CommandType;
use crate::types::ModelType;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Process {
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub task_id: Option<String>,
    pub json_input: String,
    pub timestamp: u128,
    pub owner: String,
}

impl Instruction for Process {
    fn command_type(&self) -> CommandType {
        self.command_type
    }

    fn model_type(&self) -> ModelType {
        self.model_type
    }

    fn task_id(&self) -> Option<String> {
        self.task_id.clone()
    }

    fn json_input(&self) -> Option<String> {
        Some(self.json_input.clone())
    }

    fn timestamp(&self) -> u128 {
        self.timestamp
    }

    fn owner(&self) -> String {
        self.owner.clone()
    }
}
