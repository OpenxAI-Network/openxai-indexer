use alloy::{providers::Provider, sol};
use futures_util::StreamExt;

use crate::{
    database::{Database, tokens_claimed::DatabaseTokensClaimed},
    utils::{decimals::to_6_decimals, env::claimer},
};

sol! {
    #[sol(rpc)]
    contract OpenxAIClaimer {
        event TokensClaimed(address indexed account, uint256 total, uint256 released);
    }

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
        .unwrap_or_else(|e| panic!("Could not subscribe to tokens claimed event: {e}"))
        .into_stream();

    tokens_claimed_stream
        .for_each(async |event| match event {
            Ok((
                event,
                log,
            )) => {
                let account = event.account.to_string();
                let total: i64 = match to_6_decimals(event.total).try_into() {
                    Ok(total) => total,
                    Err(e) => {
                        log::error!("Total {total} could not be converted into i64: {e}", total = event.total);
                        return;
                    }
                };
                let released: i64 = match to_6_decimals(event.released).try_into() {
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
                let log_index: i64 = match log.log_index {
                    Some(log_index) => {match log_index.try_into() {
                        Ok(log_index) => log_index,
                        Err(e) => {
                            log::error!("Log index {log_index} could not be converted into i64: {e}");
                            return;
                        }
                    }},
                    None => {
                        log::error!("Transaction does not contain log_index");
                        return;
                    }
                };

                log::info!("({transaction_hash}@{log_index}): {account} just claimed {released} tokens (new total {total})");
                let tokens_claimed = DatabaseTokensClaimed {
                    account, total, released, transaction_hash, log_index
                };
                if let Some(e) = tokens_claimed.insert(&database).await
                {
                    log::error!("COULD NOT INSERT TOKENS CLAIMED {tokens_claimed:?} INTO DATABASE: {e}");
                }
            }
            Err(e) => {
                log::warn!("Error polling tokens_claimed event: {e}")
            }
        })
        .await;
}
