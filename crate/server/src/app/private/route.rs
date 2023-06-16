use axum::{
    middleware,
    routing::{get, post},
};
use shared::constants::route;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

pub type SharedState = Arc<RwLock<State>>;
use shared::{command::Command, message::Message};

use super::{command, sse};
use crate::db;

#[derive(Debug)]
pub struct State {
    pub http_tx: tokio::sync::mpsc::Sender<Command>,
    pub tx: broadcast::Sender<Message>,
    pub pool: Arc<sqlx::Pool<sqlx::Postgres>>,
    pub config: db::Config,
}

impl State {
    pub fn new(
        http_tx: tokio::sync::mpsc::Sender<Command>,
        tx: broadcast::Sender<Message>,
        pool: Arc<sqlx::Pool<sqlx::Postgres>>,
        config: db::Config,
    ) -> Self {
        Self {
            http_tx,
            tx,
            pool,
            config,
        }
    }
}

pub async fn build(
    http_tx: tokio::sync::mpsc::Sender<Command>,
    tx: broadcast::Sender<Message>,
    pool: Arc<sqlx::Pool<sqlx::Postgres>>,
    config: db::Config,
) -> axum::Router {
    let state: SharedState = Arc::new(RwLock::new(State::new(http_tx, tx, pool, config)));

    let routes = axum::Router::new()
        .route(route::SSE_URL, get(sse::handler))
        .route(
            route::API_COMMAND_PROCESS_URL,
            post(command::handler::<command::playload::Process>),
        )
        .route(
            route::API_COMMAND_KILL_URL,
            post(command::handler::<command::playload::Kill>),
        )
        .route(
            route::API_COMMAND_SPAWN_URL,
            post(command::handler::<command::playload::Spawn>),
        )
        .route("/logout", get(super::logout::handler))
        .route("/profile", get(super::profile::handler))
        .route_layer(middleware::from_fn_with_state(state.clone(), super::auth))
        .nest_service("/images", ServeDir::new("images"))
        .with_state(state);

    axum::Router::new().nest_service("/api", routes)
}
