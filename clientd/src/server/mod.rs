use crate::{Event, EventKey, ManagerMessage, RpcRequest};
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot};

pub struct Context {
    manager_tx: Arc<mpsc::Sender<ManagerMessage>>,
}
fn register_methods(module: &mut RpcModule<Context>) -> anyhow::Result<()> {
    module.register_async_method("health_check", |_, context| async move {
        let (cbtx, cbrx) = oneshot::channel();
        let rpc_req = ManagerMessage::CallHandler(RpcRequest::HealthCheck(), cbtx);
        context.manager_tx.send(rpc_req).await.unwrap();
        Ok(cbrx.await.unwrap())
    })?;

    module.register_async_method("info", |_, context| async move {
        let (cbtx, cbrx) = oneshot::channel();
        let rpc_req = ManagerMessage::CallHandler(RpcRequest::Info(), cbtx);
        context.manager_tx.send(rpc_req).await.unwrap();
        Ok(cbrx.await.unwrap())
    })?;
    Ok(())
}
fn register_subscriptions(
    _module: &mut RpcModule<Context>,
    _se_map: HashMap<EventKey, broadcast::Sender<Event>>,
) -> anyhow::Result<()> {
    // TODO: register subscriptions for events like NewCoinsFetched et.c
    // we will hardcode the subscription closure to the EventKey. not beautiful but worth avoiding complexity in design
    Ok(())
}
fn create_rpc_module(
    manager_tx: Arc<mpsc::Sender<ManagerMessage>>,
    se_map: HashMap<EventKey, broadcast::Sender<Event>>,
) -> anyhow::Result<RpcModule<Context>> {
    let context = Context { manager_tx };
    let mut module = RpcModule::new(context);
    register_methods(&mut module)?;
    register_subscriptions(&mut module, se_map)?;
    Ok(module)
}
pub async fn run_server(
    manager_tx: Arc<mpsc::Sender<ManagerMessage>>,
    se_map: HashMap<EventKey, broadcast::Sender<Event>>,
) -> anyhow::Result<SocketAddr> {
    // TODO: Telemetry
    let server = ServerBuilder::default()
        .build("127.0.0.1:0".parse::<SocketAddr>()?)
        .await?;

    let module = create_rpc_module(manager_tx, se_map)?;
    let addr = server.local_addr()?;
    tracing::info!("server address: {}", addr);
    let handle = server.start(module)?;

    // TODO: Graceful shutdown
    tokio::spawn(handle.stopped());

    Ok(addr)
}
