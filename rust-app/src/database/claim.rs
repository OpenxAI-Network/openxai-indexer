use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as, query_scalar};

use crate::{
    database::{
        Database, DatabaseConnection, manual_tokens::DatabaseManualTokens,
        nft_staking::DatabaseNFTStaking, participated::DatabaseParticipated,
    },
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

    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, amount, description, date FROM claim WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_total_amount_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Option<i64>, Error> {
        query_scalar("SELECT SUM(amount)::INT8 FROM claim WHERE account = $1")
            .bind(account)
            .fetch_one(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Result<(), Error> {
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
            .await?;

        Ok(())
    }
}

impl From<&DatabaseParticipated> for DatabaseClaim {
    fn from(val: &DatabaseParticipated) -> Self {
        let multiplier = match val.tier {
            0 => 10.0000,
            1 => 9.8522,
            2 => 9.7087,
            3 => 9.5694,
            4 => 9.4340,
            5 => 9.3023,
            6 => 9.1743,
            7 => 9.0498,
            8 => 8.9286,
            9 => 8.8106,
            10 => 8.6957,
            11 => 8.5837,
            12 => 8.4746,
            13 => 8.3682,
            14 => 8.2645,
            15 => 8.0000,
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

impl From<&DatabaseNFTStaking> for DatabaseClaim {
    fn from(val: &DatabaseNFTStaking) -> Self {
        DatabaseClaim {
            account: val.account.clone(),
            amount: val.amount,
            description: format!(
                "Staking rewards for {collection}@{chain}@{token_id}",
                collection = val.collection,
                chain = val.chain,
                token_id = val.token_id
            ),
            date: val.date,
        }
    }
}

impl From<&DatabaseManualTokens> for DatabaseClaim {
    fn from(val: &DatabaseManualTokens) -> Self {
        DatabaseClaim {
            account: val.account.clone(),
            amount: val.amount,
            description: format!(
                "Manual token reward for {description}",
                description = val.description
            ),
            date: get_time_i64(),
        }
    }
}
