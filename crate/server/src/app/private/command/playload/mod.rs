use shared::types::{CommandType, ModelType};

pub trait Playload: Send + Sync {
    fn command_type(&self) -> CommandType;
    fn model_type(&self) -> ModelType;
    fn task_id(&self) -> Option<String>;
    fn json_input(&self) -> Option<String>;
}

pub mod spawn;
pub use spawn::Spawn;

pub mod resume;
pub use resume::Resume;

pub mod pause;
pub use pause::Pause;

pub mod process;
pub use process::Process;

pub mod kill;
pub use kill::Kill;
