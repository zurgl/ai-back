use models::summarize::Summarize;
use tokio::sync::{broadcast, mpsc};

use shared::types::MessageType;
use shared::{
    command::instruction::Instruction,
    message::{
        emit::{Emit, EmitSource},
        Message,
    },
};

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
struct Input {
    input: String,
}

pub async fn run(
    mut rx: mpsc::Receiver<Box<dyn Instruction>>,
    tx: broadcast::Sender<Message>,
    source: EmitSource,
) -> Result<(), String> {
    let source = source.set_task_id(tokio::task::id());

    if let Err(err) = MessageType::ModelStarted.emit(&tx, source.clone(), None) {
        eprintln!("{err:#?}");
    }

    let model = Summarize::default();

    if let Err(err) = MessageType::ModelLoaded.emit(&tx, source.clone(), None) {
        eprintln!("{err:#?}");
    }

    while let Some(command) = rx.recv().await {
        let owner = command.owner();
        let source = source.set_owner(&owner);

        let json_str = command.json_input().unwrap();
        let json_parsing_result: Result<Input, String> =
            serde_json::from_str(&json_str).map_err(|_| format!("Json Parse Error"));

        match json_parsing_result {
            Ok(params) => {
                let input = params.input;
                let message = model.prediction(&input);
                if let Err(err) =
                    MessageType::ModelPrediction.emit(&tx, source.clone(), Some(message.as_str()))
                {
                    eprintln!("{err:#?}");
                }
            }
            Err(error) => {
                if let Err(err) =
                    MessageType::ModelError.emit(&tx, source.clone(), Some(error.as_str()))
                {
                    eprintln!("{err:#?}");
                }
            }
        }
    }

    Ok(())
}
