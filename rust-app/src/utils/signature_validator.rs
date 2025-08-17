use std::str::FromStr;

use alloy::{
    primitives::{Address, Bytes, eip191_hash_message, fixed_bytes},
    providers::Provider,
    signers::Signature,
    sol,
};
use secrecy::{ExposeSecret, SecretString};

sol! {
    #[sol(rpc)]
    interface IERC1271 {
          function isValidSignature(bytes32 _hash, bytes memory _signature) public view returns (bytes4 magicValue);
    }
}

pub async fn validate_signature<P: Provider>(
    provider: &P,
    account: &SecretString,
    message: &str,
    signature: &str,
) -> bool {
    validate_signature_internal(provider, account.expose_secret(), message, signature).await
}

pub async fn validate_signature_str<P: Provider>(
    provider: &P,
    account: &str,
    message: &str,
    signature: &str,
) -> bool {
    validate_signature_internal(provider, account, message, signature).await
}

async fn validate_signature_internal<P: Provider>(
    provider: &P,
    account: &str,
    message: &str,
    signature: &str,
) -> bool {
    let signature = match Signature::from_str(signature) {
        Ok(signature) => signature,
        Err(_e) => {
            return false;
        }
    };

    let account = match Address::parse_checksummed(account, None) {
        Ok(account) => account,
        Err(_e) => {
            return false;
        }
    };

    match provider.get_code_at(account).await {
        Ok(code) => {
            if code.is_empty() {
                validate_eoa_signature(account, message, signature)
            } else {
                validate_smart_contract_signature(provider, account, message, signature).await
            }
        }
        Err(_e) => false,
    }
}

pub fn validate_eoa_signature(account: Address, message: &str, signature: Signature) -> bool {
    signature
        .recover_address_from_msg(message)
        .is_ok_and(|signer| signer == account)
}

// https://eips.ethereum.org/EIPS/eip-1271
pub async fn validate_smart_contract_signature<P: Provider>(
    provider: &P,
    account: Address,
    message: &str,
    signature: Signature,
) -> bool {
    IERC1271::new(account, provider)
        .isValidSignature(
            eip191_hash_message(message),
            Bytes::from_iter(signature.as_bytes()),
        )
        .call()
        .await
        .is_ok_and(|magic_value| magic_value == fixed_bytes!("1626ba7e"))
}
