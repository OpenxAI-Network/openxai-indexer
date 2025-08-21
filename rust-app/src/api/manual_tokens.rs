use actix_web::{HttpResponse, Responder, get, post, web};
use alloy::providers::DynProvider;
use serde::{Deserialize, Serialize};

use crate::{
    database::{Database, manual_tokens::DatabaseManualTokens},
    utils::{env::manualtokensigner, signature_validator::validate_signature},
};

#[get("/{account}/manual_tokens")]
async fn get_manual_tokens(
    database: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let account = path.into_inner();
    match DatabaseManualTokens::get_all_by_account(&database, &account).await {
        Ok(tokens) => HttpResponse::Ok().json(tokens),
        Err(e) => {
            log::error!("Fetching manual tokens for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ManualToken {
    pub account: String,
    pub amount: i64,
    pub description: String,
    pub release_after: i64,
}
#[derive(Serialize, Deserialize)]
pub struct ManualTokensSignature {
    pub manual_tokens: String,
    pub signature: String,
}
#[post("/manual_tokens/upload")]
async fn post_upload(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    data: web::Json<ManualTokensSignature>,
) -> impl Responder {
    if !validate_signature(
        provider.get_ref(),
        &manualtokensigner(),
        &data.manual_tokens,
        &data.signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    let manual_tokens: Vec<ManualToken> = match serde_json::from_str(&data.manual_tokens) {
        Ok(manual_tokens) => manual_tokens,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    for token in &manual_tokens {
        let token = DatabaseManualTokens {
            account: token.account.clone(),
            amount: token.amount,
            approval_signature: data.signature.clone(),
            description: token.description.clone(),
            release_after: token.release_after,
            released: false,
        };
        if let Err(e) = token.insert(&database).await {
            log::error!("COULD NOT INSERT MANUAL TOKEN {token:?}: {e}");
        }
    }

    HttpResponse::Ok().finish()
}
