use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen,
    store::LookupMap,
    AccountId, Balance, PanicOnDefault, Promise,
};

use crate::*;
use crate::{types::StorageKey, PhoenixBonds};

const ERR_NO_LINEAR_TO_CLAIM: &str = "No lost and found LiNEAR to claim";

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct LostAndFound {
    total_amount: Balance,
    user_lost_found: LookupMap<AccountId, Balance>,
}

impl LostAndFound {
    pub fn new() -> Self {
        Self {
            total_amount: 0,
            user_lost_found: LookupMap::new(StorageKey::UserLostFound),
        }
    }

    pub fn total_amount(&self) -> Balance {
        self.total_amount
    }

    pub fn insert(&mut self, user_id: &AccountId, amount: Balance) {
        self.total_amount += amount;
        let prev = self.user_amount(user_id);
        self.user_lost_found.insert(user_id.clone(), prev + amount);

        Event::LostFoundInsert {
            account_id: user_id.clone(),
            amount: amount.into(),
        }
        .emit();
    }

    fn user_amount(&self, user_id: &AccountId) -> Balance {
        self.user_lost_found.get(user_id).copied().unwrap_or(0)
    }

    fn remove(&mut self, user_id: &AccountId) -> Balance {
        let amount = self.user_amount(user_id);
        self.total_amount -= amount;
        self.user_lost_found.remove(user_id);

        Event::LostFoundClaim {
            account_id: user_id.clone(),
            amount: amount.into(),
        }
        .emit();

        amount
    }
}

#[near_bindgen]
impl PhoenixBonds {
    pub fn user_lost_and_found(&self, account_id: AccountId) -> U128 {
        self.linear_lost_and_found.user_amount(&account_id).into()
    }

    pub fn claim_lost_and_found(&mut self) -> Promise {
        // 100 Tgas
        require!(
            env::prepaid_gas() >= Gas(20 * TGAS) + GAS_FT_TRANSFER_AND_CALLBACK,
            ERR_NOT_ENOUGH_GAS
        );

        let user_id = env::predecessor_account_id();
        let amount = self.linear_lost_and_found.remove(&user_id);

        require!(amount > 0, ERR_NO_LINEAR_TO_CLAIM);

        self.transfer_linear(&user_id, amount, "Claim lost and found")
    }
}
