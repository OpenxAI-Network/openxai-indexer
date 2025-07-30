use actix_web::{HttpResponse, Responder, get, post, web};
use alloy::primitives::{Address, U256};

use crate::{
    blockchain::models::Claim,
    database::{
        handlers::Database,
        models::{DatabaseClaimer, DatabaseParticipated, DatabaseTokensClaimed},
    },
    utils::wallet::get_claimer_signature,
};

#[get("/{account}/participated")]
async fn participated(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    match DatabaseParticipated::get_all_by_account(&database, &account).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            log::error!("Fetching participated events for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/{account}/tokens_claimed")]
async fn tokens_claimed(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    match DatabaseTokensClaimed::get_all_by_account(&database, &account).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            log::error!("Fetching tokens_claimed events for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/{account}/claim")]
async fn claim(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    let claimer = match Address::parse_checksummed(&account, None) {
        Ok(claimer) => claimer,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    let total = match DatabaseClaimer::get_by_account(&database, &account).await {
        Ok(claimer) => claimer.map(|claimer| claimer.total).unwrap_or(0),
        Err(e) => {
            log::error!("Retrieving database claimer for {account}: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let claim = Claim {
        claimer,
        total: U256::from(total),
    };
    let signature = match get_claimer_signature(&claim).await {
        Ok(signature) => signature,
        Err(e) => {
            log::error!(
                "Signing claim of {total} for {claimer}: {e}",
                total = claim.total,
                claimer = claim.claimer
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(signature.to_string())
}
