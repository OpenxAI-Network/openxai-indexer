use alloy::{providers::Provider, sol};
use futures_util::StreamExt;
use serde::Serialize;

use crate::{
    database::{Database, tokens_claimed::DatabaseTokensClaimed},
    utils::env::claimer,
};

sol! {
    #[sol(rpc)]
    contract OpenxAIClaimer {
        event TokensClaimed(address indexed account, uint256 total, uint256 released);
    }

    #[derive(Serialize)]
    struct Claim {
        address claimer;
        uint256 total;
    }
}

pub async fn event_listeners<P: Provider>(provider: P, database: Database) {
    let claimer = OpenxAIClaimer::new(claimer(), provider);
    let tokens_claimed_stream = claimer
        .TokensClaimed_filter()
        .watch()
        .await
        .unwrap_or_else(|e| panic!("Could not subscribe to Participated event: {e}"))
        .into_stream();

    tokens_claimed_stream
        .for_each(async |event| match event {
            Ok((
                event,
                log,
            )) => {
                let account = event.account.to_string();
                let total: i64 = match event.total.try_into() {
                    Ok(total) => total,
                    Err(e) => {
                        log::error!("Total {total} could not be converted into i64: {e}", total = event.total);
                        return;
                    }
                };
                let released: i64 = match event.released.try_into() {
                    Ok(tier) => tier,
                    Err(e) => {
                        log::error!("Released {released} could not be converted into i64: {e}", released = event.released);
                        return;
                    }
                };
                let transaction_hash = match log.transaction_hash {
                    Some(transaction_hash) => transaction_hash.to_string(),
                    None => {
                        log::error!("Transaction does not contain transaction_hash");
                        return;
                    }
                };
                let transaction_index: i64 = match log.transaction_index {
                    Some(transaction_index) => {match transaction_index.try_into() {
                        Ok(transaction_index) => transaction_index,
                        Err(e) => {
                            log::error!("Transaction index {transaction_index} could not be converted into i64: {e}");
                            return;
                        }
                    }},
                    None => {
                        log::error!("Transaction does not contain transaction_index");
                        return;
                    }
                };

                log::info!("({transaction_hash}@{transaction_index}): {account} just claimed {released} tokens (new total {total})");
                let tokens_claimed = DatabaseTokensClaimed {
                    account, total, released, transaction_hash, transaction_index
                };
                if let Some(e) = tokens_claimed.insert(&database).await
                {
                    log::error!("Could not add tokens_claimed event {account}, {total}, {released}, {transaction_hash}, {transaction_index} into database: {e}", account = tokens_claimed.account, total = tokens_claimed.total, released = tokens_claimed.released, transaction_hash = tokens_claimed.transaction_hash, transaction_index = tokens_claimed.transaction_index);
                }
            }
            Err(e) => {
                log::warn!("Error polling tokens_claimed event: {e}")
            }
        })
        .await;
}
