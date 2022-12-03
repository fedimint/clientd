//! This module contains all datastructures which will be serialized as the result object in the json-rpc response object.
use fedimint_api::{Amount, TieredMulti};
use fedimint_client::mint::SpendableNote;
use serde::Serialize;
use std::collections::BTreeMap;

/// Contains all responses that rpc-calls can produce.
#[derive(Debug, Serialize)]
pub enum HandlerResponse {
    HealthCheck(),
    Info(InfoResponse),
}

#[derive(Debug, Serialize)]
/// Result data of the rpc-method info.
pub struct InfoResponse {
    total_amount: Amount,
    total_num_notes: usize,
    details: BTreeMap<Amount, usize>,
}

impl InfoResponse {
    pub fn new(coins: TieredMulti<SpendableNote>) -> Self {
        let details_vec = coins
            .iter_tiers()
            .map(|(amount, coins)| (amount.to_owned(), coins.len()))
            .collect();
        InfoResponse {
            total_amount: (coins.total_amount()),
            total_num_notes: (coins.item_count()),
            details: (details_vec),
        }
    }
}
