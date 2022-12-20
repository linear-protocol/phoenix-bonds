use near_sdk::{ext_contract, json_types::U128};

#[ext_contract(linear_contract)]
pub trait LiNEARInterface {
    fn deposit_and_stake(&self) -> U128;
    fn ft_price(&self) -> U128;
}
