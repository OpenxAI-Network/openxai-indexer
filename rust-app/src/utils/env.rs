use alloy::primitives::Address;

fn env_var(id: &str) -> Option<String> {
    std::env::var(id)
        .inspect_err(|e| {
            log::warn!("Could not read env var {id}: {e}");
        })
        .ok()
}

pub fn hostname() -> String {
    env_var("HOSTNAME").unwrap_or(String::from("0.0.0.0"))
}

pub fn port() -> String {
    env_var("PORT").unwrap_or(String::from("36092"))
}

pub fn claimerkey() -> String {
    env_var("CLAIMERKEY").expect("No CLAIMERKEY provided.")
}

pub fn database() -> String {
    env_var("DATABASE").unwrap_or("postgres:openxai-indexer?host=/run/postgresql".to_string())
}

pub fn rpc() -> String {
    env_var("RPC").unwrap_or("wss://base-rpc.publicnode.com".to_string())
}

pub fn chainid() -> u64 {
    env_var("CHAINID")
        .and_then(|s| {
            str::parse::<u64>(&s)
                .inspect_err(|e| {
                    log::error!("Could not parse CHAINID to u64: {e}");
                })
                .ok()
        })
        .unwrap_or(8453)
}

pub fn claimer() -> Address {
    Address::parse_checksummed(env_var("CLAIMER").expect("No CLAIMER provided."), None)
        .unwrap_or_else(|e| panic!("Invalid CLAIMER provided: {e}"))
}

pub fn genesis() -> Address {
    Address::parse_checksummed(env_var("GENESIS").expect("No GENESIS provided."), None)
        .unwrap_or_else(|e| panic!("Invalid GENESIS provided: {e}"))
}
