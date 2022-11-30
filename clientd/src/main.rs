use clap::Parser;
use clientd::server::run_server;
use clientd::{run_dispatcher, EventSubscribers, FedimintClient};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

// TODO: I saw other people using yaml and .env files to configure their project. I think this might be a better approach but for now go with cmd-args
/// JSON-RPC 2.0 for the fedimint client
#[derive(Parser)]
struct Opt {
    /// The working directory of the client containing the config and db
    #[arg(long = "workdir")]
    workdir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    let client_workdir = Opt::parse();
    let fedimint_client = FedimintClient::open_from(client_workdir.workdir).await;

    let (dispatcher_tx, dispatcher_rx) = mpsc::channel(128);
    let dispatcher_tx = Arc::new(dispatcher_tx);

    let subscribers = EventSubscribers::default();

    let _server_addr = run_server(Arc::clone(&dispatcher_tx), subscribers.clone()).await?;

    run_dispatcher(
        Arc::clone(&dispatcher_tx),
        dispatcher_rx,
        subscribers,
        fedimint_client,
    )
    .await;

    Ok(())
}
