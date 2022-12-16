use crate::*;
use near_contract_standards::fungible_token::events::FtMint;
use near_sdk::{json_types::U128, AccountId, Balance, PromiseOrValue};

near_contract_standards::impl_fungible_token_core!(MockLinear, tokens);
near_contract_standards::impl_fungible_token_storage!(MockLinear, tokens);

impl MockLinear {
    pub(crate) fn mint_linear(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        memo: Option<&str>,
    ) {
        if !self.tokens.accounts.contains_key(account_id) {
            self.tokens.internal_register_account(account_id);
        }
        self.tokens.internal_deposit(account_id, amount);
        FtMint {
            owner_id: account_id,
            amount: &U128(amount),
            memo,
        }
        .emit();
    }
}
