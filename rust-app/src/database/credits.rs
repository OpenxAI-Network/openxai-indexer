use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as};

use crate::database::{Database, DatabaseConnection, participated::DatabaseParticipated};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS credits(account TEXT NOT NULL PRIMARY KEY, credits INT8 NOT NULL)",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create credits table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseCredits {
    pub account: String,
    pub credits: i64,
}

impl DatabaseCredits {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, credits FROM credits")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_by_account(database: &Database, account: &str) -> Result<Option<Self>, Error> {
        query_as("SELECT account, credits FROM credits WHERE account = $1")
            .bind(account)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn add(&self, database: &Database) -> Option<Error> {
        let Self { account, credits } = self;

        query("INSERT INTO credits(account, credits) VALUES ($1, $2) ON CONFLICT(account) DO UPDATE SET credits = credits.credits + EXCLUDED.credits;")
        .bind(account)
        .bind(credits)
        .execute(&database.connection)
        .await.err()
    }
}

impl From<&DatabaseParticipated> for DatabaseCredits {
    fn from(val: &DatabaseParticipated) -> Self {
        let multiplier = match val.tier {
            0 => 0.1559,
            1 => 0.1102,
            2 => 0.0900,
            3 => 0.0780,
            4 => 0.0697,
            5 => 0.0637,
            6 => 0.0589,
            7 => 0.0551,
            8 => 0.0520,
            9 => 0.0493,
            10 => 0.0470,
            11 => 0.0450,
            12 => 0.0432,
            13 => 0.0417,
            14 => 0.0403,
            15 => 0.0,
            _ => 0.0,
        };
        DatabaseCredits {
            account: val.account.clone(),
            credits: ((val.amount as f64) * multiplier) as i64,
        }
    }
}
