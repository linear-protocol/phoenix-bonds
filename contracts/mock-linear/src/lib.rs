use near_bigdecimal::*;
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen, require, PanicOnDefault, ONE_NEAR,
};

mod ft;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct MockLinear {
    linear_price: u128,
    tokens: FungibleToken,
    panic: bool,
    small_change: bool,
}

#[near_bindgen]
impl MockLinear {
    #[init]
    pub fn new() -> Self {
        Self {
            linear_price: ONE_NEAR,
            tokens: FungibleToken::new(b't'),
            panic: false,
            small_change: false,
        }
    }

    // -- public LiNEAR methods

    #[payable]
    pub fn deposit_and_stake(&mut self) -> U128 {
        require!(!self.panic, "LiNEAR Panic");
        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();
        let mut shares =
            (BigDecimal::from(amount) * ONE_NEAR.into() / self.linear_price.into()).round_u128();
        if self.small_change {
            shares -= 10;
        }

        self.mint_linear(&account_id, shares, None);

        shares.into()
    }

    pub fn ft_price(&self) -> U128 {
        self.linear_price.into()
    }

    // -- mock contract methods

    pub fn set_ft_price(&mut self, price: U128) {
        self.linear_price = price.0;
    }

    pub fn set_panic(&mut self, panic: bool) {
        self.panic = panic;
    }

    pub fn set_small_change(&mut self, small_change: bool) {
        self.small_change = small_change;
    }
}
