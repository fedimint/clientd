use fedimint_api::{Amount, TieredMulti};
use fedimint_client::mint::SpendableNote;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub enum HandlerResponse {
    HealthCheck(),
    Info(InfoResponse),
}

#[derive(Debug, Serialize)]
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
