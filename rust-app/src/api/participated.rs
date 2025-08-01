use actix_web::{HttpResponse, Responder, get, web};

use crate::database::{Database, participated::DatabaseParticipated};

#[get("/{account}/participated")]
async fn get_participated(
    database: web::Data<Database>,
    path: web::Path<String>,
) -> impl Responder {
    let account = path.into_inner();
    match DatabaseParticipated::get_all_by_account(&database, &account).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            log::error!("Fetching participated events for {account}: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
