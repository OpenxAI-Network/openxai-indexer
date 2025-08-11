use alloy::{providers::Provider, sol};
use futures_util::StreamExt;

use crate::{
    database::{Database, credits::DatabaseCredits, tokenized_server::Chain},
    utils::{
        env::{deposit, usdc},
        time::get_time_i64,
    },
};

sol! {
    #[sol(rpc)]
    contract USDC {
        event Transfer(address from, address to, uint256 value);
    }
}

pub async fn event_listeners<P: Provider>(provider: P, database: Database) {
    let usdc = USDC::new(usdc(), provider);
    let deposit = deposit();
    let transfer_stream = usdc
        .Transfer_filter()
        .topic2(deposit)
        .watch()
        .await
        .unwrap_or_else(|e| panic!("Could not subscribe to USDC transfer event: {e}"))
        .into_stream();

    let chain = Chain::Base.to_string();
    transfer_stream
        .for_each(async |event| match event {
            Ok((
                event,
                log,
            )) => {
                if event.to != deposit {
                    log::warn!("USDC transfer to non-deposit address received.");
                    return;
                }

                let account = event.from.to_string();
                let amount: i64 = match event.value.try_into() {
                    Ok(amount) => amount,
                    Err(e) => {
                        log::error!("Amount {value} could not be converted into i64: {e}", value = event.value);
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

                log::info!("({transaction_hash}@{log_index}): {account} just deposited {amount} USDC for credits");
                let credits: DatabaseCredits = DatabaseCredits {
                    account,
                    credits: amount,
                    description: format!("USDC deposit on {chain}"),
                    date: get_time_i64()
                };
                if let Some(e) = credits.insert(&database).await
                {
                    log::error!("COULD NOT INSERTS CREDITS {credits:?} INTO DATABASE: {e}");
                }
            }
            Err(e) => {
                log::warn!("Error polling USDC transfer event: {e}")
            }
        })
        .await;
}
