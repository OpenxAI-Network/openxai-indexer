use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as, query_scalar};

use crate::{
    database::{Database, DatabaseConnection, participated::DatabaseParticipated},
    utils::time::get_time_i64,
};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS credits(id SERIAL PRIMARY KEY, account TEXT NOT NULL, credits INT8 NOT NULL, description TEXT NOT NULL, date INT8 NOT NULL)",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create credits table: {e}"));

    sqlx::raw_sql(
        "CREATE OR REPLACE FUNCTION check_sum_credits_before_insert()
RETURNS TRIGGER AS $$
DECLARE
    current_sum INT8;
BEGIN
    SELECT COALESCE(SUM(credits), 0)
    INTO current_sum
    FROM credits
    WHERE account = NEW.account;

    IF current_sum + NEW.credits < 0 THEN
        RAISE EXCEPTION 'Insert would cause SUM(credits) for account \"%\" to be less than 0', NEW.account;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create check_sum_credits_before_insert function: {e}"));

    sqlx::raw_sql(
        "CREATE OR REPLACE TRIGGER trg_check_sum_credits
BEFORE INSERT ON credits
FOR EACH ROW
EXECUTE FUNCTION check_sum_credits_before_insert()",
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create trg_check_sum_credits trigger: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseCredits {
    pub account: String,
    pub credits: i64,
    pub description: String,
    pub date: i64,
}

impl DatabaseCredits {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, credits, description, date FROM credits")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_by_account(database: &Database, account: &str) -> Result<Option<Self>, Error> {
        query_as("SELECT account, credits, description, date FROM credits WHERE account = $1")
            .bind(account)
            .fetch_optional(&database.connection)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_total_credits_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Option<i64>, Error> {
        query_scalar("SELECT SUM(credits) FROM credits WHERE account = $1")
            .bind(account)
            .fetch_one(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            account,
            credits,
            description,
            date,
        } = self;

        query("INSERT INTO credits(account, credits, description, date) VALUES ($1, $2, $3, $4);")
            .bind(account)
            .bind(credits)
            .bind(description)
            .bind(date)
            .execute(&database.connection)
            .await
            .err()
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
            description: format!(
                "Genesis participation {transaction_hash}@{log_index}",
                transaction_hash = val.transaction_hash,
                log_index = val.log_index
            ),
            date: get_time_i64(),
        }
    }
}
