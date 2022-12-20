use crate::{
    interfaces::{ext_fungible_token, linear_contract},
    types::*,
    utils::*,
};
use accrual::{AccrualConfig, AccrualParameter};
use bond_note::{BondNote, BondNotes, BondStatus};
use events::Event;
use lost_found::LostAndFound;
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, is_promise_success,
    json_types::U128,
    near_bindgen, require, AccountId, Balance, PanicOnDefault, Promise, PromiseError, ONE_NEAR,
    ONE_YOCTO,
};
use types::{BasisPoint, Duration, StorageKey, Timestamp, FULL_BASIS_POINT};

mod accrual;
mod active_vector;
mod bond_note;
mod events;
mod fungible_token;
mod interfaces;
mod legacy;
mod lost_found;
mod math;
mod owner;
mod types;
mod upgrade;
mod utils;
mod view;

const MINIMUM_BOND_AMOUNT: u128 = ONE_NEAR / 10; // 0.1 NEAR
const BOND_STORAGE_DEPOSIT: u128 = ONE_NEAR / 100; // 0.01 NEAR

const ERR_PAUSED: &str = "Contract paused. Please try again later";
const ERR_INVALID_TAU: &str = "Invalid tau";
const ERR_BOND_DEPOSIT: &str = "Bond requires 0.01 NEAR as storage deposit";
const ERR_SMALL_BOND_AMOUNT: &str = "Bond amount must be at least 0.1 NEAR";
const ERR_BOND_NOT_PENDING: &str = "Bond is not pending";
const ERR_GET_LINEAR_PRICE: &str = "Failed to get LiNEAR price";
const ERR_NOT_ENOUGH_PNEAR_BALANCE: &str = "Not enough pNEAR balance";
const ERR_INVALID_TRANSFER_AMOUNT: &str = "Amount of LiNEAR to transfer must not be zero";
const ERR_BOOTSTRAPPING: &str = "Commit and redeem are not allowed now";
const ERR_BAD_BOOTSTRAP_END: &str = "Bootstrap end time must be in the future";
const ERR_NOT_ENOUGH_GAS: &str = "Not enough gas";
const ERR_BURN_TOO_MANY: &str = "At least one pNEAR must be left";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PhoenixBonds {
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

pub(crate) fn assert_tau(tau: BasisPoint) {
    require!(tau < FULL_BASIS_POINT, ERR_INVALID_TAU)
}

#[near_bindgen]
impl PhoenixBonds {
    #[init]
    pub fn new(
        owner_id: AccountId,
        linear_address: AccountId,
        tau: BasisPoint,
        bootstrap_ends: Timestamp,
        accrual: AccrualConfig,
    ) -> Self {
        require!(
            bootstrap_ends > current_timestamp_ms(),
            ERR_BAD_BOOTSTRAP_END
        );
        assert_tau(tau);
        accrual.assert_valid();

        Self {
            ft: FungibleToken::new(StorageKey::FungibleToken),
            owner_id,
            linear_address,
            paused: false,
            linear_balance: 0,
            pending_pool_near_amount: 0,
            permanent_pool_near_amount: 0,
            treasury_pool_near_amount: 0,
            tau,
            linear_lost_and_found: LostAndFound::new(),
            bond_notes: BondNotes::new(),
            bootstrap_ends_at: bootstrap_ends,
            accrual_param: AccrualParameter::new(
                accrual.alpha,
                accrual.min_alpha,
                accrual.target_mean_length,
                accrual.adjust_interval,
                accrual.adjust_rate,
            ),
        }
    }

    // ======== Bond ========

    /// Create a new bond by depositing NEAR
    #[payable]
    pub fn bond(&mut self) -> Promise {
        // 120 Tgas
        require!(
            env::prepaid_gas() >= GAS_BOND + GAS_DEPOSIT_AND_STAKE + GAS_BOND_CALLBACK,
            ERR_NOT_ENOUGH_GAS
        );
        require!(!self.paused, ERR_PAUSED);

        let user_id = env::predecessor_account_id();

        require!(
            env::attached_deposit() > BOND_STORAGE_DEPOSIT,
            ERR_BOND_DEPOSIT
        );
        let bond_amount = env::attached_deposit() - BOND_STORAGE_DEPOSIT;
        require!(bond_amount >= MINIMUM_BOND_AMOUNT, ERR_SMALL_BOND_AMOUNT);

        // stake on linear
        linear_contract::ext(self.linear_address.clone())
            .with_static_gas(GAS_DEPOSIT_AND_STAKE)
            .with_attached_deposit(bond_amount)
            .deposit_and_stake_v2()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_BOND_CALLBACK)
                    .on_staked(user_id, U128(bond_amount)),
            )
    }

    #[private]
    pub fn on_staked(
        &mut self,
        user_id: AccountId,
        bond_amount: U128,
        #[callback_result] staked_linear_amount: Result<U128, PromiseError>,
    ) -> Option<u32> {
        if let Ok(linear_amount) = staked_linear_amount {
            self.pending_pool_near_amount += bond_amount.0;
            self.linear_balance += linear_amount.0;

            self.accrual_param
                .weighted_mean_insert(bond_amount.0, current_timestamp_ms());

            let note = self.bond_notes.insert_new_note(&user_id, bond_amount.0);

            Event::Bond {
                account_id: user_id,
                bond_amount,
                linear_amount,
            }
            .emit();

            Some(note.id())
        } else {
            // refund user deposited NEAR
            Promise::new(user_id).transfer(bond_amount.0 + BOND_STORAGE_DEPOSIT);
            None
        }
    }

    // ======== Cancel ========

    /// Cancel a bond, will return corresponding LiNEAR tokens to the user
    #[payable]
    pub fn cancel(&mut self, note_id: u32) -> Promise {
        // 160 Tgas
        require!(
            env::prepaid_gas() >= GAS_CANCEL + GAS_GET_LINEAR_PRICE + GAS_CANCEL_CALLBACK,
            ERR_NOT_ENOUGH_GAS
        );
        assert_one_yocto();
        require!(!self.paused, ERR_PAUSED);

        let user_id = env::predecessor_account_id();
        let bond_note = self.bond_notes.get_user_note(&user_id, note_id);

        require!(
            bond_note.status() == BondStatus::Pending,
            ERR_BOND_NOT_PENDING
        );

        self.get_linear_price().then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_CANCEL_CALLBACK)
                .on_get_linear_price_for_cancel(user_id, note_id),
        )
    }

    #[private]
    pub fn on_get_linear_price_for_cancel(
        &mut self,
        user_id: AccountId,
        note_id: u32,
        #[callback_result] linear_price: Result<U128, PromiseError>,
    ) -> Promise {
        let linear_price = linear_price.expect(ERR_GET_LINEAR_PRICE);
        let mut bond_note = self.bond_notes.get_user_note(&user_id, note_id);

        let refund_linear = near2linear(bond_note.bond_amount(), linear_price.0);

        // update user note
        bond_note.cancel();
        self.bond_notes
            .save_user_note(&user_id, note_id, bond_note.clone());

        // update status
        self.pending_pool_near_amount -= bond_note.bond_amount();
        self.linear_balance -= refund_linear;

        let current_timestamp = current_timestamp_ms();
        self.accrual_param.weighted_mean_remove(
            bond_note.bond_amount(),
            bond_note.length(current_timestamp),
            current_timestamp,
        );

        Event::Cancel {
            account_id: user_id.clone(),
            note_id,
            bond_amount: bond_note.bond_amount().into(),
            refund_linear: refund_linear.into(),
        }
        .emit();

        // transfer LiNEAR to user
        self.transfer_linear(&user_id, refund_linear, "Cancel Bond")
    }

    // ======== Commit ========

    #[payable]
    pub fn commit(&mut self, note_id: u32) -> Promise {
        // 90 Tgas
        require!(
            env::prepaid_gas() >= GAS_COMMIT + GAS_GET_LINEAR_PRICE + GAS_COMMIT_CALLBACK,
            ERR_NOT_ENOUGH_GAS
        );
        assert_one_yocto();
        require!(!self.paused, ERR_PAUSED);

        require!(
            current_timestamp_ms() >= self.bootstrap_ends_at,
            ERR_BOOTSTRAPPING
        );

        let user_id = env::predecessor_account_id();
        let bond_note = self.bond_notes.get_user_note(&user_id, note_id);
        require!(
            bond_note.status() == BondStatus::Pending,
            ERR_BOND_NOT_PENDING
        );

        self.get_linear_price().then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_COMMIT_CALLBACK)
                .on_get_linear_price_for_commit(user_id, note_id),
        )
    }

    #[private]
    pub fn on_get_linear_price_for_commit(
        &mut self,
        user_id: AccountId,
        note_id: u32,
        #[callback_result] linear_price: Result<U128, PromiseError>,
    ) -> U128 {
        let linear_price = linear_price.expect(ERR_GET_LINEAR_PRICE);
        let mut bond_note = self.bond_notes.get_user_note(&user_id, note_id);
        let bond_amount = bond_note.bond_amount();

        let current_timestamp = current_timestamp_ms();
        let is_first_commit = self.pnear_total_supply() == 0;

        let amount_for_treasury = apply_basis_point(bond_amount, self.tau);
        let mut treasury_gained_near_amount = amount_for_treasury;
        if is_first_commit {
            // assign all staking profits before the first commit to treasury
            treasury_gained_near_amount +=
                linear2near(self.linear_balance, linear_price.0) - self.pending_pool_near_amount;
        }

        let reserve_should_gain_near_amount = self.accrued_amount(
            bond_amount - amount_for_treasury,
            bond_note.length(current_timestamp),
            current_timestamp,
        );

        // this should be equal to: near2pnear(reserve_should_gain_near_amount, self.pnear_price(linear_price.0))
        let pnear_to_mint = self.note_accrued_pnear(&bond_note, linear_price.0, current_timestamp);

        let permanent_gained_near_amount =
            bond_amount - amount_for_treasury - reserve_should_gain_near_amount;

        // update state
        bond_note.commit(pnear_to_mint);
        let note_length = bond_note.length(current_timestamp);
        self.bond_notes.save_user_note(&user_id, note_id, bond_note);

        self.treasury_pool_near_amount += treasury_gained_near_amount;
        self.permanent_pool_near_amount += permanent_gained_near_amount;
        self.pending_pool_near_amount -= bond_amount;

        self.accrual_param
            .weighted_mean_remove(bond_amount, note_length, current_timestamp);

        self.mint_pnear(&user_id, pnear_to_mint, Some("Commit Bond"));

        Event::Commit {
            account_id: user_id,
            note_id,
            bond_amount: bond_amount.into(),
            pnear_amount: pnear_to_mint.into(),
        }
        .emit();

        pnear_to_mint.into()
    }

    // ======== Redeem ========

    #[payable]
    pub fn redeem(&mut self, amount: U128) -> Promise {
        // 160 Tgas
        require!(
            env::prepaid_gas() >= GAS_REDEEM + GAS_GET_LINEAR_PRICE + GAS_REDEEM_CALLBACK,
            ERR_NOT_ENOUGH_GAS
        );
        assert_one_yocto();
        require!(!self.paused, ERR_PAUSED);

        require!(
            current_timestamp_ms() >= self.bootstrap_ends_at,
            ERR_BOOTSTRAPPING
        );

        let user_id = env::predecessor_account_id();
        require!(
            self.ft.internal_unwrap_balance_of(&user_id) >= amount.0,
            ERR_NOT_ENOUGH_PNEAR_BALANCE
        );
        require!(
            self.pnear_total_supply() - amount.0 > ONE_PNEAR,
            ERR_BURN_TOO_MANY
        );

        self.get_linear_price().then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_REDEEM_CALLBACK)
                .on_get_linear_price_for_redeem(user_id, amount),
        )
    }

    #[private]
    pub fn on_get_linear_price_for_redeem(
        &mut self,
        user_id: AccountId,
        pnear_amount: U128,
        #[callback_result] linear_price: Result<U128, PromiseError>,
    ) -> Promise {
        let linear_price = linear_price.expect(ERR_GET_LINEAR_PRICE);
        require!(
            self.ft.internal_unwrap_balance_of(&user_id) >= pnear_amount.0,
            ERR_NOT_ENOUGH_PNEAR_BALANCE
        );
        require!(
            self.pnear_total_supply() - pnear_amount.0 > ONE_PNEAR,
            ERR_BURN_TOO_MANY
        );

        // equivalent amount of NEAR that given pNEAR worth
        let equivalent_near_amount = pnear2near(pnear_amount.0, self.pnear_price(linear_price.0));
        let redeemed_linear = near2linear(equivalent_near_amount, linear_price.0);

        self.linear_balance -= redeemed_linear;
        self.burn_pnear(&user_id, pnear_amount.0, Some("Redeem pNEAR"));

        Event::Redeem {
            account_id: user_id.clone(),
            pnear_amount,
            redeemed_linear: redeemed_linear.into(),
        }
        .emit();

        self.transfer_linear(&user_id, redeemed_linear, "pNEAR Redeem")
    }

    // ======== Helper Methods ========

    fn get_linear_price(&self) -> Promise {
        linear_contract::ext(self.linear_address.clone())
            .with_static_gas(GAS_GET_LINEAR_PRICE)
            .ft_price()
    }

    /// Transfer LiNEAR to given account
    /// If transfer failed, these LiNEAR will be moved to lost and found
    /// NOTE: Make sure LiNEAR balance is decreased before calling this!
    fn transfer_linear(&mut self, account_id: &AccountId, amount: Balance, memo: &str) -> Promise {
        require!(amount > 0, ERR_INVALID_TRANSFER_AMOUNT);

        ext_fungible_token::ext(self.linear_address.clone())
            .with_static_gas(GAS_FT_TRANSFER)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer(account_id.clone(), amount.into(), Some(memo.to_string()))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FT_TRANSFER_CALLBACK)
                    .on_linear_transferred(account_id.clone(), amount.into()),
            )
    }

    /// We assume all LiNEAR token transfer will succeed,
    /// if it failed, then those tokens will be moved to the lost and found pool,
    /// instead of reverting contract state.
    /// Returns the amount of LiNEAR that was successfully transferred.
    #[private]
    pub fn on_linear_transferred(&mut self, user_id: AccountId, linear_amount: U128) -> U128 {
        if is_promise_success() {
            return linear_amount;
        }

        self.linear_lost_and_found.insert(&user_id, linear_amount.0);
        0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Allow to manually override contract states in test
    pub(crate) fn new_contract(
        linear_balance: Balance,
        pending_pool_near_amount: Balance,
        permanent_pool_near_amount: Balance,
        treasury_pool_near_amount: Balance,
        alpha: Duration,
        tau: BasisPoint,
    ) -> PhoenixBonds {
        let owner = AccountId::new_unchecked("foo".into());
        let linear = AccountId::new_unchecked("bar".into());
        let min_alpha: Duration = 1;
        let target_mean_length: u64 = 15 * 86400 * 1000;
        let adjust_interval: u64 = 86400 * 1000;
        let adjust_rate = 100;
        let mut contract = PhoenixBonds::new(
            owner,
            linear,
            tau,
            1,
            AccrualConfig {
                alpha,
                min_alpha,
                target_mean_length,
                adjust_interval,
                adjust_rate,
            },
        );

        contract.linear_balance = linear_balance;
        contract.pending_pool_near_amount = pending_pool_near_amount;
        contract.permanent_pool_near_amount = permanent_pool_near_amount;
        contract.treasury_pool_near_amount = treasury_pool_near_amount;

        contract
    }
}
