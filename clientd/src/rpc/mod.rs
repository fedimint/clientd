use crate::rpc::responses::{HandlerResponse, InfoResponse};
use fedimint_client::{Client, UserClientConfig};
use jsonrpsee::types::error::CallError;

mod params;
pub mod responses;

/// Handler for the rpc-method "info"
pub async fn info(
    fedimint_client: &Client<UserClientConfig>,
) -> Result<HandlerResponse, CallError> {
    let coins = fedimint_client.coins();
    Ok(HandlerResponse::Info(InfoResponse::new(coins)))
}
