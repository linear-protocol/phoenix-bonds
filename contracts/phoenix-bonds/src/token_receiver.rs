use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{near_bindgen, serde::Deserialize, PromiseOrValue};

const MINIMUM_BOND_LINEAR_AMOUNT: u128 = ONE_NEAR / 10 + ONE_NEAR / 100; // 0.11 LiNEAR

const ERR_BAD_MSG: &str = "Unrecognized message";
const ERR_BAD_TOKEN: &str = "Only LiNEAR token can be used to bond";
const ERR_SMALL_BOND_LINEAR_AMOUNT: &str = "Bond amount must be at least 0.11 LiNEAR";
const ERR_LINEAR_PRICE: &str = "Failed to get LiNEAR price";
const ERR_MALFORMED_MESSAGE: &str = "Invalid transfer action message";

#[derive(Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
enum Action {
    Bond,
}

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

        let action = serde_json::from_str::<Action>(&msg).expect(ERR_MALFORMED_MESSAGE);
        require!(action == Action::Bond, ERR_BAD_MSG);

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
        let linear_price = linear_price.expect(ERR_LINEAR_PRICE).0;
        let near_amount = linear2near(linear_amount.0, linear_price);
        let bond_amount = near_amount - BOND_STORAGE_DEPOSIT;

        // This guarantees the pNEAR redeem price is consistent after bonding,
        // but it will make some LiNEAR left in the contract balance but not in any of the pools.
        self.internal_create_bond(user_id, bond_amount, near2linear(bond_amount, linear_price));

        U128(0)
    }
}
