use actix_web::{HttpResponse, Responder, post, web};
use alloy::providers::DynProvider;
use serde::{Deserialize, Serialize};

use crate::{
    database::{Database, credits::DatabaseCredits, promo_code::DatabasePromoCode},
    utils::{env::manualtokensigner, signature_validator::validate_signature},
};

#[derive(Serialize, Deserialize)]
pub struct PromoCodeRedeem {
    pub code: String,
    pub account: String,
}
#[post("/promo_code/redeem")]
async fn post_redeem(
    database: web::Data<Database>,
    data: web::Json<PromoCodeRedeem>,
) -> impl Responder {
    let mut code = match DatabasePromoCode::get_unredeemed_by_code(&database, &data.code).await {
        Ok(code) => match code {
            Some(code) => code,
            None => {
                return HttpResponse::BadRequest().finish();
            }
        },
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };
    if let Some(e) = code.redeem(&database, &data.account).await {
        log::error!(
            "COULD NOT REDEEM PROMO CODE {code:?} FOR {account}: {e}",
            account = data.account
        );
        return HttpResponse::InternalServerError().finish();
    }

    let credits: DatabaseCredits = match (&code).try_into() {
        Ok(credits) => credits,
        Err(e) => {
            log::error!("COULD NOT CONVERT PROMO CODE {code:?} INTO CREDITS: {e:?}");
            return HttpResponse::InternalServerError().finish();
        }
    };
    if let Some(e) = credits.insert(&database).await {
        log::error!("COULD NOT INSERT CREDITS {credits:?}: {e}");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[derive(Serialize, Deserialize)]
pub struct PromoCode {
    pub code: String,
    pub credits: i64,
    pub description: String,
}
#[derive(Serialize, Deserialize)]
pub struct PromoCodessSignature {
    pub promo_codes: String,
    pub signature: String,
}
#[post("/promo_code/add")]
async fn post_add(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    data: web::Json<PromoCodessSignature>,
) -> impl Responder {
    if !validate_signature(
        provider.get_ref(),
        &manualtokensigner(),
        &data.promo_codes,
        &data.signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    let promo_codes: Vec<PromoCode> = match serde_json::from_str(&data.promo_codes) {
        Ok(promo_codes) => promo_codes,
        Err(_e) => {
            return HttpResponse::BadRequest().finish();
        }
    };

    for code in &promo_codes {
        let code = DatabasePromoCode {
            code: code.code.clone(),
            credits: code.credits,
            description: code.description.clone(),
            redeemed_by: None,
        };
        if let Some(e) = code.insert(&database).await {
            log::error!("COULD NOT INSERT PROMO CODE {code:?}: {e}");
        }
    }

    HttpResponse::Ok().finish()
}
