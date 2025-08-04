use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS tokens_claimed(account TEXT NOT NULL, total INT8 NOT NULL, released INT8 NOT NULL, transaction_hash TEXT NOT NULL, log_index INT8 NOT NULL, PRIMARY KEY (transaction_hash, log_index))"
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create tokens_claimed table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseTokensClaimed {
    pub account: String,
    pub total: i64,
    pub released: i64,
    pub transaction_hash: String,
    pub log_index: i64,
}

impl DatabaseTokensClaimed {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, total, released, transaction_hash, log_index FROM tokens_claimed")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, total, released, transaction_hash, log_index FROM tokens_claimed WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            account,
            total,
            released,
            transaction_hash,
            log_index,
        } = self;

        query("INSERT INTO tokens_claimed(account, total, released, transaction_hash, log_index) VALUES ($1, $2, $3, $4, $5);")
        .bind(account)
        .bind(total)
        .bind(released)
        .bind(transaction_hash)
        .bind(log_index)
        .execute(&database.connection)
        .await.err()
    }
}
