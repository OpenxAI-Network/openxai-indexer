use alloy::signers::Signer;
use xnode_controller::XnodeController;
use xnode_manager_sdk::utils::Session;

use crate::{
    database::{Database, tokenized_server::DatabaseTokenizedServer},
    utils::{time::get_time_u64, wallet::get_tokenized_server_owner, xnode::address_to_xnode_user},
};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    XnodeManagerSDKError(xnode_manager_sdk::utils::Error),
    AlloySignerError(alloy::signers::Error),
}

pub struct ControlledXnode {
    pub database: Database,
    pub collection: String,
    pub chain: String,
    pub token_id: String,
    session: Session,
}

impl ControlledXnode {
    pub async fn new(
        database: Database,
        collection: String,
        chain: String,
        token_id: String,
    ) -> Result<Self, Error> {
        let session = get_session(
            format!("manager.{token_id}.{chain}.{collection}.openxai.network").as_str(),
        )
        .await?;

        Ok(Self {
            database,
            collection,
            chain,
            token_id,
            session,
        })
    }
}

impl XnodeController for ControlledXnode {
    fn get_session(&self) -> &Session {
        &self.session
    }

    async fn check_controller(&self) -> Option<String> {
        DatabaseTokenizedServer::get_by_collection_token_id(
            &self.database,
            &self.collection,
            &self.chain,
            &self.token_id,
        )
        .await
        .ok()
        .flatten()
        .map(|server| server.controller)
    }

    fn controller_config(&self, controller: String) -> String {
        let manager = self.session.base_url.replace("https://", "");
        let app = self.session.base_url.replace("https://manager.", "");
        format!(
            "\
services.xnode-auth.domains.\"{manager}\".accessList.\"{controller}\" = {{ paths = \"^(?:\\/config.*|\\/file\\/container:.*|\\/info.*|\\/process\\/container:.*|\\/usage.*|\\/request.*)\"; }};
services.xnode-auth.domains.\"{app}\".accessList.\"{controller}\" = {{ }};\
"
        )
    }
}

async fn get_session(xnode_id: &str) -> Result<Session, Error> {
    let signer = get_tokenized_server_owner();

    let user = address_to_xnode_user(signer.address());
    let domain = xnode_id.to_string();
    let timestamp = get_time_u64();
    let message = format!("Xnode Auth authenticate {domain} at {timestamp}");
    let signature = signer
        .sign_message(message.as_bytes())
        .await
        .map_err(Error::AlloySignerError)?;

    xnode_manager_sdk::auth::login(xnode_manager_sdk::auth::LoginInput {
        base_url: format!("https://{domain}"),
        user: xnode_manager_sdk::auth::User::with_signature(
            user,
            signature.to_string(),
            timestamp.to_string(),
        ),
    })
    .await
    .map_err(Error::XnodeManagerSDKError)
}
