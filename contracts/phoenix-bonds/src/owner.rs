use crate::*;
use near_sdk::{assert_one_yocto, env, near_bindgen};

const ERR_NOT_OWNER: &str = "Not owner";

#[near_bindgen]
impl PhoenixBonds {
    #[payable]
    pub fn change_owner(&mut self, new_owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = new_owner_id;
    }

    #[payable]
    pub fn set_tau(&mut self, new_tau: BasisPoint) {
        assert_tau(new_tau);
        self.assert_owner();
        self.tau = new_tau;
    }

    #[payable]
    pub fn pause(&mut self) {
        self.assert_owner();
        require!(!self.paused, "Already paused");
        self.paused = true;
    }

    #[payable]
    pub fn resume(&mut self) {
        self.assert_owner();
        require!(self.paused, "Not paused");
        self.paused = false;
    }

    #[payable]
    pub fn withdraw_treasury(&mut self) -> Promise {
        self.assert_owner();
        require!(self.treasury_pool_near_amount > 0, "Nothing to withdraw");
        require!(
            env::prepaid_gas() >= GAS_WITHDRAW + GAS_WITHDRAW_CALLBACK + GAS_GET_LINEAR_PRICE, // 160 Tgas
            ERR_NOT_ENOUGH_GAS
        );

        self.get_linear_price().then(
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_WITHDRAW_CALLBACK)
                .on_get_linear_price_for_withdraw(),
        )
    }

    #[private]
    pub fn on_get_linear_price_for_withdraw(
        &mut self,
        #[callback_result] linear_price: Result<U128, PromiseError>,
    ) -> Promise {
        let linear_price = linear_price.expect(ERR_GET_LINEAR_PRICE);
        let near_amount = self.treasury_pool_near_amount;
        require!(near_amount > 0, "Nothing to withdraw");
        // Due to precision, the calculated withdrawn amount can be slightly more than the actual balance,
        // use `min` here to avoid subtraction overflow
        let linear_amount = min(
            near2linear(near_amount, linear_price.0),
            self.linear_balance,
        );

        self.treasury_pool_near_amount = 0;
        self.linear_balance -= linear_amount;

        ext_fungible_token::ext(self.linear_address.clone())
            .with_static_gas(GAS_FT_TRANSFER)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer(
                self.owner_id.clone(),
                linear_amount.into(),
                Some("Treasury withdraw".to_string()),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FT_TRANSFER_CALLBACK)
                    .on_treasury_withdrawn(near_amount.into(), linear_amount.into()),
            )
    }

    #[private]
    pub fn on_treasury_withdrawn(&mut self, near_amount: U128, linear_amount: U128) {
        if is_promise_success() {
            Event::TreasuryWithdrawn {
                near_amount,
                linear_amount,
            }
            .emit();
        } else {
            self.treasury_pool_near_amount += near_amount.0;
            self.linear_balance += linear_amount.0;
        }
    }
}

impl PhoenixBonds {
    pub(crate) fn assert_owner(&self) {
        assert_one_yocto();
        require!(
            env::predecessor_account_id() == self.owner_id,
            ERR_NOT_OWNER
        );
    }
}
