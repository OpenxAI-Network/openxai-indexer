use actix_web::web::ServiceConfig;

pub mod claim;
pub mod credits;
pub mod ownai_v1;
pub mod participated;
pub mod tokens_claimed;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(participated::get_participated);

    cfg.service(claim::get_claim);
    cfg.service(claim::post_claim);

    cfg.service(tokens_claimed::get_tokens_claimed);

    cfg.service(credits::get_credits);
    cfg.service(credits::get_total_credits);

    cfg.service(ownai_v1::get_owner_tokens);
    cfg.service(ownai_v1::get_controller_tokens);
    cfg.service(ownai_v1::post_controller);
    cfg.service(ownai_v1::post_expires);
    cfg.service(ownai_v1::post_mint);
}
