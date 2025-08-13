use std::sync::{
    Arc,
    atomic::{AtomicI64, Ordering},
};

use actix_web::{HttpResponse, Responder, get, post, web};
use alloy::{primitives::Address, providers::DynProvider};
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        Database,
        credits::DatabaseCredits,
        staking::DatabaseStaking,
        tokenized_server::{Chain, Collection, DatabaseTokenizedServer},
    },
    utils::{
        env::ownaiv1price,
        signature_validator::validate_signature,
        time::get_time_i64,
        wallet::mint_tokenized_server,
        xnode::{available_v1, deploy_v1, str_to_xnode_user, update_controller},
    },
};

#[derive(Serialize, Deserialize)]
pub struct PublicServer {
    pub owner: String,
    pub expires: i64,
}
#[get("/ownaiv1/{chain}/{token_id}/server")]
async fn get_server(
    database: web::Data<Database>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (chain, token_id) = path.into_inner();
    let collection = Collection::OwnAIv1.to_string();

    match DatabaseTokenizedServer::get_by_collection_token_id(
        &database,
        &collection,
        &chain,
        &token_id,
    )
    .await
    {
        Ok(server) => match server {
            Some(server) => HttpResponse::Ok().json(PublicServer {
                owner: server.owner,
                expires: server.expires,
            }),
            None => HttpResponse::BadRequest().finish(),
        },
        Err(_e) => HttpResponse::BadRequest().finish(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct OwnerServer {
    pub chain: String,
    pub token_id: String,
    pub controller: String,
    pub expires: i64,
}
#[get("/ownaiv1/{owner}/owner_servers")]
async fn get_owner_servers(
    database: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let owner = path.into_inner();
    match DatabaseTokenizedServer::get_all_by_owner(&database, &owner).await {
        Ok(servers) => HttpResponse::Ok().json(
            servers
                .into_iter()
                .map(|server| OwnerServer {
                    chain: server.chain,
                    token_id: server.token_id,
                    controller: server.controller,
                    expires: server.expires,
                })
                .collect::<Vec<OwnerServer>>(),
        ),
        Err(e) => {
            log::error!("Fetching tokenized servers for owner {owner}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ControllerServer {
    pub chain: String,
    pub token_id: String,
}
#[get("/ownaiv1/{controller}/controller_servers")]
async fn get_controller_servers(
    database: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let controller = path.into_inner();
    match DatabaseTokenizedServer::get_all_by_controller(&database, &controller).await {
        Ok(servers) => HttpResponse::Ok().json(
            servers
                .into_iter()
                .map(|server| ControllerServer {
                    chain: server.chain,
                    token_id: server.token_id,
                })
                .collect::<Vec<ControllerServer>>(),
        ),
        Err(e) => {
            log::error!("Fetching tokenized servers for controller {controller}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ControllerUpdate {
    pub controller: String,
    pub owner_signature: String,
}
#[post("/ownaiv1/{chain}/{token_id}/controller")]
async fn post_controller(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    path: web::Path<(String, String)>,
    data: web::Json<ControllerUpdate>,
) -> impl Responder {
    let (chain, token_id) = path.into_inner();
    let collection = Collection::OwnAIv1.to_string();

    let mut server = match DatabaseTokenizedServer::get_by_collection_token_id(
        &database,
        &collection,
        &chain,
        &token_id,
    )
    .await
    {
        Ok(server) => match server {
            Some(server) => server,
            None => {
                return HttpResponse::BadRequest().finish();
            }
        },
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    let message = format!(
        "Update controller for {collection}@{chain}@{token_id} to {controller}",
        controller = data.controller
    );
    if !validate_signature(
        provider.get_ref(),
        &server.owner,
        &message,
        &data.owner_signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    update_controller(&database, &mut server, data.controller.clone()).await;

    HttpResponse::Ok().finish()
}

#[derive(Serialize, Deserialize)]
pub struct ExpiresExtend {
    pub months: i64,
    pub payer_address: String,
    pub payer_signature: String,
}
#[post("/ownaiv1/{chain}/{token_id}/expires")]
async fn post_expires(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    path: web::Path<(String, String)>,
    data: web::Json<ExpiresExtend>,
) -> impl Responder {
    let (chain, token_id) = path.into_inner();
    let collection = Collection::OwnAIv1.to_string();

    let mut server = match DatabaseTokenizedServer::get_by_collection_token_id(
        &database,
        &collection,
        &chain,
        &token_id,
    )
    .await
    {
        Ok(server) => match server {
            Some(server) => server,
            None => {
                return HttpResponse::BadRequest().finish();
            }
        },
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    if server.deployment.is_none() {
        // Redeployment of expired servers not possible
        return HttpResponse::BadRequest().finish();
    }

    let message = format!(
        "Extend expiry of {collection}@{chain}@{token_id} by {months} months",
        months = data.months
    );
    if !validate_signature(
        provider.get_ref(),
        &data.payer_address,
        &message,
        &data.payer_signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    if let Some(_e) = (DatabaseCredits {
        account: data.payer_address.clone(),
        credits: -ownaiv1price() * data.months,
        description: format!(
            "Extend expiry of {collection}@{chain}@{token_id} by {months} months",
            months = data.months
        ),
        date: get_time_i64(),
    })
    .insert(&database)
    .await
    {
        return HttpResponse::PaymentRequired().finish();
    }

    let one_month = 30 * 24 * 60 * 60; // 1 month in seconds
    if let Some(e) = server
        .update_expires(&database, server.expires + data.months * one_month)
        .await
    {
        log::error!(
            "COULD NOT EXTEND TOKENIZED SERVER EXPIRES {collection}@{chain}@{token_id} BY {months} MONTHS: {e}",
            months = data.months
        );
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[derive(Clone)]
pub struct OwnAIV1TokenCounter {
    counter: Arc<AtomicI64>,
}
impl OwnAIV1TokenCounter {
    pub async fn new(database: Database) -> Self {
        let collection = Collection::OwnAIv1.to_string();
        let chain = Chain::Base.to_string();
        let max = match DatabaseTokenizedServer::get_max_token_id_by_collection(
            &database,
            &collection,
            &chain,
        )
        .await
        {
            Ok(max) => max.unwrap_or(0),
            Err(e) => {
                panic!("Could not fetch max token_id for {collection} minting: {e}")
            }
        };

        Self {
            counter: Arc::new(AtomicI64::from(max + 1)),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mint {
    pub to: String,
    pub payer_address: String,
    pub payer_signature: String,
}
#[post("/ownaiv1/{chain}/mint")]
async fn post_mint(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    counter: web::Data<OwnAIV1TokenCounter>,
    path: web::Path<String>,
    data: web::Json<Mint>,
) -> impl Responder {
    let to = match Address::parse_checksummed(&data.to, None) {
        Ok(account) => account,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    if !available_v1().await {
        return HttpResponse::FailedDependency().finish();
    }

    let chain = path.into_inner();
    if chain != Chain::Base.to_string() {
        return HttpResponse::BadRequest().finish();
    }
    let collection = Collection::OwnAIv1.to_string();

    let message = format!("Mint new {collection}@{chain} to {to}", to = data.to);
    if !validate_signature(
        provider.get_ref(),
        &data.payer_address,
        &message,
        &data.payer_signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    if let Some(_e) = (DatabaseCredits {
        account: data.payer_address.clone(),
        credits: -ownaiv1price(),
        description: format!("Mint of {collection}@{chain} to {to}", to = data.to),
        date: get_time_i64(),
    })
    .insert(&database)
    .await
    {
        return HttpResponse::PaymentRequired().finish();
    }

    let token_id = counter.counter.fetch_add(1, Ordering::Relaxed);
    let one_month = 30 * 24 * 60 * 60; // 1 month in seconds
    let mut server = DatabaseTokenizedServer {
        collection,
        chain,
        token_id: token_id.to_string(),
        owner: data.to.clone(),
        controller: str_to_xnode_user(&data.to),
        deployment: None,
        expires: get_time_i64() + one_month,
    };
    if let Some(e) = server.insert(&database).await {
        log::error!("COULD NOT INSERT TOKENIZED SERVER {server:?}: {e}");
        return HttpResponse::InternalServerError().finish();
    }
    mint_tokenized_server(provider.get_ref(), to, token_id).await;
    deploy_v1(&database, &mut server).await;

    HttpResponse::Ok().json(token_id)
}

#[get("/ownaiv1/{chain}/{token_id}/staking")]
async fn get_staking(
    database: web::Data<Database>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (chain, token_id) = path.into_inner();
    let collection = Collection::OwnAIv1.to_string();

    match DatabaseStaking::get_all_by_collection_token_id(&database, &collection, &chain, &token_id)
        .await
    {
        Ok(staking) => HttpResponse::Ok().json(staking),
        Err(e) => {
            log::error!("Fetching staking for {collection}@{chain}@{token_id}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
