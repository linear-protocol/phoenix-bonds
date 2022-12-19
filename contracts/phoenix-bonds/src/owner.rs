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
