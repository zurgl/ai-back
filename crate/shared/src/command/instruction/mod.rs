pub mod process;
pub use process::Process;

pub mod kill;
pub use kill::Kill;

pub mod pause;
pub use pause::Pause;

pub mod resume;
pub use resume::Resume;

pub mod spawn;
pub use spawn::Spawn;

use crate::types::CommandType;
use crate::types::ModelType;

pub trait Instruction: Send + Sync {
    fn command_type(&self) -> CommandType;
    fn model_type(&self) -> ModelType;
    fn task_id(&self) -> Option<String>;
    fn json_input(&self) -> Option<String>;
    fn owner(&self) -> String;
    fn timestamp(&self) -> u128;
}

pub trait Asking: Send + Sync {
    fn is_paused_with_owner(&self, owner: &str) -> bool;
    fn is_resumed_with_owner(&self, owner: &str) -> bool;
}

impl Asking for Box<dyn Instruction> {
    fn is_paused_with_owner(&self, owner: &str) -> bool {
        match self.command_type() {
            CommandType::Pause => self.owner() == owner,
            _ => false,
        }
    }

    fn is_resumed_with_owner(&self, owner: &str) -> bool {
        match self.command_type() {
            CommandType::Resume => self.owner() == owner,
            _ => false,
        }
    }
}
