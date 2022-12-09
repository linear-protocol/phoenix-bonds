use crate::*;
use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_sdk::{json_types::U128, AccountId, Balance, PromiseOrValue};

near_contract_standards::impl_fungible_token_core!(PhoenixBonds, ft);

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
