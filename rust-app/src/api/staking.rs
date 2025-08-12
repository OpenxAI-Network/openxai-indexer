use actix_web::{HttpResponse, Responder, get, web};

use crate::database::{Database, staking::DatabaseStaking};

#[get("/{account}/staking")]
async fn get_staking(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    match DatabaseStaking::get_all_by_account(&database, &account).await {
        Ok(staking) => HttpResponse::Ok().json(staking),
        Err(e) => {
            log::error!("Fetching staking for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/{account}/total_staking")]
async fn get_total_staking(
    database: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let account = path.into_inner();
    match DatabaseStaking::get_total_amount_by_account(&database, &account).await {
        Ok(amount) => HttpResponse::Ok().json(amount.unwrap_or(0)),
        Err(e) => {
            log::error!("Fetching total staking for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
