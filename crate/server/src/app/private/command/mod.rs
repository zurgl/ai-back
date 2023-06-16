pub mod playload;

use axum::{extract, http::StatusCode, Extension, Json};
use shared::{command::instruction, command::Command, types::CommandType};

use playload::Playload;

use crate::db::model::User;

use super::route::SharedState;

pub async fn handler<T>(
    Extension(user): Extension<User>,
    extract::State(state): extract::State<SharedState>,
    Json(payload): Json<T>,
) -> (StatusCode, Json<T>)
where
    T: Playload + std::fmt::Debug,
{
    let command = command_from(&user.pubkey, &payload);
    state.read().await.http_tx.send(command).await.unwrap();
    (StatusCode::CREATED, Json(payload))
}

fn command_from<T: Playload + std::fmt::Debug>(user_id: &str, payload: &T) -> Command {
    let tag = payload.command_type();
    match tag {
        CommandType::Process => Command::Process(instruction::Process {
            timestamp: shared::tools::time(),
            owner: user_id.to_string(),
            command_type: CommandType::Process,
            model_type: payload.model_type(),
            task_id: payload.task_id(),
            json_input: payload.json_input().unwrap(),
        }),
        CommandType::Kill => Command::Kill(instruction::Kill {
            timestamp: shared::tools::time(),
            owner: user_id.to_string(),
            command_type: CommandType::Kill,
            model_type: payload.model_type(),
            task_id: payload.task_id().unwrap(),
        }),
        CommandType::Pause => Command::Pause(instruction::Pause {
            timestamp: shared::tools::time(),
            owner: user_id.to_string(),
            command_type: CommandType::Pause,
            model_type: payload.model_type(),
            task_id: payload.task_id().unwrap(),
        }),
        CommandType::Resume => Command::Resume(instruction::Resume {
            timestamp: shared::tools::time(),
            owner: user_id.to_string(),
            command_type: CommandType::Resume,
            model_type: payload.model_type(),
            task_id: payload.task_id().unwrap(),
        }),
        CommandType::Spawn => Command::Spawn(instruction::Spawn {
            timestamp: shared::tools::time(),
            owner: user_id.to_string(),
            command_type: CommandType::Spawn,
            model_type: payload.model_type(),
        }),
    }
}
