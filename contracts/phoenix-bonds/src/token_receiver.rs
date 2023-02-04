use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{near_bindgen, PromiseOrValue};

const MINIMUM_BOND_LINEAR_AMOUNT: u128 = ONE_NEAR / 10; // 0.1 LiNEAR
const BOND_MSG: &str = "Bond";

const ERR_BAD_MSG: &str = "Unrecognized message";
const ERR_BAD_TOKEN: &str = "Only LiNEAR token can be used to bond";
const ERR_SMALL_BOND_LINEAR_AMOUNT: &str = "Bond amount must be at least 0.1 LiNEAR";

#[near_bindgen]
impl FungibleTokenReceiver for PhoenixBonds {
    #[allow(unused_variables)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        require!(env::prepaid_gas() >= GAS_FT_ON_TRANSFER, ERR_NOT_ENOUGH_GAS);
        require!(msg == BOND_MSG, ERR_BAD_MSG);

        let token_address = env::predecessor_account_id();
        require!(token_address == self.linear_address, ERR_BAD_TOKEN);
        require!(
            amount.0 >= MINIMUM_BOND_LINEAR_AMOUNT,
            ERR_SMALL_BOND_LINEAR_AMOUNT
        );

        self.get_linear_price()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_LINEAR_BOND_CALLBACK)
                    .on_get_linear_price_for_linear_bond(sender_id, amount),
            )
            .into()
    }
}

#[near_bindgen]
impl PhoenixBonds {
    #[private]
    pub fn on_get_linear_price_for_linear_bond(
        &mut self,
        user_id: AccountId,
        linear_amount: U128,
        #[callback_result] linear_price: Result<U128, PromiseError>,
    ) -> U128 {
        let linear_price = linear_price.unwrap().0;
        let near_amount = linear2near(linear_amount.0, linear_price);
        let bond_amount = near_amount - BOND_STORAGE_DEPOSIT;

        self.internal_create_bond(user_id, bond_amount, linear_amount.0);

        U128(0)
    }
}
