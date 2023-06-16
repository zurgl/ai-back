use tokio::sync::mpsc;

use shared::command::instruction::{self, Instruction};
use shared::constants;

pub async fn should_task_pause(rx: &mut mpsc::Receiver<Box<dyn Instruction>>, owner: &str) -> bool {
    tokio::select! {
      answer = async {
        if let Some(message) = rx.recv().await {
          <dyn instruction::Asking>::is_paused_with_owner(&message, owner)
        } else {
          false
        }
      } => answer,
      _ = async {
        shared::tools::wait(constants::time::VOLATILE).await
      } => false
    }
}

pub async fn should_task_resume(rx: &mut mpsc::Receiver<Box<dyn Instruction>>, owner: &str) {
    tokio::select! {
      _ = async {
        if let Some(message) = rx.recv().await {
          if <dyn instruction::Asking>::is_resumed_with_owner(&message, owner) {}
        }
      } => (),
      _ = async {
        shared::tools::wait(constants::time::FOREVER).await
      } => ()
    }
}
