use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseParticipated {
    pub tier: i64,
    pub account: String,
    pub amount: i64,
    pub transaction_hash: String,
    pub transaction_index: i64,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseClaimer {
    pub claimer: String,
    pub total: i64,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseTokensClaimed {
    pub account: String,
    pub total: i64,
    pub released: i64,
    pub transaction_hash: String,
    pub transaction_index: i64,
}
