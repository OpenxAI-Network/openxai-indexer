use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::utils::env::database;

pub mod claim;
pub mod credits;
pub mod participated;
pub mod staking;
pub mod tokenized_server;
pub mod tokens_claimed;

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
    let connection = PgPoolOptions::new()
        .max_connections(10000)
        .connect(&database())
        .await
        .unwrap_or_else(|e| panic!("Could not establish database connection: {e}"));

    claim::create_table(&connection).await;
    credits::create_table(&connection).await;
    participated::create_table(&connection).await;
    staking::create_table(&connection).await;
    tokenized_server::create_table(&connection).await;
    tokens_claimed::create_table(&connection).await;

    connection
}
