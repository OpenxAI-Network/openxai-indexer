use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS manual_tokens(account TEXT NOT NULL, amount INT8 NOT NULL, description TEXT NOT NULL, release_after INT8 NOT NULL, approval_signature TEXT NOT NULL, released BOOLEAN NOT NULL, PRIMARY KEY (account, amount, description, release_after))"
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create manual_tokens table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseManualTokens {
    pub account: String,
    pub amount: i64,
    pub description: String,
    pub release_after: i64,
    pub approval_signature: String,
    pub released: bool,
}

impl DatabaseManualTokens {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT id, account, amount, description, release_after, approval_signature, released FROM manual_tokens")
            .fetch_all(&database.connection)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT id, account, amount, description, release_after, approval_signature, released FROM manual_tokens WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_releasable_not_released(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, amount, description, release_after, approval_signature, released FROM manual_tokens WHERE release_after > EXTRACT(EPOCH FROM CURRENT_TIMESTAMP) AND released = FALSE")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            account,
            amount,
            description,
            release_after,
            approval_signature,
            released,
        } = self;

        query("INSERT INTO manual_tokens(account, amount, description, release_after, approval_signature, released) VALUES ($1, $2, $3, $4, $5, $6);")
        .bind(account)
        .bind(amount)
        .bind(description)
        .bind(release_after)
        .bind(approval_signature)
        .bind(released)
        .execute(&database.connection)
        .await.err()
    }

    pub async fn release(&mut self, database: &Database) -> Option<Error> {
        query(
            "UPDATE manual_tokens SET released = TRUE WHERE account = $1 AND amount = $2 AND description = $3 AND release_after = $4;",
        )
        .bind(true)
        .bind(&self.account)
        .bind(self.amount)
        .bind(&self.description)
        .bind(self.release_after)
        .execute(&database.connection)
        .await
        .err()?;

        self.released = true;
        None
    }
}
