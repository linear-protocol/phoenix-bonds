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
    pub fn set_alpha(&mut self, new_alpha: Duration) {
        self.assert_owner();
        self.alpha = new_alpha;
    }

    #[payable]
    pub fn set_tau(&mut self, new_tau: BasisPoint) {
        self.assert_owner();
        self.tau = new_tau;
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
