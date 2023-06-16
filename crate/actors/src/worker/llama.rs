use tokio::sync::{broadcast, mpsc};

use models::llama::Llama;
use shared::command::instruction::Instruction;
use shared::message::{
    emit::{Emit, EmitSource},
    Message,
};
use shared::types::MessageType;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
struct Input {
    prompt: String,
    sample_len: usize,
    temperature: f64,
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

    let model = Llama::default();

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
                    prompt,
                    sample_len,
                    temperature,
                } = params;
                let message = model.prediction(&prompt, sample_len, temperature, Some(tx.clone()));
                MessageType::ModelPrediction
                    .emit(&tx, source.clone(), Some(message.as_str()))
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
