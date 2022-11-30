use crate::{DispatcherMessage, EventSubscribers, RpcRequest};
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub struct Context {
    dispatcher_tx: Arc<mpsc::Sender<DispatcherMessage>>,
}
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
fn register_subscriptions(
    _module: &mut RpcModule<Context>,
    _subscribers: EventSubscribers,
) -> anyhow::Result<()> {
    // TODO: register subscriptions for events like NewCoinsFetched et.c
    // we will hardcode the subscription closure to the EventKey. not beautiful but worth avoiding complexity in design
    Ok(())
}
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
