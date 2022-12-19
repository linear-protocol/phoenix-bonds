use crate::*;
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore,
    events::{FtBurn, FtMint},
    resolver::FungibleTokenResolver,
};
use near_sdk::{json_types::U128, near_bindgen, AccountId, Balance, PromiseOrValue};

near_contract_standards::impl_fungible_token_storage!(PhoenixBonds, ft);

#[near_bindgen]
impl FungibleTokenCore for PhoenixBonds {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        require!(!self.paused, ERR_PAUSED);
        self.ft.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        require!(!self.paused, ERR_PAUSED);
        self.ft.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.ft.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.ft.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for PhoenixBonds {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, _) =
            self.ft
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        used_amount.into()
    }
}

impl PhoenixBonds {
    pub(crate) fn mint_pnear(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        memo: Option<&str>,
    ) {
        if !self.ft.accounts.contains_key(account_id) {
            self.ft.internal_register_account(account_id);
        }
        self.ft.internal_deposit(account_id, amount);
        FtMint {
            owner_id: account_id,
            amount: &U128(amount),
            memo,
        }
        .emit();
    }

    pub(crate) fn burn_pnear(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        memo: Option<&str>,
    ) {
        self.ft.internal_withdraw(account_id, amount);
        FtBurn {
            owner_id: account_id,
            amount: &U128(amount),
            memo,
        }
        .emit();
    }

    pub(crate) fn pnear_total_supply(&self) -> Balance {
        self.ft.total_supply
    }
}
