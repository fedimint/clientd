use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use strum::EnumIter;
use strum::IntoEnumIterator;
use tokio::sync::{broadcast, mpsc, oneshot};

pub mod server;

// TODO: remove pub and use this type to enforce invariants
#[derive(Debug, Serialize)]
pub struct CallbackResponse(pub String);

#[derive(Debug)]
pub enum ManagerMessage {
    CallHandler(RpcRequest, oneshot::Sender<CallbackResponse>),
    HandleEvent(Event),
}

#[derive(Debug)]
pub enum RpcRequest {
    HealthCheck(),
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    event_key: EventKey,
    data: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, EnumIter, Serialize)]
pub enum EventKey {
    Mock,
}
pub async fn run_manager(
    _manager_tx: Arc<mpsc::Sender<ManagerMessage>>, // we will give the manager_tx to worker-tasks that can emit events
    mut manager_rx: mpsc::Receiver<ManagerMessage>,
    mut subscribers: HashMap<EventKey, broadcast::Sender<Event>>,
) {
    while let Some(msg) = manager_rx.recv().await {
        match msg {
            ManagerMessage::CallHandler(request, callback) => {
                let response = handle_rpc_request(request);
                callback
                    .send(CallbackResponse(response))
                    .expect("the rpc-method closure did not drop the callback receiver");
            }
            ManagerMessage::HandleEvent(event) => {
                // the tx is broadcast so this will handle all the subscribers
                let subscriber = subscribers.get_mut(&event.event_key).expect(
                    "if the event exists it was inserted into the hashmap at initialization",
                );
                if subscriber.send(event).is_err() {
                    // we will do if let Err(e) => handle error later but for now make the linter happy
                    tracing::error!("error broadcasting the event to the subscribers");
                }
            }
        }
    }
}

pub fn map_subscribers_to_events() -> HashMap<EventKey, broadcast::Sender<Event>> {
    let mut se_map = HashMap::new();
    for event in EventKey::iter() {
        let (tx, _rx) = broadcast::channel(128);
        se_map.insert(event, tx);
    }
    se_map
}

fn handle_rpc_request(request: RpcRequest) -> String {
    match request {
        RpcRequest::HealthCheck() => "".to_string(),
    }
}
