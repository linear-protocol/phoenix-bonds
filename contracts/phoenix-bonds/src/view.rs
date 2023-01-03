use near_sdk::{near_bindgen, serde::Serialize};

use crate::*;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccrualInfo {
    alpha: Duration,
    min_alpha: Duration,
    adjust_interval: Duration,
    adjust_rate: BasisPoint,
    decreasing: bool,
    target_mean_length: Duration,
    current_mean_length: Duration,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Summary {
    owner_id: AccountId,
    linear_balance: U128,
    reserve_pool_near_amount: U128,
    pending_pool_near_amount: U128,
    permanent_pool_near_amount: U128,
    treasury_pool_near_amount: U128,
    bootstrap_ends_at: Timestamp,
    tau: BasisPoint,
    accrual_parameter: AccrualInfo,
}

#[near_bindgen]
impl PhoenixBonds {
    pub fn get_summary(&self, linear_price: U128) -> Summary {
        let current_ms = current_timestamp_ms();
        Summary {
            owner_id: self.owner_id.clone(),
            linear_balance: self.linear_balance.into(),
            reserve_pool_near_amount: self.reserve_pool_near_amount(linear_price.0).into(),
            pending_pool_near_amount: self.pending_pool_near_amount.into(),
            permanent_pool_near_amount: self.permanent_pool_near_amount.into(),
            treasury_pool_near_amount: self.treasury_pool_near_amount.into(),
            bootstrap_ends_at: self.bootstrap_ends_at,
            tau: self.tau,
            accrual_parameter: AccrualInfo {
                alpha: self.accrual_param.current_alpha(current_ms),
                min_alpha: self.accrual_param.min_alpha,
                adjust_interval: self.accrual_param.adjust_interval,
                adjust_rate: self.accrual_param.adjust_rate,
                decreasing: self.accrual_param.mean_length.mean(current_ms)
                    > self.accrual_param.target_mean_length,
                target_mean_length: self.accrual_param.target_mean_length,
                current_mean_length: self.accrual_param.mean_length.mean(current_ms),
            },
        }
    }

    /// Returns the expected alpha at a future timestamp
    pub fn get_alpha(&self, timestamp: Timestamp) -> Duration {
        self.accrual_param.current_alpha(timestamp)
    }
}
