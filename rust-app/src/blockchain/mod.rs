use alloy::providers::{ProviderBuilder, WsConnect};
use tokio::{spawn, try_join};

use crate::{database::Database, utils::env::wsrpc};

pub mod claimer;
pub mod credits;
pub mod genesis;
pub mod ownai_v1;

pub async fn start_event_listeners(database: Database) {
    let provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new(wsrpc()))
        .await
        .unwrap_or_else(|e| panic!("Could not connect to WS rpc provider: {e}"));

    if let Err(e) = try_join!(
        spawn(claimer::event_listeners(provider.clone(), database.clone())),
        spawn(credits::event_listeners(provider.clone(), database.clone())),
        // spawn(genesis::event_listeners(provider.clone(), database.clone())),
        spawn(ownai_v1::event_listeners(
            provider.clone(),
            database.clone()
        )),
    ) {
        panic!("Event listener error: {e}");
    }
}
