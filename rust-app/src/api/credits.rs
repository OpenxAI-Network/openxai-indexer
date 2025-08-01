use actix_web::{HttpResponse, Responder, get, web};

use crate::database::{Database, credits::DatabaseCredits};

#[get("/{account}/credits")]
async fn get_credits(database: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let account = path.into_inner();
    match DatabaseCredits::get_by_account(&database, &account).await {
        Ok(credits) => HttpResponse::Ok().json(credits.map(|credits| credits.credits).unwrap_or(0)),
        Err(e) => {
            log::error!("Fetching credits for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
