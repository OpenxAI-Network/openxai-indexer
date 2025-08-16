use alloy::{primitives::Address, providers::Provider, sol};
use futures_util::StreamExt;

use crate::{
    database::{
        Database,
        tokenized_server::{Chain, Collection, DatabaseTokenizedServer},
    },
    utils::{
        env::ownaiv1,
        xnode::{address_to_xnode_user, update_controller},
    },
};

sol! {
    #[sol(rpc)]
    contract OpenxAITokenizedServerV1 {
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);

        function mint(address account, uint256 tokenId) external;
    }
}

pub async fn event_listeners<P: Provider>(provider: P, database: Database) {
    let ownaiv1_addr = ownaiv1().unwrap_or_else(|e| {
        log::error!("Failed to get ownaiv1 address: {}", e);
        panic!("Critical configuration error: ownaiv1 address required");
    });
    let ownaiv1 = OpenxAITokenizedServerV1::new(ownaiv1_addr, provider);
    let transfer_stream = ownaiv1
        .Transfer_filter()
        .subscribe()
        .await
        .unwrap_or_else(|e| panic!("Could not subscribe to tokenized server transfer event: {e}"))
        .into_stream();

    let collection = Collection::OwnAIv1.to_string();
    let chain = Chain::Base.to_string();
    transfer_stream
        .for_each(async |event| match event {
            Ok((
                event,
                _log,
            )) => {
                let from = event.from.to_string();
                let to = event.to.to_string();
                let token_id: i64 = match event.tokenId.try_into() {
                    Ok(token_id) => token_id,
                    Err(e) => {
                        log::error!("Token id {token_id} could not be converted into i64: {e}", token_id = event.tokenId);
                        return;
                    }
                };

                log::info!("{token_id} just got transferred from {from} to {to}");
                if Address::parse_checksummed(from, None).is_ok_and(|address| address == Address::ZERO) {
                    // Freshly minted server, database already up to date
                } else {
                    let mut tokenized_server = match DatabaseTokenizedServer::get_by_collection_token_id(&database, &collection, &chain, &token_id.to_string()).await
                     {
                        Ok(tokenized_server) => match tokenized_server {
                            Some(tokenized_server) => tokenized_server,
                            None => {
                            log::error!("TRANSFER OF NON-EXISTENT TOKENIZED SERVER {collection}@{chain}@{token_id}");
                            return;
                        }
                        },
                        Err(e) => {
                            log::error!("FETCHING TRANSFERRED TOKENIZED SERVER {collection}@{chain}@{token_id}: {e}");
                            return;
                        }
                    };
                    if let Some(e) = tokenized_server.update_owner(&database, to.clone()).await
                    {
                        log::error!("COULD NOT UPDATE TOKENIZED SERVER OWNER {collection}@{chain}@{token_id} to {to}: {e}", collection = tokenized_server.collection, token_id = tokenized_server.token_id);
                        return;
                    };
                    update_controller(&database, &mut tokenized_server, address_to_xnode_user(event.to)).await;
                }
            }
            Err(e) => {
                log::warn!("Error polling tokenized server transfer event: {e}")
            }
        })
        .await;
}
