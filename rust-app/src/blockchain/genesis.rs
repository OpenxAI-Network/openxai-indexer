use alloy::{providers::Provider, sol};
use futures_util::StreamExt;

use crate::{
    database::{
        Database, claim::DatabaseClaim, credits::DatabaseCredits,
        participated::DatabaseParticipated,
    },
    utils::env::genesis,
};

sol! {
    #[sol(rpc)]
    contract OpenxAIGenesis {
          event Participated(
            uint256 indexed tier,
            address indexed account,
            uint256 amount
        );
    }
}

pub async fn event_listeners<P: Provider>(provider: P, database: Database) {
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

                log::info!("({transaction_hash}@{log_index}): {account} just participated in tier {tier} with {amount}");
                let participated = DatabaseParticipated {
                    account, amount, tier, transaction_hash, log_index
                };
                if let Some(e) = participated.insert(&database).await
                {
                    log::error!("COULD NOT INSERT PARTICIPATED EVENT {participated:?} INTO DATABASE: {e}");
                }

                let claim: DatabaseClaim = (&participated).into();
                if let Some(e) = claim.insert(&database).await
                {
                    log::error!("COULD NOT INSERT CLAIM {claim:?} INTO DATABASE: {e}");
                }

                let credits: DatabaseCredits = (&participated).into();
                if let Some(e) = credits.insert(&database).await
                {
                    log::error!("COULD NOT INSERT CREDITS {credits:?} INTO DATABASE: {e}");
                }
            }
            Err(e) => {
                log::warn!("Error polling participated event: {e}")
            }
        })
        .await;
}
