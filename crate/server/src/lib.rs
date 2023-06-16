use axum_server::HttpConfig;
use shared::{
    command::Command,
    constants::{chan, server::PORT},
    message::Message,
    types::ModelType,
};
use std::net::SocketAddr;
use tokio::sync::{broadcast, mpsc};

pub mod app;
pub mod cors;
pub mod db;
pub mod tls;

use actors::supervisor;

pub async fn run(ip: Option<String>, port: Option<String>) -> Result<(), &'static str> {
    console_subscriber::init();

    let (htx, srx) = mpsc::channel::<Command>(chan::MPSC_LEN);
    let (stx, _unused) = broadcast::channel::<Message>(chan::BORDCAST_LEN);
    let tx = stx.clone();

    let ip = ip
        .map(|value| serde_json::from_str(&value).expect("env must be defined"))
        .unwrap_or([127, 0, 0, 1]);

    let port = port
        .map(|value| serde_json::from_str(&value).expect("env must be defined"))
        .unwrap_or(PORT);

    let axum = tokio::task::Builder::new()
        .name("axum server")
        .spawn(async move {
            let socket = SocketAddr::from((ip, port));
            println!("socket: {socket:?}");

            let router = app::router(htx, tx).await;
            let tls_config = tls::config_load(true).await;
            let http_config = HttpConfig::new().http2_only(true).build();

            axum_server::bind_rustls(socket, tls_config)
                .http_config(http_config)
                .serve(router.into_make_service())
                .await
                .unwrap();
        })
        .map_err(|_| "Cannot spawn axum server")?;

    let supervisor = tokio::task::Builder::new()
        .name("supervisor")
        .spawn(async move {
            supervisor::run(stx, srx, vec![ModelType::Sentiment, ModelType::Translation]).await
        })
        .map_err(|_| "Cannot spawn supervisor")?;

    axum.await
        .and(supervisor.await)
        .map_err(|_| "Unexpected error")
}
