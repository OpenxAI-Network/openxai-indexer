use actix_web::{HttpResponse, Responder, get, post, web};
use alloy::primitives::{Address, U256};

use crate::{
    blockchain::claimer::Claim,
    database::{Database, claim::DatabaseClaim},
    utils::{decimals::to_18_decimals, wallet::get_claimer_signature},
};

#[get("/{account}/claim")]
async fn get_claim(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    match DatabaseClaim::get_all_by_account(&database, &account).await {
        Ok(claim) => HttpResponse::Ok().json(claim),
        Err(e) => {
            log::error!("Fetching claim for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/{account}/claim_total")]
async fn get_claim_total(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    match DatabaseClaim::get_total_amount_by_account(&database, &account).await {
        Ok(total) => HttpResponse::Ok().json(
            total
                .map(|total| to_18_decimals(U256::from(total)))
                .unwrap_or(U256::from(0)),
        ),
        Err(e) => {
            log::error!("Fetching claim total for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/{account}/claim")]
async fn post_claim(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    if account == "0x0FF350487269Fda1aE176620D42c8ab9958493E2"
        || account == "0xE87C55363A51845352a9eD15521ab6AB6AA33Dc4"
        || account == "0x03f0c4A0652B2E02Ab476D647E34F1F4CfbFA724"
        || account == "0xd438C6D4a1450b55284847f06E0ed291fb053238"
    {
        return HttpResponse::BadRequest().finish();
    }
    let claimer = match Address::parse_checksummed(&account, None) {
        Ok(claimer) => claimer,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    let total = match DatabaseClaim::get_total_amount_by_account(&database, &account).await {
        Ok(total) => total
            .map(|total| to_18_decimals(U256::from(total)))
            .unwrap_or(U256::from(0)),
        Err(e) => {
            log::error!("Fetching claim for {account}: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let claim = Claim { claimer, total };
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
