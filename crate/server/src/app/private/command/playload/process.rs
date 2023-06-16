use shared::types::{CommandType, ModelType};

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Process {
    pub command_type: CommandType,
    pub model_type: ModelType,
    pub json_input: String,
    pub task_id: Option<String>,
}

impl Process {
    pub fn new(model_type: ModelType, task_id: &str, json_input: &str) -> Self {
        Self {
            command_type: CommandType::Process,
            model_type,
            json_input: json_input.to_owned(),
            task_id: Some(task_id.to_owned()),
        }
    }
}

impl super::Playload for Process {
    fn command_type(&self) -> CommandType {
        self.command_type
    }

    fn model_type(&self) -> ModelType {
        self.model_type
    }

    fn task_id(&self) -> Option<String> {
        self.task_id.to_owned()
    }

    fn json_input(&self) -> Option<String> {
        Some(self.json_input.to_owned())
    }
}
