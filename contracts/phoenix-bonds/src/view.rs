use near_sdk::{near_bindgen, serde::Serialize};

use crate::*;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Summary {
    linear_balance: U128,
    reserve_pool_near_amount: U128,
    pending_pool_near_amount: U128,
    permanent_pool_near_amount: U128,
    treasury_pool_near_amount: U128,
    bootstrap_ends_at: Timestamp,
    tau: BasisPoint,
    alpha: Duration,
}

#[near_bindgen]
impl PhoenixBonds {
    pub fn get_summary(&self, linear_price: U128) -> Summary {
        Summary {
            linear_balance: self.linear_balance.into(),
            reserve_pool_near_amount: self.reserve_pool_near_amount(linear_price.0).into(),
            pending_pool_near_amount: self.pending_pool_near_amount.into(),
            permanent_pool_near_amount: self.permanent_pool_near_amount.into(),
            treasury_pool_near_amount: self.treasury_pool_near_amount.into(),
            bootstrap_ends_at: self.bootstrap_ends_at,
            tau: self.tau,
            alpha: self.accrual_param.current_alpha(current_timestamp_ms()),
        }
    }
}
