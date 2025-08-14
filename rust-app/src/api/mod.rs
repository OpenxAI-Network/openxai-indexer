use actix_web::web::ServiceConfig;

pub mod claim;
pub mod credits;
pub mod deployment_signature;
pub mod manual_tokens;
pub mod ownai_v1;
pub mod participated;
pub mod staking;
pub mod tokens_claimed;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(claim::get_claim);
    cfg.service(claim::get_claim_total);
    cfg.service(claim::post_claim);

    cfg.service(credits::get_credits);
    cfg.service(credits::get_total_credits);

    cfg.service(deployment_signature::get_latest);
    cfg.service(deployment_signature::get_app_total);
    cfg.service(deployment_signature::get_app_version_total);
    cfg.service(deployment_signature::post_upload);

    cfg.service(manual_tokens::get_manual_tokens);
    cfg.service(manual_tokens::post_upload);

    cfg.service(ownai_v1::get_server);
    cfg.service(ownai_v1::get_owner_servers);
    cfg.service(ownai_v1::get_controller_servers);
    cfg.service(ownai_v1::post_controller);
    cfg.service(ownai_v1::post_expires);
    cfg.service(ownai_v1::post_mint);
    cfg.service(ownai_v1::get_staking);

    cfg.service(participated::get_participated);

    cfg.service(staking::get_staking);
    cfg.service(staking::get_total_staking);

    cfg.service(tokens_claimed::get_tokens_claimed);
}
