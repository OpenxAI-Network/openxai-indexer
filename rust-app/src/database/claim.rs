use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as};

use crate::database::{Database, DatabaseConnection, participated::DatabaseParticipated};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS claim(account TEXT NOT NULL PRIMARY KEY, total INT8 NOT NULL)",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create claim table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseClaim {
    pub account: String,
    pub total: i64,
}

impl DatabaseClaim {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, total FROM claim")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_by_account(database: &Database, account: &str) -> Result<Option<Self>, Error> {
        query_as("SELECT account, total FROM claim WHERE account = $1")
            .bind(account)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn add(&self, database: &Database) -> Option<Error> {
        let Self { account, total } = self;

        query("INSERT INTO claim(account, total) VALUES ($1, $2) ON CONFLICT(account) DO UPDATE SET total = claim.total + EXCLUDED.total;")
        .bind(account)
        .bind(total)
        .execute(&database.connection)
        .await.err()
    }
}

impl From<&DatabaseParticipated> for DatabaseClaim {
    fn from(val: &DatabaseParticipated) -> Self {
        let multiplier = match val.tier {
            0 => 12.5000,
            1 => 12.2699,
            2 => 12.0482,
            3 => 11.8343,
            4 => 11.6279,
            5 => 11.4286,
            6 => 11.2360,
            7 => 11.0497,
            8 => 10.8696,
            9 => 10.6952,
            10 => 10.5263,
            11 => 10.3627,
            12 => 10.2041,
            13 => 10.0503,
            14 => 9.9010,
            15 => 8.3333,
            _ => 0.0,
        };
        DatabaseClaim {
            account: val.account.clone(),
            total: ((val.amount as f64) * multiplier) as i64,
        }
    }
}
