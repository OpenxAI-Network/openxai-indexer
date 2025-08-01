use actix_web::web::ServiceConfig;

pub mod claim;
pub mod credits;
pub mod participated;
pub mod tokens_claimed;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(participated::get_participated);

    cfg.service(claim::get_claim);
    cfg.service(claim::post_claim);

    cfg.service(tokens_claimed::get_tokens_claimed);

    cfg.service(credits::get_credits);
}
