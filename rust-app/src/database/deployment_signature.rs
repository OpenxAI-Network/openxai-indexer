use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as, query_scalar};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS deployment_signature(id SERIAL PRIMARY KEY, xnode TEXT NOT NULL, app TEXT NOT NULL, version TEXT NOT NULL, deployer TEXT, signature TEXT, date INT8 NOT NULL)"
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create deployment_signature table: {e}"));
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseDeploymentSignature {
    pub xnode: String,
    pub app: String,
    pub version: String,
    pub deployer: Option<String>,
    pub signature: Option<String>,
    pub date: i64,
}

impl DatabaseDeploymentSignature {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT xnode, app, version, deployer, signature, date FROM deployment_signature")
            .fetch_all(&database.connection)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_all_by_app(database: &Database, app: &str) -> Result<Vec<Self>, Error> {
        query_as("SELECT xnode, app, version, deployer, signature, date FROM deployment_signature WHERE app = $1")
            .bind(app)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_latest_by_app(
        database: &Database,
        app: &str,
        max: i64,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT xnode, app, version, deployer, signature, date FROM deployment_signature WHERE app = $1 ORDER BY date DESC LIMIT $2")
            .bind(app)
            .bind(max)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_count_by_app(database: &Database, app: &str) -> Result<i64, Error> {
        query_scalar("SELECT COUNT(*) FROM deployment_signature WHERE app = $1")
            .bind(app)
            .fetch_one(&database.connection)
            .await
    }

    pub async fn get_count_by_app_version(
        database: &Database,
        app: &str,
        version: &str,
    ) -> Result<i64, Error> {
        query_scalar("SELECT COUNT(*) FROM deployment_signature WHERE app = $1 AND version = $2")
            .bind(app)
            .bind(version)
            .fetch_one(&database.connection)
            .await
    }

    pub async fn insert(&self, database: &Database) -> Option<Error> {
        let Self {
            xnode,
            app,
            version,
            deployer,
            signature,
            date,
        } = self;

        query("INSERT INTO deployment_signature(xnode, app, version, deployer, signature, date) VALUES ($1, $2, $3, $4, $5, $6);")
        .bind(xnode)
        .bind(app)
        .bind(version)
        .bind(deployer)
        .bind(signature)
        .bind(date)
        .execute(&database.connection)
        .await.err()
    }
}
