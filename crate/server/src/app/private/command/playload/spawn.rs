use shared::types::{CommandType, ModelType};

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Spawn {
    pub command_type: CommandType,
    pub model_type: ModelType,
}

impl Spawn {
    pub fn new(model_type: ModelType) -> Self {
        Self {
            command_type: CommandType::Spawn,
            model_type,
        }
    }
}

impl super::Playload for Spawn {
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
}
