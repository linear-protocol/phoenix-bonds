//! This module contains all contract state versions, which are needed
//! when upgrading contract.
use crate::{accrual::WeightedMeanLength, *};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen, AccountId, Balance,
};

// ------ v1.0.0 ------

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ContractV1_0_0 {
    /// pNEAR token
    pub ft: FungibleToken,
    /// contract owner
    pub owner_id: AccountId,
    /// LiNEAR contract address
    pub linear_address: AccountId,
    /// if all user interactions of the contract should be paused
    pub paused: bool,

    /// total LiNEAR balance this contract holds
    pub linear_balance: Balance,
    /// amount of NEAR that has been bonded but not yet claimed/canceled
    pub pending_pool_near_amount: Balance,
    /// amount of NEAR that the protocol owns
    pub permanent_pool_near_amount: Balance,
    /// amount of NEAR to reward AMM liquidity provider
    pub treasury_pool_near_amount: Balance,
    /// percentage of bond amount that goes to treasury pool when a user claims
    pub tau: BasisPoint,

    /// amount of LiNEAR that was not successfully transferred
    pub linear_lost_and_found: LostAndFound,
    /// bond note for each user
    pub bond_notes: BondNotes,
    /// when bootstrapping period ends, before which commit & redeem are disabled
    pub bootstrap_ends_at: Timestamp,
    /// helper module to calculate accrual parameter (alpha)
    pub accrual_param: AccrualParameterV1_0_0,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AccrualParameterV1_0_0 {
    /// last updated alpha value
    pub alpha: Duration,
    /// minimum value that alpha could decrease to
    pub min_alpha: Duration,
    /// target weighted mean length of all pending bonds
    pub target_mean_length: Duration,
    /// alpha decreases each interval after mean length exceeds target
    pub adjust_interval: Duration,
    /// how much should alpha decrease in each interval
    pub adjust_rate: BasisPoint,
    /// when was alpha last updated
    pub last_updated_at: Timestamp,
    /// volume weighted mean bonding length
    pub mean_length: WeightedMeanLength,
}

impl From<AccrualParameterV1_0_0> for AccrualParameter {
    fn from(val: AccrualParameterV1_0_0) -> Self {
        Self {
            alpha: val.alpha,
            min_alpha: val.min_alpha,
            target_mean_length: val.target_mean_length,
            adjust_interval: val.adjust_interval,
            adjust_rate: val.adjust_rate,
            exceeds_target_at: 0, // TODO
            mean_length: val.mean_length,
        }
    }
}

// ------ v1.0.1 ------
// - Replace last_updated_at with exceeds_target_at in AccrualParameterV1_0_0

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ContractV1_0_1 {
    /// pNEAR token
    ft: FungibleToken,
    /// contract owner
    owner_id: AccountId,
    /// LiNEAR contract address
    linear_address: AccountId,
    /// if all user interactions of the contract should be paused
    paused: bool,

    /// total LiNEAR balance this contract holds
    linear_balance: Balance,
    /// amount of NEAR that has been bonded but not yet claimed/canceled
    pending_pool_near_amount: Balance,
    /// amount of NEAR that the protocol owns
    permanent_pool_near_amount: Balance,
    /// amount of NEAR to reward AMM liquidity provider
    treasury_pool_near_amount: Balance,
    /// percentage of bond amount that goes to treasury pool when a user claims
    tau: BasisPoint,

    /// amount of LiNEAR that was not successfully transferred
    linear_lost_and_found: LostAndFound,
    /// bond note for each user
    bond_notes: BondNotes,
    /// when bootstrapping period ends, before which commit & redeem are disabled
    bootstrap_ends_at: Timestamp,
    /// helper module to calculate accrual parameter (alpha)
    accrual_param: AccrualParameter,
}
