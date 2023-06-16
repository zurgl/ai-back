pub mod private;
pub mod public;

use shared::{command::Command, message::Message};
use tokio::sync::broadcast;

pub async fn router(
    http_tx: tokio::sync::mpsc::Sender<Command>,
    tx: broadcast::Sender<Message>,
) -> axum::Router {
    let (pool, config) = crate::db::init().await;

    public::route::build(pool.clone(), config.clone())
        .await
        .merge(private::route::build(http_tx, tx, pool, config).await)
        .layer(crate::cors::load())
}
