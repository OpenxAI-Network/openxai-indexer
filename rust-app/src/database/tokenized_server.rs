use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, query, query_as, query_scalar, types::Json};

use crate::database::{Database, DatabaseConnection};

pub async fn create_table(connection: &DatabaseConnection) {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS tokenized_server(collection TEXT NOT NULL, chain TEXT NOT NULL, token_id TEXT NOT NULL, owner TEXT NOT NULL, controller TEXT NOT NULL, deployment JSON, expires INT8 NOT NULL, PRIMARY KEY (collection, chain, token_id))"
    )
    .execute(connection)
    .await
    .unwrap_or_else(|e| panic!("Could not create tokenized_server table: {e}"));
}

pub enum Collection {
    OwnAIv1,
}

impl Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Collection::OwnAIv1 => f.write_str("ownaiv1"),
        }
    }
}

pub enum Chain {
    Base,
}

impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Base => f.write_str("base"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TokenizedServerDeployment {
    Hyperstack { id: u64 },
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct DatabaseTokenizedServer {
    pub collection: String,
    pub chain: String,
    pub token_id: String,
    pub owner: String,
    pub controller: String,
    pub deployment: Option<Json<TokenizedServerDeployment>>,
    pub expires: i64,
}

impl DatabaseTokenizedServer {
    #[allow(dead_code)]
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT collection, chain, token_id, owner, controller, deployment, expires FROM tokenized_server")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_by_owner(database: &Database, owner: &str) -> Result<Vec<Self>, Error> {
        query_as("SELECT collection, chain, token_id, owner, controller, deployment, expires FROM tokenized_server WHERE owner = $1")
            .bind(owner)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_by_controller(
        database: &Database,
        controller: &str,
    ) -> Result<Vec<Self>, Error> {
        query_as("SELECT collection, chain, token_id, owner, controller, deployment, expires FROM tokenized_server WHERE controller = $1")
            .bind(controller)
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_deployed_expired(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT collection, chain, token_id, owner, controller, deployment, expires FROM tokenized_server WHERE deployment IS NOT NULL AND expires < EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_all_not_expired(database: &Database) -> Result<Vec<Self>, Error> {
        query_as("SELECT collection, chain, token_id, owner, controller, deployment, expires FROM tokenized_server WHERE expires > EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)")
            .fetch_all(&database.connection)
            .await
    }

    pub async fn get_by_collection_token_id(
        database: &Database,
        collection: &str,
        chain: &str,
        token_id: &str,
    ) -> Result<Option<Self>, Error> {
        query_as("SELECT collection, chain, token_id, owner, controller, deployment, expires FROM tokenized_server WHERE collection = $1 AND chain = $2 AND token_id = $3")
            .bind(collection)
            .bind(chain)
            .bind(token_id)
            .fetch_optional(&database.connection)
            .await
    }

    pub async fn get_max_token_id_by_collection(
        database: &Database,
        collection: &str,
        chain: &str,
    ) -> Result<Option<i64>, Error> {
        query_scalar(
            "SELECT MAX(token_id::INT8) FROM tokenized_server WHERE collection = $1 AND chain = $2",
        )
        .bind(collection)
        .bind(chain)
        .fetch_one(&database.connection)
        .await
    }

    pub async fn get_not_expired_count_by_collection(
        database: &Database,
        collection: &str,
        chain: &str,
    ) -> Result<i64, Error> {
        query_scalar(
            "SELECT COUNT(*) FROM tokenized_server WHERE collection = $1 AND chain = $2 AND expires > EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)",
        )
        .bind(collection)
        .bind(chain)
        .fetch_one(&database.connection)
        .await
    }

    pub async fn insert(&self, database: &Database) -> Result<(), Error> {
        let Self {
            collection,
            chain,
            token_id,
            owner,
            controller,
            deployment,
            expires,
        } = self;

        query("INSERT INTO tokenized_server(collection, chain, token_id, owner, controller, deployment, expires) VALUES ($1, $2, $3, $4, $5, $6, $7);")
        .bind(collection)
        .bind(chain)
        .bind(token_id)
        .bind(owner)
        .bind(controller)
        .bind(deployment)
        .bind(expires)
        .execute(&database.connection)
        .await?;

        Ok(())
    }

    pub async fn update_owner(&mut self, database: &Database, owner: String) -> Result<(), Error> {
        query("UPDATE tokenized_server SET owner = $1 WHERE collection = $2 AND chain = $3 AND token_id = $4;")
            .bind(&owner)
            .bind(&self.collection)
            .bind(&self.chain)
            .bind(&self.token_id)
            .execute(&database.connection)
            .await?;

        self.owner = owner;
        Ok(())
    }

    pub async fn update_controller(
        &mut self,
        database: &Database,
        controller: String,
    ) -> Result<(), Error> {
        query(
            "UPDATE tokenized_server SET controller = $1 WHERE collection = $2 AND chain = $3 AND token_id = $4;",
        )
        .bind(&controller)
        .bind(&self.collection)
        .bind(&self.chain)
        .bind(&self.token_id)
        .execute(&database.connection)
        .await?;

        self.controller = controller;
        Ok(())
    }

    pub async fn update_expires(&mut self, database: &Database, expires: i64) -> Result<(), Error> {
        query("UPDATE tokenized_server SET expires = $1 WHERE collection = $2 AND chain = $3 AND token_id = $4;")
            .bind(expires)
            .bind(&self.collection)
            .bind(&self.chain)
            .bind(&self.token_id)
            .execute(&database.connection)
            .await?;

        self.expires = expires;
        Ok(())
    }

    pub async fn deploy(
        &mut self,
        database: &Database,
        deployment: Json<TokenizedServerDeployment>,
    ) -> Result<(), Error> {
        query(
            "UPDATE tokenized_server SET deployment = $1 WHERE collection = $2 AND chain = $3 AND token_id = $4;",
        )
        .bind(&deployment)
        .bind(&self.collection)
        .bind(&self.chain)
        .bind(&self.token_id)
        .execute(&database.connection)
        .await?;

        self.deployment = Some(deployment);
        Ok(())
    }

    pub async fn undeploy(&mut self, database: &Database) -> Result<(), Error> {
        query("UPDATE tokenized_server SET deployment = NULL WHERE collection = $1 AND chain = $2 AND token_id = $3;")
            .bind(&self.collection)
            .bind(&self.chain)
            .bind(&self.token_id)
            .execute(&database.connection)
            .await?;

        self.deployment = None;
        Ok(())
    }
}
