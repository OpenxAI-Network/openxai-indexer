use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::participated);
    cfg.service(handlers::tokens_claimed);
    cfg.service(handlers::claim);
}
