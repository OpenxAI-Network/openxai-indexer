use alloy::{
    primitives::{B256, U256},
    signers::{Error, Signature, Signer, local::PrivateKeySigner},
    sol_types::{SolStruct, eip712_domain},
};

use crate::{
    blockchain::models::Claim,
    utils::env::{chainid, claimer, claimerkey},
};

pub async fn get_claimer_signature(claim: &Claim) -> Result<Signature, Error> {
    let signer: PrivateKeySigner = claimerkey()
        .parse()
        .unwrap_or_else(|e| panic!("Could not parse claimerkey: {e}"));

    let domain = eip712_domain! {
        name: "OpenxAIClaiming",
        version: "1",
        chain_id: chainid(),
        verifying_contract: claimer(),
        salt: B256::from(U256::from(0)),
    };

    // Derive the EIP-712 signing hash.
    let hash = claim.eip712_signing_hash(&domain);

    // Sign the hash asynchronously with the wallet.
    let signature = signer.sign_hash(&hash).await?;

    Ok(signature)
}
