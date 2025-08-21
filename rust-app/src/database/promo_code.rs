use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS promo_code(code TEXT NOT NULL PRIMARY KEY, credits INT8 NOT NULL, description TEXT NOT NULL, redeemed_by TEXT)"
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create promo_code table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabasePromoCode {
    pub code: String,
    pub credits: i64,
    pub description: String,
    pub redeemed_by: Option<String>,
}

impl DatabasePromoCode {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT code, credits, description, redeemed_by FROM promo_code")
            .fetch_all(&database.connection)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_all_by_redeemed_by(
        database: &Database,
        redeemed_by: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as(
            "SELECT code, credits, description, redeemed_by FROM promo_code WHERE redeemed_by = $1",
        )
        .bind(redeemed_by)
        .fetch_all(&database.connection)
        .await
    }

    pub async fn get_unredeemed_by_code(
        database: &Database,
        code: &str,
    ) -> Result<Option<Self>, Error> {
        query_as("SELECT code, credits, description, redeemed_by FROM promo_code WHERE code = $1 AND redeemed_by IS NULL")
        .bind(code)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Result<(), Error> {
        let Self {
            code,
            credits,
            description,
            redeemed_by,
        } = self;

        query("INSERT INTO promo_code(code, credits, description, redeemed_by) VALUES ($1, $2, $3, $4);")
        .bind(code)
        .bind(credits)
        .bind(description)
        .bind(redeemed_by)
        .execute(&database.connection)
        .await?;

        Ok(())
    }

    pub async fn redeem(&mut self, database: &Database, redeemed_by: &str) -> Result<(), Error> {
        query("UPDATE promo_code SET redeemed_by = $1 WHERE code = $2 AND redeemed_by IS NULL;")
            .bind(redeemed_by)
            .bind(&self.code)
            .execute(&database.connection)
            .await?;

        self.redeemed_by = Some(redeemed_by.to_string());
        Ok(())
    }
}
