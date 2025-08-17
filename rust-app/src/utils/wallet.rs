use alloy::{
    primitives::{Address, Uint},
    providers::{Provider, ProviderBuilder},
    signers::{Signature, Signer, local::PrivateKeySigner},
    sol_types::{SolStruct, eip712_domain},
};

use crate::{
    blockchain::{claimer::Claim, ownai_v1::OpenxAITokenizedServerV1},
    utils::env::{chainid, claimer, ownaiv1, secure_claimerkey_protected, secure_tokenownerkey_protected, secure_tokenminterkey_protected},
};



pub async fn get_claimer_signature(claim: &Claim) -> Result<Signature, Box<dyn std::error::Error>> {
    let secure_key = secure_claimerkey_protected().map_err(|e| {
        log::error!("Failed to retrieve claimer key: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;
    
    let signer: PrivateKeySigner = match secure_key.with_key(|key_bytes| {
        let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
            log::error!("Invalid UTF-8 in private key: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;
        
        key_str.parse().map_err(|e| {
            log::error!("Failed to parse claimer key");
            Box::new(e) as Box<dyn std::error::Error>
        })
     }) {
        Ok(result) => result?,
        Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error>),
    };

    let claimer_addr = claimer().map_err(|e| {
        log::error!("Failed to get claimer address: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    let domain = eip712_domain! {
        name: "OpenxAIClaiming",
        version: "1",
        chain_id: chainid(),
        verifying_contract: claimer_addr,
    };

    // Derive the EIP-712 signing hash.
    let hash = claim.eip712_signing_hash(&domain);

    // Sign the hash asynchronously with the wallet.
    let signature = signer.sign_hash(&hash).await.map_err(|e| {
        log::error!("Failed to sign hash");
        e
    })?;

    Ok(signature)
}

pub fn get_tokenized_server_owner() -> Result<PrivateKeySigner, Box<dyn std::error::Error>> {
    let secure_key = secure_tokenownerkey_protected().map_err(|e| {
        log::error!("Failed to retrieve token owner key: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;
    
    match secure_key.with_key(|key_bytes| {
        let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
            log::error!("Invalid UTF-8 in private key: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;
        
        key_str.parse().map_err(|e| {
            log::error!("Failed to parse token owner key");
            Box::new(e) as Box<dyn std::error::Error>
        })
    }) {
        Ok(result) => result,
        Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
    }
}

pub async fn mint_tokenized_server<P: Provider>(
    provider: P,
    to: Address,
    token_id: i64,
) -> Option<Box<dyn std::error::Error>> {
    let secure_key = match secure_tokenminterkey_protected() {
        Ok(key) => key,
        Err(e) => {
            log::error!("Failed to retrieve token minter key: {}", e);
            return Some(e.into());
        }
    };
    
    let signer: PrivateKeySigner = match secure_key.with_key(|key_bytes| {
        let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
            format!("Invalid UTF-8 in private key: {}", e)
        })?;
        
        key_str.parse().map_err(|e| {
            format!("Failed to parse private key: {}", e)
        })
    }) {
        Ok(result) => match result {
            Ok(signer) => signer,
            Err(e) => {
                log::error!("Failed to parse private key: {}", e);
                return Some(e.into());
            }
        },
        Err(e) => {
            let error_msg = format!("Security error: {}", e);
            log::error!("{}", error_msg);
            return Some(error_msg.into());
        }
    };
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_provider(provider);

    let ownaiv1_addr = match ownaiv1() {
        Ok(addr) => addr,
        Err(e) => {
            log::error!("Failed to get ownaiv1 address: {}", e);
            return Some(e.into());
        }
    };
    let ownaiv1 = OpenxAITokenizedServerV1::new(ownaiv1_addr, provider);
    ownaiv1.mint(to, Uint::from(token_id)).send().await.err()?;

    None
}
