use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as, query_scalar};

use crate::{
    database::{Database, DatabaseConnection, participated::DatabaseParticipated},
    utils::time::get_time_i64,
};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS claim(id SERIAL PRIMARY KEY, account TEXT NOT NULL, amount INT8 NOT NULL, description TEXT NOT NULL, date INT8 NOT NULL)",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create claim table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseClaim {
    pub account: String,
    pub amount: i64,
    pub description: String,
    pub date: i64,
}

impl DatabaseClaim {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, amount, description, date FROM claim")
            .fetch_all(&database.connection)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_by_account(database: &Database, account: &str) -> Result<Option<Self>, Error> {
        query_as("SELECT account, amount, description, date FROM claim WHERE account = $1")
            .bind(account)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn get_total_amount_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Option<i64>, Error> {
        query_scalar("SELECT SUM(amount) FROM claim WHERE account = $1")
            .bind(account)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            account,
            amount,
            description,
            date,
        } = self;

        query("INSERT INTO claim(account, amount, description, date) VALUES ($1, $2, $3, $4);")
            .bind(account)
            .bind(amount)
            .bind(description)
            .bind(date)
            .execute(&database.connection)
            .await
            .err()
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
            amount: ((val.amount as f64) * multiplier) as i64,
            description: format!(
                "Genesis participation {transaction_hash}@{log_index}",
                transaction_hash = val.transaction_hash,
                log_index = val.log_index
            ),
            date: get_time_i64(),
        }
    }
}
