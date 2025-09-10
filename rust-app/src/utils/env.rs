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

pub fn tokenownerkey() -> String {
    env_var("TOKENOWNERKEY").expect("No TOKENOWNERKEY provided.")
}

pub fn tokenminterkey() -> String {
    env_var("TOKENMINTERKEY").expect("No TOKENMINTERKEY provided.")
}

pub fn manualtokensigner() -> String {
    env_var("MANUALTOKENSIGNER").expect("No MANUALTOKENSIGNER provided.")
}

pub fn database() -> String {
    env_var("DATABASE").unwrap_or("postgres:openxai-indexer?host=/run/postgresql".to_string())
}

pub fn subdomaindistributor() -> String {
    env_var("SUBDOMAINDISTRIBUTOR")
        .unwrap_or("http://subdomain-distributor.local:42923".to_string())
}

pub fn httprpc() -> String {
    env_var("HTTPRPC").unwrap_or("https://base-rpc.publicnode.com".to_string())
}

pub fn wsrpc() -> String {
    env_var("WSRPC").unwrap_or("wss://base-rpc.publicnode.com".to_string())
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
    Address::parse_checksummed(
        env_var("CLAIMER").unwrap_or("0x1D2A4145bf920E674D05C26DE57Aad5eAFF3772f".to_string()),
        None,
    )
    .unwrap_or_else(|e| panic!("Invalid CLAIMER provided: {e}"))
}

pub fn genesis() -> Address {
    Address::parse_checksummed(
        env_var("GENESIS").unwrap_or("0x84599c907B42e9bc21F9FE26D9e5A5D3747109D3".to_string()),
        None,
    )
    .unwrap_or_else(|e| panic!("Invalid GENESIS provided: {e}"))
}

pub fn ownaiv1() -> Address {
    Address::parse_checksummed(
        env_var("OWNAIV1").unwrap_or("0x5d3a48B6f16Ba9a830b19B452d8DAA0409e0FE05".to_string()),
        None,
    )
    .unwrap_or_else(|e| panic!("Invalid OWNAIV1 provided: {e}"))
}

pub fn deposit() -> Address {
    Address::parse_checksummed(
        env_var("DEPOSIT").unwrap_or("0x0DA956C8865633AC2E7f02d935EBa495Aae63598".to_string()),
        None,
    )
    .unwrap_or_else(|e| panic!("Invalid DEPOSIT provided: {e}"))
}

pub fn usdc() -> Address {
    Address::parse_checksummed(
        env_var("USDC").unwrap_or("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()),
        None,
    )
    .unwrap_or_else(|e| panic!("Invalid USDC provided: {e}"))
}

pub fn ownaiv1price() -> i64 {
    env_var("OWNAIV1PRICE")
        .and_then(|s| {
            str::parse::<i64>(&s)
                .inspect_err(|e| {
                    log::error!("Could not parse OWNAIV1PRICE to i64: {e}");
                })
                .ok()
        })
        .unwrap_or(100_000_000)
}

pub fn hyperstackapikey() -> String {
    env_var("HYPERSTACKAPIKEY").expect("No HYPERSTACKAPIKEY provided.")
}
