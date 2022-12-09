use near_bigdecimal::*;
#[allow(unused_imports)]
use near_sdk::borsh::BorshDeserialize;
use near_sdk::{env, Timestamp};
use near_sdk::{near_bindgen, Balance};

use crate::types::FULL_BASIS_POINT;
use crate::*;

fn near_like_decimals() -> BigDecimal {
    BigDecimal::from(10_u64).pow(24)
}

pub fn near2linear(near_amount: Balance, linear_price: Balance) -> Balance {
    (BigDecimal::from(near_amount) * near_like_decimals() / linear_price.into()).round_u128()
}

pub fn linear2near(linear_amount: Balance, linear_price: Balance) -> Balance {
    (BigDecimal::from(linear_amount) * linear_price.into() / near_like_decimals()).round_u128()
}

pub fn pnear2near(pnear_amount: Balance, pnear_price: Balance) -> Balance {
    (BigDecimal::from(pnear_amount) * pnear_price.into() / near_like_decimals()).round_u128()
}

pub fn near2pnear(near_amount: Balance, pnear_price: Balance) -> Balance {
    (BigDecimal::from(near_amount) * near_like_decimals() / pnear_price.into()).round_u128()
}

pub fn apply_basis_point(value: u128, point: u32) -> u128 {
    value * point as u128 / FULL_BASIS_POINT as u128
}

#[cfg(not(feature = "test"))]
pub fn current_timestamp_ms() -> Timestamp {
    env::block_timestamp_ms()
}

#[cfg(feature = "test")]
pub fn current_timestamp_ms() -> Timestamp {
    let test_timestamp_key: &[u8] = "_test_ts_".as_bytes();
    let raw_timestamp_option = env::storage_read(test_timestamp_key);

    if let Some(raw_ts) = raw_timestamp_option {
        u64::try_from_slice(&raw_ts).unwrap_or(0)
    } else {
        0
    }
}

#[near_bindgen]
impl PhoenixBonds {
    #[cfg(feature = "test")]
    pub fn set_current_timestamp_ms(&mut self, ms: Timestamp) {
        let test_timestamp_key: &[u8] = "_test_ts_".as_bytes();
        env::storage_write(test_timestamp_key, &ms.try_to_vec().unwrap_or_default());
    }
}

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

#[cfg(test)]
pub mod tests {
    pub const ONE_DAY_MS: u64 = 24 * 3600 * 1000;
}
