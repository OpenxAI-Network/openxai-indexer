use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as, query_scalar};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS agreement(id SERIAL PRIMARY KEY, for_account TEXT NOT NULL, title TEXT NOT NULL, description TEXT NOT NULL, created_at INT8 NOT NULL, signed_at INT8, signature TEXT)"
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create agreement table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseAgreement {
    pub id: i32,
    pub for_account: String,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub signed_at: Option<i64>,
    pub signature: Option<String>,
}

impl DatabaseAgreement {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as(
            "SELECT id, for_account, title, description, created_at, signed_at, signature FROM agreement ORDER BY id DESC",
        )
        .fetch_all(&database.connection)
        .await
    }

    #[allow(dead_code)]
    pub async fn get_all_by_for_account(
        database: &Database,
        for_account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT id, for_account, title, description, created_at, signed_at, signature FROM agreement WHERE for_account = $1")
            .bind(for_account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_by_id(database: &Database, id: i32) -> Result<Option<Self>, Error> {
        query_as("SELECT id, for_account, title, description, created_at, signed_at, signature FROM agreement WHERE id = $1")
            .bind(id)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn insert(&mut self, database: &Database) -> Result<(), Error> {
        let id: i32 = query_scalar("INSERT INTO agreement(for_account, title, description, created_at, signed_at, signature) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id")
            .bind(&self.for_account)
            .bind(&self.title)
            .bind(&self.description)
            .bind(self.created_at)
            .bind(self.signed_at)
            .bind(&self.signature)
            .fetch_one(&database.connection)
            .await?;

        self.id = id;

        Ok(())
    }

    pub async fn sign(
        &mut self,
        database: &Database,
        signed_at: i64,
        signature: String,
    ) -> Result<(), Error> {
        let signed_at = Some(signed_at);
        let signature = Some(signature);

        query("UPDATE agreement SET signed_at = $1, signature = $2 WHERE id = $3 AND signature IS NULL;")
            .bind(signed_at)
            .bind(&signature)
            .bind(self.id)
            .execute(&database.connection)
            .await?;

        self.signed_at = signed_at;
        self.signature = signature;

        Ok(())
    }
}
