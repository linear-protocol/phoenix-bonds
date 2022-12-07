use near_sdk::{
    borsh::{self, BorshSerialize},
    AccountId, BorshStorageKey, ONE_NEAR,
};

#[derive(BorshSerialize, BorshStorageKey)]
pub(crate) enum StorageKey {
    FungibleToken,
    BondNotes,
    UserLostFound,
    UserNotes(AccountId),
}

/// Timestamp in milliseconds
pub type Timestamp = u64;
/// Time duration in milliseconds
pub type Duration = u64;

pub type BasisPoint = u32;
pub const FULL_BASIS_POINT: u32 = 10000;

pub const PNEAR_DECIMALS: u8 = 24;
pub const ONE_PNEAR: u128 = ONE_NEAR;
