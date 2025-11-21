use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use alloy::providers::{DynProvider, ProviderBuilder};
use tokio::{spawn, try_join};

use crate::{
    blockchain::start_event_listeners,
    database::Database,
    utils::{
        env::{hostname, httprpc, port},
        xnode::undeploy_expired_servers,
    },
};

mod api;
mod blockchain;
mod database;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();

    let database = Database::new().await;
    let provider = ProviderBuilder::new()
        .connect(&httprpc())
        .await
        .unwrap_or_else(|e| panic!("Could not connect to HTTP rpc provider: {e}"));
    let token_counter = api::ownai_v1::OwnAIV1TokenCounter::new(database.clone()).await;

    if let Err(e) = try_join!(
        spawn(start_event_listeners(database.clone())),
        spawn(undeploy_expired_servers(database.clone())),
        // spawn(distribute_staking_rewards(database.clone())),
        // spawn(distribute_manual_tokens(database.clone())),
        spawn(
            HttpServer::new(move || {
                App::new()
                    .wrap(Cors::permissive())
                    .app_data(web::Data::new(database.clone()))
                    .app_data(web::Data::new(DynProvider::new(provider.clone())))
                    .app_data(web::Data::new(token_counter.clone()))
                    .service(web::scope("/api").configure(api::configure))
            })
            .bind(format!(
                "{hostname}:{port}",
                hostname = hostname(),
                port = port()
            ))
            .unwrap_or_else(|e| {
                panic!(
                    "Could not bind http server to {hostname}:{port}: {e}",
                    hostname = hostname(),
                    port = port()
                )
            })
            .run()
        ),
    ) {
        panic!("Main loop error: {e}");
    };
}
