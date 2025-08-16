use alloy::primitives::Address;
use std::fmt;
use secrecy::{SecretString, ExposeSecret};

#[derive(Debug)]
pub enum EnvError {
    MissingVariable,
    InvalidFormat,
    InsufficientEntropy,
    InvalidAddress,
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvError::MissingVariable => write!(f, "Required environment variable not found"),
            EnvError::InvalidFormat => write!(f, "Environment variable has invalid format"),
            EnvError::InsufficientEntropy => write!(f, "Private key has insufficient entropy"),
            EnvError::InvalidAddress => write!(f, "Invalid blockchain address format"),
        }
    }
}

impl std::error::Error for EnvError {}

fn env_var(id: &str) -> Option<String> {
    std::env::var(id)
        .inspect_err(|_| {
            log::warn!("Could not read environment variable");
        })
        .ok()
}

fn secure_env_var(id: &str) -> Result<String, EnvError> {
    std::env::var(id)
        .map_err(|_| {
            log::error!("Missing required environment variable");
            EnvError::MissingVariable
        })
}

// Secure wrapper for sensitive environment variables
fn secure_secret_env_var(key: &str) -> Result<SecretString, EnvError> {
    std::env::var(key)
        .map(|value| SecretString::new(value.into()))
        .map_err(|_| {
            log::error!("Missing required sensitive environment variable: [REDACTED]");
            EnvError::MissingVariable
        })
}

// Validate private key format without exposing content
fn validate_private_key_format(secret: &SecretString) -> Result<(), EnvError> {
    let key = secret.expose_secret();
    
    // Basic validation: check length and hex format
    if key.len() != 64 && key.len() != 66 {
        log::error!("Invalid private key length: expected 64 or 66 characters");
        return Err(EnvError::InvalidFormat);
    }
    
    // Check if it's valid hex (with or without 0x prefix)
    let hex_key = if key.starts_with("0x") { &key[2..] } else { key };
    
    if !hex_key.chars().all(|c| c.is_ascii_hexdigit()) {
        log::error!("Invalid private key format: not valid hexadecimal");
        return Err(EnvError::InvalidFormat);
    }
    
    Ok(())
}

// Validate private key entropy without exposing content
fn validate_private_key_entropy(secret: &SecretString) -> Result<(), EnvError> {
    let key = secret.expose_secret();
    let hex_key = if key.starts_with("0x") { &key[2..] } else { key };
    
    // Convert hex to bytes for entropy analysis
    let key_bytes = match hex::decode(hex_key) {
        Ok(bytes) => bytes,
        Err(_) => {
            log::error!("Failed to decode private key for entropy validation");
            return Err(EnvError::InvalidFormat);
        }
    };
    
    // Check for obvious weak patterns
    if key_bytes.iter().all(|&b| b == 0) {
        log::error!("Private key contains all zeros - insufficient entropy");
        return Err(EnvError::InsufficientEntropy);
    }
    
    if key_bytes.iter().all(|&b| b == 0xFF) {
        log::error!("Private key contains all ones - insufficient entropy");
        return Err(EnvError::InsufficientEntropy);
    }
    
    // Check for repeating patterns
    let unique_bytes: std::collections::HashSet<u8> = key_bytes.iter().cloned().collect();
    if unique_bytes.len() < 8 {
        log::error!("Private key has insufficient byte diversity - insufficient entropy");
        return Err(EnvError::InsufficientEntropy);
    }
    
    // Basic entropy check using Shannon entropy approximation
    let mut byte_counts = [0u32; 256];
    for &byte in &key_bytes {
        byte_counts[byte as usize] += 1;
    }
    
    let mut entropy = 0.0;
    let total_bytes = key_bytes.len() as f64;
    
    for &count in &byte_counts {
        if count > 0 {
            let probability = count as f64 / total_bytes;
            entropy -= probability * probability.log2();
        }
    }
    
    // Require minimum entropy threshold (4.0 bits per byte is reasonable)
    if entropy < 4.0 {
        log::error!("Private key entropy too low: {:.2} bits per byte", entropy);
        return Err(EnvError::InsufficientEntropy);
    }
    
    Ok(())
}



pub fn hostname() -> String {
    env_var("HOSTNAME").unwrap_or(String::from("0.0.0.0"))
}

pub fn port() -> String {
    env_var("PORT").unwrap_or(String::from("36092"))
}

// Legacy functions removed - use secure_* versions instead

// Secure versions using SecretString for private keys with full validation
pub fn secure_claimerkey() -> Result<SecretString, EnvError> {
    let secret = secure_secret_env_var("CLAIMERKEY")?;
    validate_private_key_format(&secret)?;
    validate_private_key_entropy(&secret)?;
    Ok(secret)
}

pub fn secure_tokenownerkey() -> Result<SecretString, EnvError> {
    let secret = secure_secret_env_var("TOKENOWNERKEY")?;
    validate_private_key_format(&secret)?;
    validate_private_key_entropy(&secret)?;
    Ok(secret)
}

pub fn secure_tokenminterkey() -> Result<SecretString, EnvError> {
    let secret = secure_secret_env_var("TOKENMINTERKEY")?;
    validate_private_key_format(&secret)?;
    validate_private_key_entropy(&secret)?;
    Ok(secret)
}

pub fn secure_manualtokensigner() -> Result<SecretString, EnvError> {
    let secret = secure_secret_env_var("MANUALTOKENSIGNER")?;
    validate_private_key_format(&secret)?;
    validate_private_key_entropy(&secret)?;
    Ok(secret)
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

pub fn claimer() -> Result<Address, EnvError> {
    let addr_str = env_var("CLAIMER").unwrap_or("0xc749169dB9C231E1797Aa9cD7f5B7a88AeD25b08".to_string());
    Address::parse_checksummed(addr_str, None)
        .map_err(|_| {
            log::warn!("Failed to parse CLAIMER address, using fallback");
            EnvError::InvalidAddress
        })
}

pub fn genesis() -> Result<Address, EnvError> {
    let addr_str = env_var("GENESIS").unwrap_or("0x84599c907B42e9bc21F9FE26D9e5A5D3747109D3".to_string());
    Address::parse_checksummed(addr_str, None)
        .map_err(|_| {
            log::warn!("Failed to parse GENESIS address, using fallback");
            EnvError::InvalidAddress
        })
}

pub fn ownaiv1() -> Result<Address, EnvError> {
    let addr_str = env_var("OWNAIV1").unwrap_or("0x5d3a48B6f16Ba9a830b19B452d8DAA0409e0FE05".to_string());
    Address::parse_checksummed(addr_str, None)
        .map_err(|_| {
            log::warn!("Failed to parse OWNAIV1 address, using fallback");
            EnvError::InvalidAddress
        })
}

pub fn deposit() -> Result<Address, EnvError> {
    let addr_str = env_var("DEPOSIT").unwrap_or("0x1EdE9dE47e5E3B8941884e7f5DDa43D82570180D".to_string());
    Address::parse_checksummed(addr_str, None)
        .map_err(|_| {
            log::warn!("Failed to parse DEPOSIT address, using fallback");
            EnvError::InvalidAddress
        })
}

pub fn usdc() -> Result<Address, EnvError> {
    let addr_str = env_var("USDC").unwrap_or("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string());
    Address::parse_checksummed(addr_str, None)
        .map_err(|_| {
            log::warn!("Failed to parse USDC address, using fallback");
            EnvError::InvalidAddress
        })
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

pub fn hyperstackapikey() -> Result<String, EnvError> {
    let api_key = secure_env_var("HYPERSTACKAPIKEY")?;
    
    // Validate API key format (UUID format: 8-4-4-4-12 characters)
    if api_key.len() != 36 {
        log::error!("API key has invalid length");
        return Err(EnvError::InvalidFormat);
    }
    
    if !api_key.chars().enumerate().all(|(i, c)| {
        match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        }
    }) {
        log::error!("API key has invalid format");
        return Err(EnvError::InvalidFormat);
    }
    
    Ok(api_key)
}
