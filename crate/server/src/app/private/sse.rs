use shared::message::Owner;

use axum::{
    extract,
    response::sse::{Event, KeepAlive, Sse},
    Extension,
};
use futures::stream::Stream;

use crate::db::model::User;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use super::route::SharedState;

pub async fn handler(
    Extension(user): Extension<User>,
    extract::State(state): extract::State<SharedState>,
) -> Sse<impl Stream<Item = Result<Event, serde_json::Error>>> {
    let user_id = user.pubkey;
    let stream = BroadcastStream::new(state.read().await.tx.subscribe())
        .map(|value| value.ok().unwrap())
        // .filter(move |message| Owner::owner(message) == user_id || message.is_health())
        .map(|data| Event::default().json_data(data));
    Sse::new(stream).keep_alive(KeepAlive::default())
}
