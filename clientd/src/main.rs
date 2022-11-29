use clientd::server::run_server;
use clientd::{init_fedimint_client, map_subscribers_to_events, run_manager};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    // TODO: print error
    let cfg: PathBuf = std::env::args().nth(1).expect("no cfg").into();
    let fedimint_client = init_fedimint_client(cfg).await;

    let (manager_tx, manager_rx) = mpsc::channel(128);
    let manager_tx = Arc::new(manager_tx);

    let se_map = map_subscribers_to_events();
    let _server_addr = run_server(Arc::clone(&manager_tx), se_map.clone()).await?;

    run_manager(Arc::clone(&manager_tx), manager_rx, se_map, fedimint_client).await;

    Ok(())
}
