use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS nftclaim(collection TEXT NOT NULL, token_id TEXT NOT NULL, account TEXT NOT NULL, description TEXT NOT NULL, transaction_hash TEXT, PRIMARY KEY (collection, token_id))",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create claim table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseNFTClaim {
    pub collection: String,
    pub token_id: String,
    pub account: String,
    pub description: String,
    pub transaction_hash: Option<String>,
}

impl DatabaseNFTClaim {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as(
            "SELECT collection, token_id, account, description, transaction_hash FROM nftclaim",
        )
        .fetch_all(&database.connection)
        .await
    }

    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT collection, token_id, account, description, transaction_hash FROM nftclaim WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_by_collection_token_id(
        database: &Database,
        collection: &str,
        token_id: &str,
    ) -> Result<Option<Self>, Error> {
        query_as("SELECT collection, token_id, account, description, transaction_hash FROM nftclaim WHERE collection = $1 AND token_id = $2")
            .bind(collection)
            .bind(token_id)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Result<(), Error> {
        let Self {
            collection,
            token_id,
            account,
            description,
            transaction_hash,
        } = self;

        query("INSERT INTO nftclaim(collection, token_id, account, description, transaction_hash) VALUES ($1, $2, $3, $4, $5);")
            .bind(collection)
            .bind(token_id)
            .bind(account)
            .bind(description)
            .bind(transaction_hash)
            .execute(&database.connection)
            .await?;

        Ok(())
    }

    pub async fn claimed(
        &mut self,
        database: &Database,
        transaction_hash: String,
    ) -> Result<(), Error> {
        query("UPDATE nftclaim SET transaction_hash = $1 WHERE collection = $2 AND token_id = $3 AND transaction_hash IS NULL;")
            .bind(&transaction_hash)
            .bind(&self.collection)
            .bind(&self.token_id)
            .execute(&database.connection)
            .await?;

        self.transaction_hash = Some(transaction_hash);
        Ok(())
    }
}
