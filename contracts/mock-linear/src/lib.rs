use near_bigdecimal::*;
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen, PanicOnDefault, ONE_NEAR,
};

mod ft;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct MockLinear {
    linear_price: u128,
    tokens: FungibleToken,
}

#[near_bindgen]
impl MockLinear {
    #[init]
    pub fn new() -> Self {
        Self {
            linear_price: ONE_NEAR,
            tokens: FungibleToken::new(b't'),
        }
    }

    // -- public LiNEAR methods

    #[payable]
    pub fn deposit_and_stake_v2(&mut self) -> U128 {
        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();
        let shares =
            (BigDecimal::from(amount) * ONE_NEAR.into() / self.linear_price.into()).round_u128();

        self.mint_linear(&account_id, amount, None);

        shares.into()
    }

    pub fn ft_price(&self) -> U128 {
        self.linear_price.into()
    }

    // -- mock contract methods

    pub fn set_ft_price(&mut self, price: U128) {
        self.linear_price = price.0;
    }
}
