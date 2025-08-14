use std::time::Duration;

use chrono::{NaiveTime, Utc};
use tokio::time::{self, Instant};

use crate::database::{Database, claim::DatabaseClaim, manual_tokens::DatabaseManualTokens};

pub async fn distribute_manual_tokens(database: Database) {
    let utc_now = Utc::now();
    let utc_midnight = (utc_now + chrono::Duration::days(1))
        .with_time(
            NaiveTime::from_hms_opt(0, 0, 0).expect("Invalid manual token distribution time"),
        )
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
        log::info!("Distributing manual token rewards");
        let tokens = match DatabaseManualTokens::get_all_releasable_not_released(&database).await {
            Ok(tokens) => tokens,
            Err(e) => {
                log::error!("COULD NOT GET MANUAL TOKENS: {e}");
                continue;
            }
        };
        for mut token in tokens {
            if let Some(e) = token.release(&database).await {
                log::error!("COULD NOT MARK MANUAL TOKEN {token:?} AS RELEASED INTO DATABASE: {e}");
                return;
            }

            let claim: DatabaseClaim = (&token).into();
            if let Some(e) = claim.insert(&database).await {
                log::error!("COULD NOT INSERT CLAIM {claim:?} INTO DATABASE: {e}");
            }
        }
    }
}
