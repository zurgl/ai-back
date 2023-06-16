use tokio::sync::{broadcast, mpsc};

use models::translation::Translation;
use shared::command::instruction::Instruction;
use shared::message::{
    emit::{Emit, EmitSource},
    Message,
};
use shared::types::MessageType;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
struct Input {
    input: String,
    owner: String,
    source_lang: String,
    target_lang: String,
}

pub async fn run(
    mut rx: mpsc::Receiver<Box<dyn Instruction>>,
    tx: broadcast::Sender<Message>,
    source: EmitSource,
) -> Result<(), String> {
    let source = source.set_task_id(tokio::task::id());

    MessageType::ModelStarted
        .emit(&tx, source.clone(), None)
        .map_err(|_| format!("Json Parse Error"))?;

    let model = Translation::default();

    MessageType::ModelLoaded
        .emit(&tx, source.clone(), None)
        .map_err(|err| format!("{err}"))?;

    while let Some(instruction) = rx.recv().await {
        let json_str = instruction.json_input().unwrap();
        let json_parsing_result: Result<Input, String> =
            serde_json::from_str(&json_str).map_err(|_| format!("Json Parse Error"));

        match json_parsing_result {
            Ok(params) => {
                let Input {
                    input,
                    owner,
                    source_lang,
                    target_lang,
                } = params;
                let new_source = source.clone();
                let new_source = new_source.set_owner(&owner);
                let message = model.prediction(&input, &source_lang, &target_lang);
                MessageType::ModelPrediction
                    .emit(&tx, new_source.clone(), Some(message.as_str()))
                    .map_err(|err| format!("{err}"))?;
            }
            Err(error) => {
                MessageType::ModelError
                    .emit(&tx, source.clone(), Some(error.as_str()))
                    .map_err(|err| format!("{err}"))?;
            }
        }
    }

    Ok(())
}
