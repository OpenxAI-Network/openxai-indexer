use alloy::{
    primitives::{Address, Uint},
    providers::{Provider, ProviderBuilder},
    signers::{Error, Signature, Signer, local::PrivateKeySigner},
    sol_types::{SolStruct, eip712_domain},
};

use crate::{
    blockchain::{claimer::Claim, ownai_v1::OpenxAITokenizedServerV1},
    utils::env::{chainid, claimer, claimerkey, ownaiv1, tokenminterkey, tokenownerkey},
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
    };

    // Derive the EIP-712 signing hash.
    let hash = claim.eip712_signing_hash(&domain);

    // Sign the hash asynchronously with the wallet.
    let signature = signer.sign_hash(&hash).await?;

    Ok(signature)
}

pub fn get_tokenized_server_owner() -> PrivateKeySigner {
    tokenownerkey()
        .parse()
        .unwrap_or_else(|e| panic!("Could not parse tokenownerkey: {e}"))
}

pub async fn mint_tokenized_server<P: Provider>(
    provider: P,
    to: Address,
    token_id: i64,
) -> Option<Error> {
    let signer: PrivateKeySigner = tokenminterkey()
        .parse()
        .unwrap_or_else(|e| panic!("Could not parse tokenminterkey: {e}"));
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_provider(provider);

    let ownaiv1 = OpenxAITokenizedServerV1::new(ownaiv1(), provider);
    ownaiv1.mint(to, Uint::from(token_id)).send().await.err()?;

    None
}
