use fedimint_client::{Client, UserClientConfig};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use strum::EnumIter;
use strum::IntoEnumIterator;
use tokio::sync::{broadcast, mpsc, oneshot};

mod rpc;
pub mod server;

#[derive(Clone)]
pub struct EventSubscribers(HashMap<EventKey, broadcast::Sender<Event>>);

impl EventSubscribers {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        for event in EventKey::iter() {
            let (tx, _rx) = broadcast::channel(128);
            map.insert(event, tx);
        }
        Self(map)
    }

    // will be used later
    pub fn _subscribe(&self, event_key: &EventKey) -> broadcast::Receiver<Event> {
        self.0
            .get(event_key)
            .expect("if the event exists it was inserted into the hashmap at initialization")
            .subscribe()
    }

    pub fn send(&self, event: Event) -> Result<usize, broadcast::error::SendError<Event>> {
        let tx = self
            .0
            .get(&event.event_key)
            .expect("if the event exists it was inserted into the hashmap at initialization");
        tx.send(event)
    }
}

impl Default for EventSubscribers {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: remove pub and use this type to enforce invariants
#[derive(Debug, Serialize)]
pub struct CallbackResponse(pub Result<String, RpcError>);

// TODO: figure out best approach for error handling
#[derive(Debug, Serialize)]
pub enum RpcError {
    ClientError,
}
#[derive(Debug)]
pub enum DispatcherMessage {
    CallHandler(RpcRequest, oneshot::Sender<CallbackResponse>),
    HandleEvent(Event),
}

#[derive(Debug)]
pub enum RpcRequest {
    HealthCheck(),
    Info(),
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
pub async fn run_dispatcher(
    _dispatcher_tx: Arc<mpsc::Sender<DispatcherMessage>>, // we will give the dispatcher_tx to worker-tasks that can emit events
    mut dispatcher_rx: mpsc::Receiver<DispatcherMessage>,
    subscribers: EventSubscribers,
    mut fedimint_client: Client<UserClientConfig>,
) {
    while let Some(msg) = dispatcher_rx.recv().await {
        match msg {
            DispatcherMessage::CallHandler(request, callback) => {
                let response = handle_rpc_request(request, &mut fedimint_client).await;
                callback
                    .send(CallbackResponse(response))
                    .expect("the rpc-method closure did not drop the callback receiver");
            }
            DispatcherMessage::HandleEvent(event) => {
                // the tx is broadcast so this will handle all the subscribers
                if subscribers.send(event).is_err() {
                    // we will do if let Err(e) => handle error later but for now make the linter happy
                    tracing::error!("error broadcasting the event to the subscribers");
                }
            }
        }
    }
}

pub async fn init_fedimint_client(cfg: PathBuf) -> Client<UserClientConfig> {
    let cfg_path = cfg.join("client.json");
    let db_path = cfg.join("client.db");
    let cfg: UserClientConfig = load_from_file(&cfg_path);
    let db = fedimint_rocksdb::RocksDb::open(db_path).unwrap().into();

    Client::new(cfg.clone(), db, Default::default()).await
}

// FIXME: jsonrpsee will do the serialization for me. how can I just pass any type over the channel, the serialize -> string -> serialize feels a bit stupid
async fn handle_rpc_request(
    request: RpcRequest,
    fedimint_client: &mut Client<UserClientConfig>,
) -> Result<String, RpcError> {
    match request {
        RpcRequest::HealthCheck() => Ok("".to_string()),
        RpcRequest::Info() => rpc::info(fedimint_client)
            .await
            .map(|info_res| serde_json::to_string(&json!(info_res)).unwrap()),
    }
}

fn load_from_file<T: DeserializeOwned>(path: &Path) -> T {
    let file = std::fs::File::open(path).expect("Can't read cfg file.");
    serde_json::from_reader(file).expect("Could not parse cfg file.")
}
