use near_sdk::{
    borsh::{self, BorshSerialize},
    AccountId, BorshStorageKey, Gas, ONE_NEAR,
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

// -- Gas
pub const TGAS: u64 = Gas::ONE_TERA.0;

pub const GAS_BOND: Gas = Gas(20 * TGAS);
pub const GAS_BOND_CALLBACK: Gas = Gas(50 * TGAS);
pub const GAS_CANCEL: Gas = Gas(20 * TGAS);
/// 120 Tgas
pub const GAS_CANCEL_CALLBACK: Gas = Gas(40 * TGAS + GAS_FT_TRANSFER_AND_CALLBACK.0);
pub const GAS_COMMIT: Gas = Gas(20 * TGAS);
pub const GAS_COMMIT_CALLBACK: Gas = Gas(50 * TGAS);
pub const GAS_REDEEM: Gas = Gas(20 * TGAS);
/// 120 Tgas
pub const GAS_REDEEM_CALLBACK: Gas = Gas(40 * TGAS + GAS_FT_TRANSFER_AND_CALLBACK.0);
pub const GAS_CLAIM: Gas = Gas(20 * TGAS);
pub const GAS_WITHDRAW: Gas = Gas(20 * TGAS);
/// 120 Tgas
pub const GAS_WITHDRAW_CALLBACK: Gas = Gas(40 * TGAS + GAS_FT_TRANSFER_AND_CALLBACK.0);
/// 90 Tgas
pub const GAS_FT_ON_TRANSFER: Gas =
    Gas(20 * TGAS + GAS_GET_LINEAR_PRICE.0 + GAS_LINEAR_BOND_CALLBACK.0);
pub const GAS_LINEAR_BOND_CALLBACK: Gas = Gas(50 * TGAS);

pub const GAS_DEPOSIT_AND_STAKE: Gas = Gas(50 * TGAS);
pub const GAS_GET_LINEAR_PRICE: Gas = Gas(20 * TGAS);

pub const GAS_FT_TRANSFER: Gas = Gas(50 * TGAS);
pub const GAS_FT_TRANSFER_CALLBACK: Gas = Gas(30 * TGAS);
/// 80 Tgas
pub const GAS_FT_TRANSFER_AND_CALLBACK: Gas = Gas(GAS_FT_TRANSFER.0 + GAS_FT_TRANSFER_CALLBACK.0);
