use actix_web::{HttpResponse, Responder, get, web};

use crate::database::{Database, tokens_claimed::DatabaseTokensClaimed};

#[get("/{account}/tokens_claimed")]
async fn get_tokens_claimed(
    database: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let account = path.into_inner();
    match DatabaseTokensClaimed::get_all_by_account(&database, &account).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            log::error!("Fetching tokens_claimed events for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
