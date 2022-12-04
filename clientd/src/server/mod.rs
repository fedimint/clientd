use crate::{DispatcherMessage, EventSubscribers, RpcRequest};
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// The server context containing a sender to the dispatcher to relay the rpc-calls to it.
pub struct Context {
    dispatcher_tx: Arc<mpsc::Sender<DispatcherMessage>>,
}

/// Registers methods on the rpc-module. The closures will send a message to the servers dispatcher alongside a callback for the result.
fn register_methods(module: &mut RpcModule<Context>) -> anyhow::Result<()> {
    module.register_async_method("health_check", |_, context| async move {
        let (cbtx, cbrx) = oneshot::channel();
        let rpc_req = DispatcherMessage::CallHandler(RpcRequest::HealthCheck(), cbtx);
        context.dispatcher_tx.send(rpc_req).await.unwrap();
        cbrx.await.unwrap()
    })?;

    module.register_async_method("info", |_, context| async move {
        let (cbtx, cbrx) = oneshot::channel();
        let rpc_req = DispatcherMessage::CallHandler(RpcRequest::Info(), cbtx);
        context.dispatcher_tx.send(rpc_req).await.unwrap();
        cbrx.await.unwrap()
    })?;
    Ok(())
}

/// Registers subscriptions on the rpc-module. The closures will subscribe to the correct event.
fn register_subscriptions(
    _module: &mut RpcModule<Context>,
    _subscribers: EventSubscribers,
) -> anyhow::Result<()> {
    // TODO: register subscriptions for events like NewCoinsFetched et.c
    // we will hardcode the subscription closure to the EventKey. not beautiful but worth avoiding complexity in design
    Ok(())
}

/// Initializes the rpc-module with the `Context` and register methods and subscriptions on it.
fn create_rpc_module(
    dispatcher_tx: Arc<mpsc::Sender<DispatcherMessage>>,
    subscribers: EventSubscribers,
) -> anyhow::Result<RpcModule<Context>> {
    let context = Context { dispatcher_tx };
    let mut module = RpcModule::new(context);
    register_methods(&mut module)?;
    register_subscriptions(&mut module, subscribers)?;
    Ok(module)
}

/// Runs the webserver and return the socket address it listens on. This will spawn the webserver which can be used over http or ws and
/// initializes the rpc-module by registering handlers for rpc-calls and subscriptions.
pub async fn run_server(
    dispatcher_tx: Arc<mpsc::Sender<DispatcherMessage>>,
    subscribers: EventSubscribers,
) -> anyhow::Result<SocketAddr> {
    // TODO: Telemetry
    let server = ServerBuilder::default()
        .build("127.0.0.1:0".parse::<SocketAddr>()?)
        .await?;

    let module = create_rpc_module(dispatcher_tx, subscribers)?;
    let addr = server.local_addr()?;
    tracing::info!("server address: {}", addr);
    let handle = server.start(module)?;

    // TODO: Graceful shutdown
    tokio::spawn(handle.stopped());

    Ok(addr)
}
