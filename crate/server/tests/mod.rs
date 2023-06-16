use shared::constants::chan;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use shared::types::{
    CommandType, MessageType,
    ModelType::{Sentiment, Summarize},
};
use shared::{command::Command, message::Message};

pub mod framework;
use framework::{build_app, run_axum, run_supervisor};

pub mod sse;
use sse::spawn_sse_listener;

pub mod query;
use query::{get_cookie, post_cmd};

pub mod utils;

use crate::utils::validate;

const PAUSE_TIME: u64 = 2_000;

#[tokio::test]
async fn run() {
    console_subscriber::init();

    let messages = Arc::new(Mutex::new(Vec::<Message>::new()));
    let task_id = Arc::new(Mutex::new(None));

    let (http_tx, supervisor_rx) = tokio::sync::mpsc::channel::<Command>(chan::MPSC_LEN);
    let (supervisor_tx, _unused) = broadcast::channel::<Message>(chan::BORDCAST_LEN);

    let app = build_app(supervisor_tx.clone(), http_tx).await;
    let _axum = run_axum(app.clone()).await;
    let _supervisor = run_supervisor(supervisor_tx, supervisor_rx).await;
    shared::tools::wait(PAUSE_TIME).await;

    let cookie = get_cookie(&app).await;
    spawn_sse_listener(
        Arc::clone(&messages),
        cookie.to_string(),
        Arc::clone(&task_id),
    )
    .await;
    shared::tools::wait(PAUSE_TIME).await;

    /*
        NEXT TEST MODEL SENTIMENTS
    */
    post_cmd(&app, &cookie, Sentiment, CommandType::Spawn, None).await;
    shared::tools::wait(PAUSE_TIME).await;

    let id = task_id.lock().await.clone().unwrap().clone();
    post_cmd(&app, &cookie, Sentiment, CommandType::Process, Some(&id)).await;
    shared::tools::wait(PAUSE_TIME).await;

    post_cmd(&app, &cookie, Sentiment, CommandType::Kill, Some(&id)).await;
    shared::tools::wait(PAUSE_TIME).await;

    /*
        NEXT TEST MODEL SUMMARIZE
    */
    post_cmd(&app, &cookie, Summarize, CommandType::Spawn, None).await;
    shared::tools::wait(PAUSE_TIME).await;

    let id = task_id.lock().await.clone().unwrap().clone();
    post_cmd(&app, &cookie, Summarize, CommandType::Process, Some(&id)).await;
    shared::tools::wait(5_000).await;

    post_cmd(&app, &cookie, Summarize, CommandType::Kill, Some(&id)).await;
    shared::tools::wait(PAUSE_TIME).await;

    //println!("{messages:?}");

    use std::cmp::Ordering::{Equal, Greater};
    assert!(validate(&messages, &MessageType::CommandFailed, Equal, 0).await);
    assert!(validate(&messages, &MessageType::CommandSucess, Equal, 6).await);
    assert!(validate(&messages, &MessageType::ModelPrediction, Equal, 2).await);
    assert!(validate(&messages, &MessageType::ModelLoaded, Equal, 2).await);
    assert!(validate(&messages, &MessageType::ModelStarted, Equal, 2).await);
    assert!(validate(&messages, &MessageType::ModelKilled, Equal, 2).await);
    assert!(validate(&messages, &MessageType::Health, Greater, 1).await);
}
