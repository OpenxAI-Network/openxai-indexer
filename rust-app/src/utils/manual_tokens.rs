use std::time::Duration;

use chrono::{Timelike, Utc};
use tokio::time::{self, Instant};

use crate::database::{Database, claim::DatabaseClaim, manual_tokens::DatabaseManualTokens};

pub async fn distribute_manual_tokens(database: Database) {
    let utc_now = Utc::now();
    let utc_next_hour = (utc_now + chrono::Duration::hours(1))
        .with_minute(0)
        .expect("Could not set minutes to 0")
        .with_second(0)
        .expect("Could not set seconds to 0");
    let until_next_hour = utc_next_hour
        .signed_duration_since(utc_now)
        .to_std()
        .expect("Unable to convert until_utc_midnight into std duration");

    let mut interval = time::interval_at(
        Instant::now() + until_next_hour,
        Duration::from_secs(60 * 60),
    );

    loop {
        interval.tick().await;
        log::info!("Distributing manual token rewards");
        let tokens = match DatabaseManualTokens::get_all_releasable_not_released(&database).await {
            Ok(tokens) => tokens,
            Err(e) => {
                log::error!("COULD NOT GET MANUAL TOKENS: {e}");
                continue;
            }
        };
        for mut token in tokens {
            if let Err(e) = token.release(&database).await {
                log::error!("COULD NOT MARK MANUAL TOKEN {token:?} AS RELEASED INTO DATABASE: {e}");
                return;
            }

            let claim: DatabaseClaim = (&token).into();
            if let Err(e) = claim.insert(&database).await {
                log::error!("COULD NOT INSERT CLAIM {claim:?} INTO DATABASE: {e}");
            }
        }
    }
}
