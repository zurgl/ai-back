use eventsource_client as es;
use futures::{Stream, TryStreamExt};
use shared::constants::server::PORT;
use shared::message::Message;
use shared::types::MessageType;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn spawn_sse_listener(
    arced_messages: Arc<Mutex<Vec<Message>>>,
    cookie: String,
    task_id: Arc<Mutex<Option<String>>>,
) {
    tokio::task::Builder::new()
        .name("listener")
        .spawn(async move {
            let mut sse_stream = sse_stream(&cookie).ok().unwrap();

            while let Ok(Some(data)) = sse_stream.try_next().await {
                let json_msg: Message = serde_json::from_str(&data.clone().unwrap()).ok().unwrap();
                //println!("{json_msg:?}");
                if !(json_msg.message_type() == MessageType::Health) {
                    //println!("{json_msg:?}");
                    if json_msg.task_id().is_some() {
                        task_id.lock().await.replace(json_msg.task_id().unwrap());
                    }
                }

                let mut messages = arced_messages.lock().await;
                messages.push(json_msg);
            }
        })
        .unwrap();
}

fn tail_events(client: impl es::Client) -> impl Stream<Item = Result<Option<String>, ()>> {
    client
        .stream()
        .map_ok(|event| match event {
            es::SSE::Event(ev) => Some(ev.data),
            es::SSE::Comment(comment) => {
                println!("got a comment: \n{comment}");
                None
            }
        })
        .map_err(|err| eprintln!("error streaming events: {err:?}"))
}

fn sse_stream(cookie: &str) -> Result<impl Stream<Item = Result<Option<String>, ()>>, &str> {
    let client = es::ClientBuilder::for_url(&format!("http://localhost:{}/sse", PORT))
        .map_err(|_| "cannot build sse")?
        .header("Cookie", cookie)
        .map_err(|_| "Insert Cookie on header failed")?
        .build();

    Ok(tail_events(client))
}
