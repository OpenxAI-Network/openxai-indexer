use sqlx::{Error, Pool, Postgres, postgres::PgPoolOptions, query, query_as};

use crate::{
    database::models::{DatabaseClaimer, DatabaseParticipated, DatabaseTokensClaimed},
    utils::env::database,
};

pub type DatabaseConnection = Pool<Postgres>;

#[derive(Clone)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        Self {
            connection: create_connection().await,
        }
    }
}

pub async fn create_connection() -> DatabaseConnection {
    let pool = PgPoolOptions::new()
        .max_connections(10000)
        .connect(&database())
        .await
        .unwrap_or_else(|e| panic!("Could not establish database connection: {e}"));

    sqlx::raw_sql("CREATE TABLE IF NOT EXISTS participated(tier INTEGER NOT NULL, account TEXT NOT NULL, amount INTEGER NOT NULL, transaction_hash TEXT NOT NULL, transaction_index INTEGER NOT NULL, PRIMARY KEY (transaction_hash, transaction_index))")
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("Could not create participated table: {e}"));

    sqlx::raw_sql("CREATE TABLE IF NOT EXISTS claim(claimer TEXT NOT NULL PRIMARY KEY, total INTEGER NOT NULL)")
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("Could not create claim table: {e}"));

    sqlx::raw_sql("CREATE TABLE IF NOT EXISTS tokens_claimed(account TEXT NOT NULL, total INTEGER NOT NULL, released INTEGER NOT NULL, transaction_hash TEXT NOT NULL, transaction_index INTEGER NOT NULL, PRIMARY KEY (transaction_hash, transaction_index))")
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("Could not create tokens_claimed table: {e}"));

    pool
}

impl DatabaseParticipated {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as(
            "SELECT tier, account, amount, transaction_hash, transaction_index FROM participated",
        )
        .fetch_all(&database.connection)
        .await
    }

    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT tier, account, amount, transaction_hash, transaction_index FROM participated WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            tier,
            account,
            amount,
            transaction_hash,
            transaction_index,
        } = self;

        query("INSERT INTO participated(tier, account, amount, transaction_hash, transaction_index) VALUES ($1, $2, $3, $4, $5);")
        .bind(tier)
        .bind(account)
        .bind(amount)
        .bind(transaction_hash)
        .bind(transaction_index)
        .execute(&database.connection)
        .await.err()
    }
}

impl From<DatabaseParticipated> for DatabaseClaimer {
    fn from(val: DatabaseParticipated) -> Self {
        DatabaseClaimer {
            claimer: val.account,
            total: val.amount * 5,
        }
    }
}

impl DatabaseClaimer {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT claimer, total FROM claim")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_by_account(database: &Database, account: &str) -> Result<Option<Self>, Error> {
        query_as("SELECT claimer, total FROM claim WHERE account = $1")
            .bind(account)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn add(&self, database: &Database) -> Option<Error> {
        let Self { claimer, total } = self;

        query("INSERT INTO claim(claimer, total) VALUES ($1, $2) ON CONFLICT(claimer) DO UPDATE SET total = claim.total + EXCLUDED.total;")
        .bind(claimer)
        .bind(total)
        .execute(&database.connection)
        .await.err()
    }
}

impl DatabaseTokensClaimed {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as(
            "SELECT account, total, released, transaction_hash, transaction_index FROM tokens_claimed",
        )
        .fetch_all(&database.connection)
        .await
    }

    pub async fn get_all_by_account(
        database: &Database,
        account: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT account, total, released, transaction_hash, transaction_index FROM tokens_claimed WHERE account = $1")
            .bind(account)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            account,
            total,
            released,
            transaction_hash,
            transaction_index,
        } = self;

        query("INSERT INTO tokens_claimed(account, total, released, transaction_hash, transaction_index) VALUES ($1, $2, $3, $4, $5);")
        .bind(account)
        .bind(total)
        .bind(released)
        .bind(transaction_hash)
        .bind(transaction_index)
        .execute(&database.connection)
        .await.err()
    }
}
