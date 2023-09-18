use axum::{
    extract,
    response::sse::{Event, KeepAlive, Sse},
    Extension,
};
use futures::stream::Stream;
use shared::types::MessageType;

use crate::db::model::User;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use super::route::SharedState;

pub async fn handler(
    Extension(user): Extension<User>,
    extract::State(state): extract::State<SharedState>,
) -> Sse<impl Stream<Item = Result<Event, serde_json::Error>>> {
    let _user_id = user.pubkey;
    let stream = BroadcastStream::new(state.read().await.tx.subscribe())
        .map(|value| value.ok().unwrap())
        // .filter(move |message| Owner::owner(message) == user_id || message.is_health())
        .map(|data| {
            match data.value() {
                None => (),
                Some(value) => {
                    if data.message_type() != MessageType::Health {
                        println!("{value:?}");
                    }
                }
            };
            Event::default().json_data(data)
        });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
