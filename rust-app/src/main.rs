use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use tokio::{spawn, try_join};

use crate::{
    blockchain::start_event_listeners,
    database::Database,
    utils::env::{hostname, port},
};

mod api;
mod blockchain;
mod database;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();

    let database = Database::new().await;

    if let Err(e) = try_join!(
        spawn(start_event_listeners(database.clone())),
        spawn(
            HttpServer::new(move || {
                App::new()
                    .wrap(Cors::permissive())
                    .app_data(web::Data::new(database.clone()))
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
