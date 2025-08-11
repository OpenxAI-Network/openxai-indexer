use std::time::Duration;

use alloy::primitives::Address;
use serde_json::json;
use sqlx::types::Json;
use tokio::time;
use xnode_controller::XnodeController;
use xnode_deployer::{
    DeployInput, OptionalSupport, XnodeDeployer,
    hyperstack::{HyperstackDeployer, HyperstackHardware, HyperstackOutput},
};

use crate::{
    database::{
        Database,
        tokenized_server::{DatabaseTokenizedServer, TokenizedServerDeployment},
    },
    utils::{
        controller::ControlledXnode,
        env::{hyperstackapikey, subdomaindistributor},
        wallet::get_tokenized_server_owner,
    },
};

pub fn address_to_xnode_user(address: Address) -> String {
    str_to_xnode_user(address.to_string().as_str())
}

pub fn str_to_xnode_user(address: &str) -> String {
    address.replace("0x", "eth:")
}

pub fn get_v1_deployer(name: String) -> HyperstackDeployer {
    HyperstackDeployer::new(
        hyperstackapikey(),
        HyperstackHardware::VirtualMachine {
            name,
            environment_name: "default-NORWAY-1".to_string(),
            flavor_name: "n3-RTX-A4000x1".to_string(),
            key_name: "NixOS".to_string(),
        },
    )
}

pub fn get_deploy_input(domain: String, xnode_owner: String) -> DeployInput {
    DeployInput {
        acme_email: Some("sam@openxai.org".to_string()),
        domain: Some(format!("manager.{domain}")),
        encrypted: None,
        initial_config: Some(format!(
            "services.nginx.serverNamesHashBucketSize = 128; nixpkgs.config.allowUnfree = true; hardware.graphics = {{ enable = true; extraPackages = [ pkgs.nvidia-vaapi-driver ]; }}; hardware.nvidia.open = true; services.xserver.videoDrivers = [ \\\\\\\"nvidia\\\\\\\" ]; services.xnode-reverse-proxy.rules.\\\\\\\"{domain}\\\\\\\" = [ {{ forward = \\\\\\\"http://xnode-ai-chat.container:8080\\\\\\\"; }} ];"
        )),
        user_passwd: None,
        xnode_owner: Some(xnode_owner),
    }
}

pub async fn available_v1() -> bool {
    let client = reqwest::Client::new();
    let target_region = "NORWAY-1".to_string();
    let target_model = "RTX-A4000".to_string();

    let response = match client
        .get("https://infrahub-api.nexgencloud.com/v1/core/stocks")
        .header(
            "api_key",
            "7eeb6090-7fe6-48c5-ab6e-7d7d88b529fd".to_string(),
        ) //hyperstackapikey())
        .send()
        .await
        .and_then(|response| response.error_for_status())
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(response) => response,
            Err(_e) => {
                return false;
            }
        },
        Err(_e) => {
            return false;
        }
    };

    if let serde_json::Value::Object(map) = &response {
        if let Some(serde_json::Value::Array(stocks)) = map.get("stocks") {
            if let Some(serde_json::Value::Object(stock)) = stocks.iter().find(|stock| {
                if let serde_json::Value::Object(stock) = stock {
                    stock.get("region").is_some_and(|region| {
                        if let serde_json::Value::String(region) = region {
                            region == &target_region
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            }) {
                if let Some(serde_json::Value::Array(models)) = stock.get("models") {
                    if let Some(serde_json::Value::Object(model)) = models.iter().find(|model| {
                        if let serde_json::Value::Object(model) = model {
                            model.get("model").is_some_and(|model| {
                                if let serde_json::Value::String(model) = model {
                                    model == &target_model
                                } else {
                                    false
                                }
                            })
                        } else {
                            false
                        }
                    }) {
                        if let Some(serde_json::Value::Object(configurations)) =
                            model.get("configurations")
                        {
                            if let Some(serde_json::Value::Number(available)) =
                                configurations.get("1x")
                            {
                                if let Some(available) = available.as_u64() {
                                    return available > 0;
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    false
}

pub async fn deploy_v1(database: &Database, server: &mut DatabaseTokenizedServer) {
    let subdomain = format!(
        "{token_id}.{chain}.{collection}",
        token_id = server.token_id,
        chain = server.chain,
        collection = server.collection,
    );
    let domain = format!("{subdomain}.openxai.network");
    let deployer = get_v1_deployer(subdomain.replace(".", "-"));
    let deployment = match deployer
        .deploy(get_deploy_input(
            domain.clone(),
            address_to_xnode_user(get_tokenized_server_owner().address()),
        ))
        .await
    {
        Ok(deployment) => deployment,
        Err(e) => {
            log::error!(
                "DEPLOYMENT OF {collection}@{chain}@{token_id} FAILED: {e:?}",
                collection = server.collection,
                chain = server.chain,
                token_id = server.token_id
            );
            return;
        }
    };
    if let Some(e) = server
        .deploy(
            database,
            Json(TokenizedServerDeployment::Hyperstack { id: deployment.id }),
        )
        .await
    {
        log::error!(
            "DATABASE UPDATE OF DEPLOYMENT {id} FOR {collection}@{chain}@{token_id} FAILED: {e}",
            id = deployment.id,
            collection = server.collection,
            chain = server.chain,
            token_id = server.token_id
        );
    };

    let mut interval = time::interval(Duration::from_secs(1)); // 1 second
    loop {
        interval.tick().await;
        if let Ok(OptionalSupport::Supported(Some(ip))) = deployer.ipv4(&deployment).await {
            let ipv4 = ip.to_string();
            let client = reqwest::Client::new();
            if let Err(e) = client
                .post(format!(
                    "{subdomaindistributor}/{subdomain}/reserve",
                    subdomaindistributor = subdomaindistributor()
                ))
                .json(&json!({
                    "user": "",
                    "ipv4": ipv4
                }))
                .send()
                .await
                .and_then(|response| response.error_for_status())
            {
                log::error!("SUBDOMAIN RESERVATION FOR {subdomain} -> {ipv4} FAILED: {e}");
            } else {
                log::info!("Subdomain {subdomain} reserved for {ipv4}");
            }
            break;
        }
    }
}

pub async fn undeploy(database: &Database, server: &mut DatabaseTokenizedServer) {
    let deployment = match server.deployment.clone() {
        Some(deployment) => deployment.0,
        None => {
            log::warn!(
                "Attempted undeployment of {collection}@{chain}@{token_id}, but no deployment in database",
                collection = server.collection,
                chain = server.chain,
                token_id = server.token_id
            );
            return;
        }
    };

    match deployment {
        TokenizedServerDeployment::Hyperstack { id } => {
            let deployer = get_v1_deployer(format!("ownaiv1@{}", server.token_id));
            if let Some(e) = deployer.undeploy(HyperstackOutput { id }).await {
                log::error!(
                    "UNDEPLOYMENT OF {collection}@{chain}@{token_id} FAILED: {e:?}",
                    collection = server.collection,
                    chain = server.chain,
                    token_id = server.token_id
                );
                return;
            };
        }
    }

    if let Some(e) = server.undeploy(database).await {
        log::error!(
            "DATABASE UPDATE OF UNDEPLOYMENT {deployment:?} FOR {collection}@{chain}@{token_id} FAILED: {e}",
            collection = server.collection,
            chain = server.chain,
            token_id = server.token_id
        );
    };
}

pub async fn undeploy_expired_servers(database: Database) {
    let mut interval = time::interval(Duration::from_secs(60)); // 1 minute

    loop {
        interval.tick().await;
        let expired_servers =
            match DatabaseTokenizedServer::get_all_deployed_expired(&database).await {
                Ok(expired_servers) => expired_servers,
                Err(e) => {
                    log::warn!("Could not get expired servers: {e}");
                    continue;
                }
            };
        for mut server in expired_servers {
            undeploy(&database, &mut server).await;
        }
    }
}

pub async fn update_controller(
    database: &Database,
    server: &mut DatabaseTokenizedServer,
    controller: String,
) {
    let xnode = match ControlledXnode::new(
        database.clone(),
        server.collection.clone(),
        server.chain.clone(),
        server.token_id.clone(),
    )
    .await
    {
        Ok(xnode) => xnode,
        Err(e) => {
            log::error!(
                "CREATE XNODE SESSION FOR {collection}@{chain}@{token_id} FAILED: {e:?}",
                collection = server.collection,
                chain = server.chain,
                token_id = server.token_id
            );
            return;
        }
    };

    if let Err(e) = xnode.set_controller(Some(controller.clone())).await {
        log::error!(
            "XNODE MANAGER UPDATE OF CONTROLLER {controller} FOR {collection}@{chain}@{token_id} FAILED: {e:?}",
            collection = server.collection,
            chain = server.chain,
            token_id = server.token_id
        );
    }

    if let Some(e) = server.update_controller(database, controller.clone()).await {
        log::error!(
            "DATABASE UPDATE OF CONTROLLER {controller} FOR {collection}@{chain}@{token_id} FAILED: {e}",
            collection = server.collection,
            chain = server.chain,
            token_id = server.token_id
        );
    }
}
