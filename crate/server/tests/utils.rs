use shared::message::Message;
use shared::types::MessageType;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn validate(
    messages: &Arc<Mutex<Vec<Message>>>,
    message_type: &MessageType,
    ord: std::cmp::Ordering,
    value: usize,
) -> bool {
    messages
        .lock()
        .await
        .iter()
        .filter(|&message| message.message_type() == *message_type)
        .count()
        .cmp(&value)
        == ord
}
