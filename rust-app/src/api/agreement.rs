use actix_web::{HttpResponse, Responder, get, post, web};
use alloy::providers::DynProvider;
use serde::{Deserialize, Serialize};

use crate::{
    database::{Database, agreement::DatabaseAgreement},
    utils::{env::agreementsigner, signature_validator::validate_signature, time::get_time_i64},
};

#[get("/agreement/list")]
async fn list(database: web::Data<Database>) -> impl Responder {
    match DatabaseAgreement::get_all(&database).await {
        Ok(agreements) => HttpResponse::Ok().json(agreements),
        Err(e) => {
            log::error!("Fetching agreements: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/agreement/{id}/info")]
async fn info(database: web::Data<Database>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    match DatabaseAgreement::get_by_id(&database, id).await {
        Ok(agreement) => match agreement {
            Some(agreement) => HttpResponse::Ok().json(agreement),
            None => HttpResponse::NotFound().finish(),
        },
        Err(e) => {
            log::error!("Fetching agreements: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateAgreement {
    pub for_account: String,
    pub title: String,
    pub description: String,
    pub signature: String,
}
#[post("/agreement/create")]
async fn create(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    data: web::Json<CreateAgreement>,
) -> impl Responder {
    let message = format!(
        "Create agreement {agreement} with title {title} for {for_account}",
        agreement = data.description,
        title = data.title,
        for_account = data.for_account
    );
    if !validate_signature(
        provider.get_ref(),
        &agreementsigner(),
        &message,
        &data.signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    let mut agreement = DatabaseAgreement {
        id: 0,
        for_account: data.for_account.clone(),
        title: data.title.clone(),
        description: data.description.clone(),
        created_at: get_time_i64(),
        signed_at: None,
        signature: None,
    };

    if let Err(e) = agreement.insert(&database).await {
        log::error!("COULD NOT INSERT AGREEMENT {agreement:?}: {e}");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(agreement.id)
}

#[derive(Serialize, Deserialize)]
pub struct AgreementSignature {
    pub agreement: i32,
    pub signature: String,
    pub signed_at: i64,
}
#[post("/agreement/sign")]
async fn sign(
    database: web::Data<Database>,
    provider: web::Data<DynProvider>,
    data: web::Json<AgreementSignature>,
) -> impl Responder {
    let mut agreement = match DatabaseAgreement::get_by_id(&database, data.agreement).await {
        Ok(agreement) => match agreement {
            Some(agreement) => agreement,
            None => {
                return HttpResponse::NotFound().finish();
            }
        },
        Err(e) => {
            log::error!("Fetching agreement {id}: {e}", id = data.agreement);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let message = format!(
        "I agree to {agreement} titled {title} at {signed_at}",
        agreement = agreement.description,
        title = agreement.title,
        signed_at = data.signed_at
    );
    if !validate_signature(
        provider.get_ref(),
        &agreement.for_account,
        &message,
        &data.signature,
    )
    .await
    {
        return HttpResponse::Unauthorized().finish();
    }

    if let Err(e) = agreement
        .sign(&database, data.signed_at, data.signature.clone())
        .await
    {
        log::error!(
            "COULD NOT SIGN AGREEMENT {agreement} WITH SIGNATURE {signature} SIGNED AT {signed_at}: {e}",
            agreement = data.agreement,
            signature = data.signature,
            signed_at = data.signed_at
        );
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
