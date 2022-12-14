//! This module contains all contract state versions, which are needed
//! when upgrading contract.
use crate::*;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen, AccountId, Balance,
};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ContractV1_0_0 {
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
    /// amout of NEAR that the protocol owns
    permanent_pool_near_amount: Balance,
    /// amount of NEAR to reward AMM liquidity provider
    treasury_pool_near_amount: Balance,
    /// max percentage of bond amount that goes to treasury pool when a user claims
    tau: BasisPoint,

    /// amount of LiNEAR that was not sucessfully transferred
    linear_lost_and_found: LostAndFound,
    /// bond note for each user
    bond_notes: BondNotes,
    /// when bootstraping period ends, before which commit & redeem are disabled
    bootstrap_ends_at: Timestamp,
    /// helper module to calculate accural parameter (alpha)
    accural_param: AccuralParameter,
}
