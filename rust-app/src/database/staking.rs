use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as, query_scalar};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS staking(id SERIAL PRIMARY KEY, account TEXT NOT NULL, amount INT8 NOT NULL, collection TEXT NOT NULL, chain TEXT NOT NULL, token_id TEXT NOT NULL, date INT8 NOT NULL)",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create staking table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseStaking {
    pub account: String,
    pub amount: i64,
    pub collection: String,
    pub chain: String,
    pub token_id: String,
    pub date: i64,
}

impl DatabaseStaking {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, amount, collection, chain, token_id, date FROM staking")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, amount, collection, chain, token_id, date FROM staking WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_by_collection_token_id(
        database: &Database,
        collection: &str,
        chain: &str,
        token_id: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, amount, collection, chain, token_id, date FROM staking WHERE collection = $1 AND chain = $2 AND token_id = $3")
            .bind(collection)
            .bind(chain)
            .bind(token_id)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_total_amount_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Option<i64>, Error> {
        query_scalar("SELECT SUM(amount)::INT8 FROM staking WHERE account = $1")
            .bind(account)
            .fetch_one(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Result<(), Error> {
        let Self {
            account,
            amount,
            collection,
            chain,
            token_id,
            date,
        } = self;

        query("INSERT INTO staking(account, amount, collection, chain, token_id, date) VALUES ($1, $2, $3, $4, $5, $6);")
            .bind(account)
            .bind(amount)
            .bind(collection)
            .bind(chain)
            .bind(token_id)
            .bind(date)
            .execute(&database.connection)
            .await?;

        Ok(())
    }
}
