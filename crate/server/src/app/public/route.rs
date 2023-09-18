use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension,
};
use tower_http::services::ServeDir;

#[derive(Debug)]
pub struct State {
    pub db: Arc<sqlx::Pool<sqlx::Postgres>>,
    pub env: crate::db::Config,
}

impl State {
    pub fn new(pool: Arc<sqlx::Pool<sqlx::Postgres>>, config: crate::db::Config) -> Arc<Self> {
        Arc::new(Self {
            db: pool,
            env: config,
        })
    }
}

async fn health() -> impl axum::response::IntoResponse {
    axum::response::Json(serde_json::json!({ "status": "alive" }))
}

pub async fn build(
    pool: Arc<sqlx::Pool<sqlx::Postgres>>,
    config: crate::db::Config,
) -> axum::Router {
    let state = State::new(pool, config);

    let key = std::option_env!("CSRF_KEY").expect("CSRF_KEY env not set");
    let key = csrf::Key::from(key.as_bytes());
    let config = csrf::CsrfConfig::default()
        .with_key(Some(key))
        .with_secure(true)
        .with_cookie_same_site(csrf::SameSite::None)
        .with_http_only(false);

    axum::Router::new()
        .route("/health", get(health))
        .route("/login", post(super::login::handler))
        .route("/", get(super::handler))
        .nest_service("/images", ServeDir::new("images"))
        .layer(Extension(state))
        .with_state(config)
}
