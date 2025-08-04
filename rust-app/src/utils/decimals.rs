use alloy::primitives::U256;

pub fn to_6_decimals(number: U256) -> U256 {
    number / U256::from(10).pow(U256::from(12))
}

pub fn to_18_decimals(number: U256) -> U256 {
    number * U256::from(10).pow(U256::from(12))
}
