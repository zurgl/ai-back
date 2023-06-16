pub mod instruction;

use crate::tools::root;
use crate::types::CommandType;
use crate::types::ModelType;

use self::instruction::Instruction;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Command {
    Process(instruction::Process),
    Kill(instruction::Kill),
    Pause(instruction::Pause),
    Resume(instruction::Resume),
    Spawn(instruction::Spawn),
}

impl Command {
    pub fn boxed_instruction(&self) -> Box<dyn Instruction> {
        match self {
            Command::Process(instruction) => Box::new(instruction.clone()),
            Command::Kill(instruction) => Box::new(instruction.clone()),
            Command::Pause(instruction) => Box::new(instruction.clone()),
            Command::Resume(instruction) => Box::new(instruction.clone()),
            Command::Spawn(instruction) => Box::new(instruction.clone()),
        }
    }

    pub fn spawn(model_type: ModelType) -> Self {
        Command::Spawn(instruction::Spawn {
            command_type: CommandType::Spawn,
            timestamp: crate::tools::time(),
            model_type,
            owner: crate::constants::role::ROOT.to_owned(),
        })
    }

    pub fn process(id: &str, input: &str, model_type: ModelType) -> Self {
        Command::Process(instruction::Process {
            command_type: CommandType::Process,
            timestamp: crate::tools::time(),
            model_type,
            owner: root(),
            task_id: Some(id.to_owned()),
            json_input: input.to_owned(),
        })
    }

    pub fn kill(id: &str, model_type: ModelType) -> Self {
        Command::Kill(instruction::Kill {
            command_type: CommandType::Kill,
            timestamp: crate::tools::time(),
            model_type,
            owner: root(),
            task_id: id.to_owned(),
        })
    }
}
