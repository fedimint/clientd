use crate::rpc::responses::{HandlerResponse, InfoResponse};
use fedimint_client::{Client, UserClientConfig};
use jsonrpsee::types::error::CallError;

mod params;
pub mod responses;

pub async fn info(
    fedimint_client: &Client<UserClientConfig>,
) -> Result<HandlerResponse, CallError> {
    let coins = fedimint_client.coins();
    Ok(HandlerResponse::Info(InfoResponse::new(coins)))
}
