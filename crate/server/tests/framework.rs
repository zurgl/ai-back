use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    sync::{self, RwLock},
    task::JoinHandle,
};

use server::{
    app::{app, state::State},
    cors,
};

use actors::supervisor;
use shared::types::ModelType;
use shared::{
    command::Command,
    constants::{server::ADDRESS, server::PORT},
    message::Message,
};

pub async fn build_app(
    supervisor_tx: sync::broadcast::Sender<Message>,
    http_tx: sync::mpsc::Sender<Command>,
) -> Router {
    let supervisor_tx_clone = supervisor_tx;
    let state = State::new(http_tx, supervisor_tx_clone, None, None);
    let cors = cors::load();
    app(Arc::new(RwLock::new(state)), cors)
}

pub async fn run_axum(app: Router) -> JoinHandle<()> {
    let axum = tokio::task::Builder::new()
        .name("axum")
        .spawn(async move {
            println!("listening on {:?}:{}", ADDRESS, PORT);
            axum::Server::bind(&SocketAddr::from((ADDRESS, PORT)))
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .unwrap();
        })
        .unwrap();

    axum
}

pub async fn run_supervisor(
    supervisor_tx: sync::broadcast::Sender<Message>,
    supervisor_rx: sync::mpsc::Receiver<Command>,
) -> JoinHandle<()> {
    let supervisor = tokio::task::Builder::new()
        .name("supervisor")
        .spawn(async move {
            supervisor::run(
                supervisor_tx,
                supervisor_rx,
                vec![ModelType::Summarize, ModelType::Sentiment],
            )
            .await
        })
        .unwrap();

    supervisor
}
