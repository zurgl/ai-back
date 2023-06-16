// cargo test -p diffusion --test '*' -- --nocapture
#![allow(dead_code, unused_assignments, unused_variables, unused_imports)]
#![feature(async_closure)]
use actors::supervisor;
use shared::types::MessageType;
use shared::types::ModelType;
use shared::{command::Command, constants, message::Message, tools};

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};

pub mod input;
use input::Input;

pub mod helper;
use helper::validate;

#[tokio::test]
async fn run() -> Result<(), &'static str> {
    console_subscriber::init();

    let messages = Arc::new(Mutex::new(Vec::<Message>::new()));
    let task_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let (http_tx, supervisor_rx) = mpsc::channel::<Command>(constants::chan::MPSC_LEN);
    let (supervisor_tx, mut http_rx) = broadcast::channel::<Message>(constants::chan::BORDCAST_LEN);

    let message_clone = messages.clone();
    let task_id_clone = task_id.clone();
    tokio::task::Builder::new()
        .name("listener")
        .spawn(async move || -> Result<(), String> {
            while let Ok(message) = http_rx.recv().await {
                if !message.is_health() && message.task_id().is_some() {
                    task_id_clone
                        .lock()
                        .await
                        .replace(message.task_id().unwrap());
                }
                let mut messages = message_clone.lock().await;
                messages.push(message);
            }
            Ok(())
        }())
        .map_err(|_| "Cannot spawn the listener")?;

    tokio::task::Builder::new()
        .name("supervisor")
        .spawn(async move {
            supervisor::run(
                supervisor_tx,
                supervisor_rx,
                vec![ModelType::Summarize, ModelType::Sentiment],
            )
            .await
        })
        .map_err(|_| "Cannot spawn the supervisor")?;
    tools::wait(250).await;

    /*
        NEXT TEST MODEL SENTIMENTS
    */
    let command = Command::spawn(ModelType::Sentiment);
    http_tx.send(command).await.map_err(|_| "Cannot send")?;
    tools::wait(50).await;

    let current_id = task_id.lock().await.clone().unwrap().clone();
    let input = Input::input(&ModelType::Sentiment);
    let command = Command::process(&current_id, &input, ModelType::Sentiment);
    http_tx.send(command).await.map_err(|_| "Cannot send")?;
    tools::wait(50).await;

    let current_id = task_id.lock().await.clone().unwrap().clone();
    let command = Command::kill(&current_id, ModelType::Sentiment);
    http_tx.send(command).await.map_err(|_| "Cannot send")?;
    tools::wait(50).await;

    /*
        NEXT TEST MODEL SUMMARIZE
    */
    let command = Command::spawn(ModelType::Summarize);
    http_tx.send(command).await.map_err(|_| "Cannot send")?;
    tools::wait(50).await;

    let current_id = task_id.lock().await.clone().unwrap().clone();
    let input = Input::input(&ModelType::Summarize);
    let command = Command::process(&current_id, &input, ModelType::Summarize);
    http_tx.send(command).await.map_err(|_| "Cannot send")?;
    tools::wait(50).await;

    let current_id = task_id.lock().await.clone().unwrap().clone();
    let command = Command::kill(&current_id, ModelType::Summarize);
    http_tx.send(command).await.map_err(|_| "Cannot send")?;
    tools::wait(50).await;

    use std::cmp::Ordering::{Equal, Greater};
    assert!(validate(&messages, &MessageType::CommandFailed, Equal, 0).await);
    assert!(validate(&messages, &MessageType::CommandSucess, Equal, 6).await);
    assert!(validate(&messages, &MessageType::ModelPrediction, Equal, 2).await);
    assert!(validate(&messages, &MessageType::ModelLoaded, Equal, 4).await);
    assert!(validate(&messages, &MessageType::ModelStarted, Equal, 4).await);
    assert!(validate(&messages, &MessageType::ModelKilled, Equal, 2).await);
    assert!(validate(&messages, &MessageType::Health, Greater, 1).await);

    //println!("{messages:#?}");

    Ok(())
}
