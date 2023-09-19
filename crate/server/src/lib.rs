use axum_server::HttpConfig;
use shared::{command::Command, constants::chan, message::Message, types::ModelType};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::sync::{broadcast, mpsc};

pub mod app;
pub mod cors;
pub mod db;
pub mod tls;

use actors::supervisor;

pub async fn run() -> Result<(), &'static str> {
    console_subscriber::init();

    let (htx, srx) = mpsc::channel::<Command>(chan::MPSC_LEN);
    let (stx, _unused) = broadcast::channel::<Message>(chan::BORDCAST_LEN);
    let tx = stx.clone();

    let ip = option_env!("IP")
        .unwrap_or("127.0.0.1")
        .parse::<Ipv4Addr>()
        .expect("unable to parse ip");

    let port = option_env!("PORT")
        .unwrap_or("7443")
        .parse::<u16>()
        .expect("unable to parse port");

    let axum = tokio::task::Builder::new()
        .name("axum server")
        .spawn(async move {
            let socket = SocketAddr::from((ip, port));
            println!("socket: {socket:?}");

            let router = app::router(htx, tx).await;
            let tls_config = tls::config_load(false).await;
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
            // supervisor::run(stx, srx, vec![ModelType::Sentiment, ModelType::Translation]).await
            supervisor::run(stx, srx, vec![ModelType::Diffusion]).await
        })
        .map_err(|_| "Cannot spawn supervisor")?;

    axum.await
        .and(supervisor.await)
        .map_err(|_| "Unexpected error")
}
