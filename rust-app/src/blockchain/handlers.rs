use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use futures_util::StreamExt;
use tokio::{spawn, try_join};

use crate::{
    blockchain::models::{OpenxAIClaimer, OpenxAIGenesis},
    database::{
        handlers::Database,
        models::{DatabaseClaimer, DatabaseParticipated, DatabaseTokensClaimed},
    },
    utils::env::{claimer, genesis, rpc},
};

pub async fn start_event_listeners(database: Database) {
    let provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new(rpc()))
        .await
        .unwrap_or_else(|e| panic!("Could not connect to rpc provider: {e}"));

    if let Err(e) = try_join!(
        spawn(claimer_event_listeners(provider.clone(), database.clone())),
        spawn(genesis_event_listeners(provider.clone(), database.clone())),
    ) {
        panic!("Event listener error: {e}");
    }
}

async fn claimer_event_listeners<P: Provider>(provider: P, database: Database) {
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

async fn genesis_event_listeners<P: Provider>(provider: P, database: Database) {
    let genesis = OpenxAIGenesis::new(genesis(), provider);
    let participated_stream = genesis
        .Participated_filter()
        .watch()
        .await
        .unwrap_or_else(|e| panic!("Could not subscribe to Participated event: {e}"))
        .into_stream();

    participated_stream
        .for_each(async |event| match event {
            Ok((
                event,
                log,
            )) => {
                let tier: i64 = match event.tier.try_into() {
                    Ok(tier) => tier,
                    Err(e) => {
                        log::error!("Tier {tier} could not be converted into i64: {e}", tier = event.tier);
                        return;
                    }
                };
                let account = event.account.to_string();
                let amount: i64 = match event.amount.try_into() {
                    Ok(tier) => tier,
                    Err(e) => {
                        log::error!("Amount {amount} could not be converted into i64: {e}", amount = event.amount);
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

                log::info!("({transaction_hash}@{transaction_index}): {account} just participated in tier {tier} with {amount}");
                let participated = DatabaseParticipated {
                    account, amount, tier, transaction_hash, transaction_index
                };
                if let Some(e) = participated.insert(&database).await
                {
                    log::error!("Could not add participated event {tier}, {account}, {amount}, {transaction_hash}, {transaction_index} into database: {e}", tier = participated.tier, account = participated.account, amount = participated.account, transaction_hash = participated.transaction_hash, transaction_index = participated.transaction_index);
                    return;
                }

                let claimer: DatabaseClaimer = participated.into();
                if let Some(e) = claimer.add(&database).await
                {
                    log::error!("Could not add {total} to claimer entry for {claimer} into database: {e}", total = claimer.total, claimer = claimer.claimer);
                }
            }
            Err(e) => {
                log::warn!("Error polling participated event: {e}")
            }
        })
        .await;
}
