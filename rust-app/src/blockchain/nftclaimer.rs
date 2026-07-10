use alloy::{providers::Provider, sol};
use futures_util::StreamExt;

use crate::{
    database::{Database, nftclaim::DatabaseNFTClaim},
    utils::env::{claimer, nftclaimer},
};

sol! {
    #[sol(rpc)]
    contract OpenxAINFTClaimer {
        event NFTClaimed(address indexed token, address indexed account, uint256 indexed tokenId);
    }

    struct Claim {
        address token;
        address claimer;
        uint256 tokenId;
    }
}

pub async fn event_listeners<P: Provider>(provider: P, database: Database) {
    let claimer = OpenxAINFTClaimer::new(nftclaimer(), provider);
    let nft_claimed_stream = claimer
        .NFTClaimed_filter()
        .subscribe()
        .await
        .unwrap_or_else(|e| panic!("Could not subscribe to tokens claimed event: {e}"))
        .into_stream();

    nft_claimed_stream
        .for_each(async |event| match event {
            Ok((event, log)) => {
                let collection = event.token.to_string();
                let token_id = event.tokenId.to_string();
                let account = event.account.to_string();

                let transaction_hash = match log.transaction_hash {
                    Some(transaction_hash) => transaction_hash.to_string(),
                    None => {
                        log::error!("Transaction does not contain transaction_hash");
                        return;
                    }
                };

                log::info!(
                    "({transaction_hash}): {account} just claimed token {token_id} of collection {collection}"
                );
                let mut nftclaim = match DatabaseNFTClaim::get_by_collection_token_id(&database, &collection, &token_id).await {
                    Ok(nftclaim) => match nftclaim {
                        Some(nftclaim) => nftclaim,
                        None => {
                            log::error!("CLAIM OF NON-EXISTENT NFT {collection}@{token_id}");
                            return;
                        }
                    }
                    Err(e) => {
                            log::error!("FETCHING CLAIMED NFT {collection}@{token_id}: {e}");
                        return;
                    }
                };
                if let Err(e) = nftclaim.claimed(&database, transaction_hash).await {
                    log::error!(
                        "COULD NOT MARK NFT AS CLAIMED {nftclaim:?} IN DATABASE: {e}"
                    );
                }
            }
            Err(e) => {
                log::warn!("Error polling nft_claimed event: {e}")
            }
        })
        .await;
}
