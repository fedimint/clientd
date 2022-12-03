use crate::rpc::responses::HandlerResponse;
use fedimint_client::{Client, UserClientConfig};
use jsonrpsee::core::Error;
use jsonrpsee::types::error::CallError;
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

// TODO: think about using only one broadcast channel and filter for events
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

    pub fn subscribe(&self, event_key: &EventKey) -> broadcast::Receiver<Event> {
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

pub struct FedimintClient(Client<UserClientConfig>);

impl FedimintClient {
    pub async fn open_from(client_workdir: PathBuf) -> Self {
        let cfg_path = client_workdir.join("client.json");
        let db_path = client_workdir.join("client.db");
        let cfg: UserClientConfig = load_from_file(&cfg_path);
        let db = fedimint_rocksdb::RocksDb::open(db_path).unwrap().into();

        let client = Client::new(cfg.clone(), db, Default::default()).await;
        Self(client)
    }

    pub async fn fake() -> Self {
        // TODO: make the config configurable (integrationtests/fixtures)
        let fake_cfg = json!({
                  "federation_name": "Hals_trusty_mint",
                  "nodes": [],
                  "modules": {
                      "ln": {
                          "fee_consensus": {
                              "contract_input": 0,
                              "contract_output": 0
                          },
                          "threshold_pub_key": "b3f2f47dbb31c9c9becb98c8f1125a281ba3e4f32e81bd5367fdffb994d37e85\
                          b32ec70535a805d2bc21effb391721e5"
                      },
                      "mint": {
                          "fee_consensus": {
                              "coin_issuance_abs": 0,
                              "coin_spend_abs": 0
                          },
                          "tbs_pks": {
                              "1": "8fb88f2d02658995d6738a3f38601a71e6e6498815acdb2cf505480b65cdaad37bec62455eae06ba491144ebdb5\
                                  725af021d0235b22d1d0dbc85d2a54cf88a4532041ed36f4254c80badbc5f03184c2946cae3f051b2c4c55cbfddd6af566238",
                              "10": "a1c09650f3d109a07fa5a0afaea47df5665eb3ad652c16fc4edc04aa873c0cab03a97adbb6b8ce896330119bd7\
                                  82a52612747b4be6bdaecf2e3d77a4d886e94f4d11d3ecdffff5ef00616607e4eda1084449e4207c23cf0c465b8ac038afd326",
                              "100": "883860e09a1703e9988886264e86e45027fbc5df15fd09778ffcaad79a63c3f7d792839e67be5bd3fdfd35a46\
                                  b72b0ea08e7f62ca4286e50263d2fb68d68640f31524c8ac375c02b368187636e78f74055789af4cbfb74c41c0e0eb3b5d829a0",
                              "1000": "8e89328ebb74522b9a7c7f479a51a91f5e88ebfd8a4f862cc353e73c74a66e5068fb5b6afd9808637dcb7bc5\
                                  95bceae1086b98974f3722909dffa113af7b82c29248c86cfb5fa6a88566a7f02509807d6d03d101459c8d90c2ad5ea807e617d3",
                              "10000": "8018ea0d8f0d007e133896241fe316f4997e1242e4792ac9d19570464b79e711fed7f1f8ed93737f64caa79\
                                  e1ab0cc910fc3e02c3c7cf59be59a9c0fe2f516cd3fa06ef7f6b208e19537b38453eb6584dbe060736a54f82998e61b514e5e4617",
                              "100000": "b0d16d171b018eb38875805f80fd313831df53f7cfd1409bf2b6d363a0d8eebcb97f485c6559e5110fc617\
                                  41e988a44a17aa28e0f4ccf4d8a49ec13a71054e5002638efee3baafbda164ec951b40a5bae11f0cdd0e60d5fd7aa356450ca9fe81",
                              "1000000": "a183a431c64f66161bfc2719fe820d0cd83837a1c13b5fa3b8da6b29ee0e0fafd9c613e716e16fe90d0d6\
                                  169ceaeae690449923b66abd7c220c38d66a229750124701ba7850e0c4192d8471a9d6178bde5b4d1edb26b955c970600de18cd3fe3",
                              "10000000": "98df02744862134e723173002e0eb19d9909ca5f85a9297674513294b6eb3d8ecbb64d14f91b005f1bb2\
                                  7dae92623a1c0818638f638cc092e58351d768f3d829849c429c0a132366db43d9919e48b5f2e5c60cf60d583816c0e691b83e6fdf23",
                              "100000000": "a338c02e45e2e3a871de7f2d323d11dfeff833c79a6d3712b1de0eb411c48bdb7c496ac69ff30a40bbc\
                                  72adefdc71add0a70e8944fa7bc1515885142028089a5e7d69eb0f24e116eefc314771f7980110648d99e1538a9dc4d4ffbc1476a6fea",
                              "1000000000": "afd3fe615350dcd011987407b414b0ee0593135646ff3795b8bc8a9f355eb2e3e655f4673fd57f8151\
                                  fba2c4f1b4c698051c7b1fe03b360fea8335762474e8c9bfb01e68c1b8b9815c2fb3df5ffea2e41cf3a8c56d89ef85183a8acd1d8b705c"
                          }
                  },
                      "wallet": {
                                "fee_consensus": {
                                  "peg_in_abs": 0,
                                  "peg_out_abs": 0
                                },
                                "finality_delay": 10,
                                "network": "regtest",
                                "peg_in_descriptor": "wsh(sortedmulti(3,02abb4d16a11e50bf74f8c50bdbc9cfa4a6f3e14148278d0ea619f3b14d\
                          b14848a,02c5b4fade57cef6b3bc22b09155cb1c1174074ca6f32efbceb5a2a9c5dcb3926a,02455f4063052ff38ae417d4e0ef19\
                          e8de845b2e59b9a5ff6610e51d7ebc16368f,03186bd2615f96796eb26635f895df2b1a5c7012da6aa4050bb274b47a05b4e4e5))\
                          #ymdn2asg"
          }
        }
              });
        let fake_cfg: UserClientConfig = serde_json::from_value(fake_cfg).unwrap();
        let db = fedimint_api::db::mem_impl::MemDatabase::new().into();

        let client = Client::new(fake_cfg, db, Default::default()).await;
        Self(client)
    }
}

impl AsRef<Client<UserClientConfig>> for FedimintClient {
    fn as_ref(&self) -> &Client<UserClientConfig> {
        &self.0
    }
}

#[derive(Debug)]
pub enum DispatcherMessage {
    CallHandler(RpcRequest, oneshot::Sender<Result<HandlerResponse, Error>>),
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
    fedimint_client: FedimintClient,
) {
    while let Some(msg) = dispatcher_rx.recv().await {
        match msg {
            DispatcherMessage::CallHandler(request, callback) => {
                let response = handle_rpc_request(request, fedimint_client.as_ref())
                    .await
                    .map_err(Error::from);
                callback
                    .send(response)
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

async fn handle_rpc_request(
    request: RpcRequest,
    fedimint_client: &Client<UserClientConfig>,
) -> Result<HandlerResponse, CallError> {
    match request {
        RpcRequest::HealthCheck() => Ok(HandlerResponse::HealthCheck()),
        RpcRequest::Info() => rpc::info(fedimint_client).await,
    }
}

fn load_from_file<T: DeserializeOwned>(path: &Path) -> T {
    let file = std::fs::File::open(path).expect("Can't read cfg file.");
    serde_json::from_reader(file).expect("Could not parse cfg file.")
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn event_subscribers_can_be_used_to_send_and_recv() {
        // might be redundant because of the `dispatcher_handles_event` test but its just a small test anyway
        let subscribers = EventSubscribers::default();

        let mut rx = subscribers.subscribe(&EventKey::Mock);
        let subs2 = subscribers.clone();

        // the spawn might be unnecessary but it shows in what context this can be used in
        tokio::spawn(async move {
            for _ in 0..5 {
                subs2
                    .send(Event {
                        event_key: EventKey::Mock,
                        data: "test".to_string(),
                    })
                    .unwrap();
            }
        })
        .await
        .unwrap();

        // FIXME: refactor this to be more clever
        let mut results = vec![];
        for _ in 0..5 {
            if let Ok(msg) = rx.recv().await {
                results.push(msg);
            }
        }

        assert_eq!(
            results.into_iter().map(|e| e.data).collect::<Vec<String>>(),
            vec!["test".to_string(); 5]
        )
    }

    #[tokio::test]
    async fn dispatcher_handles_rpc() {
        let subscribers = EventSubscribers::default();
        let fm_client = FedimintClient::fake().await;
        let (dispatcher_tx, dispachter_rx) = mpsc::channel(8);
        let dispatcher_tx = Arc::new(dispatcher_tx);

        let dispatcher = run_dispatcher(
            Arc::clone(&dispatcher_tx),
            dispachter_rx,
            subscribers,
            fm_client,
        );
        let handle = tokio::spawn(dispatcher);

        let (cbtx, cbrx) = oneshot::channel();
        let req = RpcRequest::HealthCheck();
        let msg = DispatcherMessage::CallHandler(req, cbtx);

        dispatcher_tx.send(msg).await.unwrap();

        let timeout = tokio::time::sleep(Duration::from_secs(10));
        tokio::select! {
            _ = timeout => panic!("no response from dispatcher after timeout"),
            // we can cbrx.await in case this looks confusing
            Ok(cb_res) = cbrx => {
                let result = cb_res.unwrap();
                match result {
                    HandlerResponse::HealthCheck() => (),
                    _ => panic!("dispatcher answered incorrectly")
                }
            }
        }
        drop(handle);
    }

    #[tokio::test]
    async fn dispatcher_handles_event() {
        let subscribers = EventSubscribers::default();
        let fm_client = FedimintClient::fake().await;
        let mut mock_event_subscriber = subscribers.subscribe(&EventKey::Mock);
        let (dispatcher_tx, dispachter_rx) = mpsc::channel(8);
        let dispatcher_tx = Arc::new(dispatcher_tx);

        let dispatcher = run_dispatcher(
            Arc::clone(&dispatcher_tx),
            dispachter_rx,
            subscribers,
            fm_client,
        );
        let handle = tokio::spawn(dispatcher);

        let event = Event {
            event_key: EventKey::Mock,
            data: "test".to_string(),
        };
        dispatcher_tx
            .send(DispatcherMessage::HandleEvent(event))
            .await
            .unwrap();

        let timeout = tokio::time::sleep(Duration::from_secs(10));
        tokio::select! {
            _ = timeout => panic!("nothing received on the subscriber after timeout"),
            Ok(e) = mock_event_subscriber.recv() => {
                assert_eq!(e.data, "test".to_string())
            },
        }

        drop(handle);
    }
}
