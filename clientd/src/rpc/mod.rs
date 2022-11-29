use crate::rpc::responses::InfoResponse;
use crate::RpcError;
use fedimint_client::{Client, UserClientConfig};

mod params;
mod responses;

pub async fn info(
    fedimint_client: &mut Client<UserClientConfig>,
) -> Result<InfoResponse, RpcError> {
    let coins = fedimint_client.coins();
    Ok(InfoResponse::new(coins))
}
