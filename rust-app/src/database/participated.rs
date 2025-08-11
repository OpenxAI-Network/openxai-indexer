use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS participated(tier INT8 NOT NULL, account TEXT NOT NULL, amount INT8 NOT NULL, transaction_hash TEXT NOT NULL, log_index INT8 NOT NULL, PRIMARY KEY (transaction_hash, log_index))"
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create participated table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseParticipated {
    pub tier: i64,
    pub account: String,
    pub amount: i64,
    pub transaction_hash: String,
    pub log_index: i64,
}

impl DatabaseParticipated {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT tier, account, amount, transaction_hash, log_index FROM participated")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT tier, account, amount, transaction_hash, log_index FROM participated WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            tier,
            account,
            amount,
            transaction_hash,
            log_index,
        } = self;

        query("INSERT INTO participated(tier, account, amount, transaction_hash, log_index) VALUES ($1, $2, $3, $4, $5);")
        .bind(tier)
        .bind(account)
        .bind(amount)
        .bind(transaction_hash)
        .bind(log_index)
        .execute(&database.connection)
        .await.err()
    }
}
