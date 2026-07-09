use std::str::FromStr;

use actix_web::{HttpResponse, Responder, get, post, web};
use alloy::{
    primitives::{Address, U256},
    providers::DynProvider,
};
use serde::{Deserialize, Serialize};

use crate::{
    blockchain::nftclaimer::Claim,
    database::{Database, claim::DatabaseClaim, nftclaim::DatabaseNFTClaim},
    utils::{
        env::manualtokensigner, signature_validator::validate_signature,
        wallet::get_nft_claimer_signature,
    },
};

#[get("/nftclaim/{account}")]
async fn get(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    match DatabaseClaim::get_all_by_account(&database, &account).await {
        Ok(claim) => HttpResponse::Ok().json(claim),
        Err(e) => {
            log::error!("Fetching nftclaim for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/nftclaim/{account}/{collection}/{token_id}")]
async fn post(
    database: web::Data<Database>,
    path: web::Path<(String, String, String)>,
) -> impl Responder {
    let (account, collection, token_id) = path.into_inner();

    if !DatabaseNFTClaim::get_by_collection_token_id(&database, &collection, &token_id)
        .await
        .is_ok_and(|nftclaim| nftclaim.is_some_and(|nftclaim| nftclaim.account == account))
    {
        return HttpResponse::BadRequest().finish();
    }

    let claimer = match Address::parse_checksummed(&account, None) {
        Ok(claimer) => claimer,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    let token = match Address::parse_checksummed(&collection, None) {
        Ok(token) => token,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    let tokenId = match U256::from_str(&token_id) {
        Ok(tokenId) => tokenId,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    let claim = Claim {
        claimer,
        token,
        tokenId,
    };
    let signature = match get_nft_claimer_signature(&claim).await {
        Ok(signature) => signature,
        Err(e) => {
            log::error!(
                "Signing nftclaim {collection}@{token_id} for {claimer}: {e}",
                collection = claim.token,
                token_id = claim.tokenId,
                claimer = claim.claimer
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(signature.to_string())
}

#[derive(Serialize, Deserialize)]
pub struct NFTClaim {
    pub collection: String,
    pub token_id: String,
    pub account: String,
    pub description: String,
}
#[derive(Serialize, Deserialize)]
pub struct NFTClaimSignature {
    pub nftclaim: String,
    pub signature: String,
}
#[post("/nftclaim/upload")]
async fn post_upload(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    data: web::Json<NFTClaimSignature>,
) -> impl Responder {
    if !validate_signature(
        provider.get_ref(),
        &manualtokensigner(),
        &data.nftclaim,
        &data.signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    let nftclaim: Vec<NFTClaim> = match serde_json::from_str(&data.nftclaim) {
        Ok(nftclaim) => nftclaim,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    for claim in nftclaim {
        let claim = DatabaseNFTClaim {
            collection: claim.collection,
            token_id: claim.token_id,
            account: claim.account,
            description: claim.description,
            transaction_hash: None,
        };
        if let Err(e) = claim.insert(&database).await {
            log::error!("COULD NOT INSERT NFTCLAIM {claim:?}: {e}");
        }
    }

    HttpResponse::Ok().finish()
}
