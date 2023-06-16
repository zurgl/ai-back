use shared::tools::wait;

use futures::Future;
use shared::message::Message;
use shared::types::MessageType;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn validate(
    messages: &Arc<Mutex<Vec<Message>>>,
    tag: &MessageType,
    ord: std::cmp::Ordering,
    value: usize,
) -> bool {
    messages
        .lock()
        .await
        .iter()
        .filter(|&message| message.message_type() == *tag)
        .count()
        .cmp(&value)
        == ord
}

pub async fn parked_for(delay: u64, callback: impl Future<Output = ()>) {
    callback.await;
    wait(delay).await;
}
