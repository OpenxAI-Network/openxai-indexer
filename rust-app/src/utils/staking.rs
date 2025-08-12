use std::time::Duration;

use chrono::{NaiveTime, Utc};
use tokio::time::{self, Instant};

use crate::{
    database::{
        Database, claim::DatabaseClaim, staking::DatabaseStaking,
        tokenized_server::DatabaseTokenizedServer,
    },
    utils::time::get_time_i64,
};

pub async fn distribute_staking_rewards(database: Database) {
    let utc_now = Utc::now();
    let utc_midnight = (utc_now + chrono::Duration::days(1))
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).expect("Invalid staking time"))
        .unwrap();
    let until_utc_midnight = utc_midnight
        .signed_duration_since(utc_now)
        .to_std()
        .expect("Unable to convert until_utc_midnight into std duration");

    let mut interval = time::interval_at(
        Instant::now() + until_utc_midnight,
        Duration::from_secs(24 * 60 * 60),
    );

    loop {
        interval.tick().await;
        log::info!("Distributing staking rewards");
        let servers = match DatabaseTokenizedServer::get_all_not_expired(&database).await {
            Ok(servers) => servers,
            Err(e) => {
                log::error!("COULD NOT GET STAKING REWARD ELIGIBLE SERVERS: {e}");
                continue;
            }
        };
        for server in servers {
            let staking_reward = DatabaseStaking {
                account: server.owner.clone(),
                amount: calculate_staking_reward(&server),
                collection: server.collection.clone(),
                chain: server.chain.clone(),
                token_id: server.token_id.clone(),
                date: get_time_i64(),
            };

            if let Some(e) = staking_reward.insert(&database).await {
                log::error!("COULD NOT INSERT STAKING REWARD {staking_reward:?}: {e}");
            }

            let claim: DatabaseClaim = (&staking_reward).into();
            if let Some(e) = claim.insert(&database).await {
                log::error!("COULD NOT INSERT CLAIM {claim:?} INTO DATABASE: {e}");
            }
        }
    }
}

fn calculate_staking_reward(_server: &DatabaseTokenizedServer) -> i64 {
    0
}
