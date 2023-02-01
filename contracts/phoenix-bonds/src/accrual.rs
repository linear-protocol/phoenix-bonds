use near_bigdecimal::BigDecimal;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    require,
    serde::{Deserialize, Serialize},
    Balance, PanicOnDefault,
};
use std::{cmp::max, convert::TryInto};

use crate::{
    types::{BasisPoint, Duration, Timestamp, FULL_BASIS_POINT},
    utils::apply_basis_point,
};

const ERR_BAD_MIN_ALPHA: &str = "Min alpha cannot be 0";
const ERR_BAD_ALPHA: &str = "Alpha cannot be lower than min alpha";
const ERR_BAD_TARGET_MEAN_LENGTH: &str = "Target mean length cannot be 0";
const ERR_BAD_ADJUST_INTERVAL: &str = "Adjust interval cannot be 0";
const ERR_BAD_ADJUST_RATE: &str = "Adjust rate must be less than 10000";
const ERR_BAD_TIMESTAMP: &str = "Bad timestamp for computing mean";

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccrualConfig {
    pub alpha: Duration,
    pub min_alpha: Duration,
    pub target_mean_length: Duration,
    pub adjust_interval: Duration,
    pub adjust_rate: BasisPoint,
}

impl AccrualConfig {
    pub fn assert_valid(&self) {
        require!(self.min_alpha > 0, ERR_BAD_MIN_ALPHA);
        require!(self.alpha >= self.min_alpha, ERR_BAD_ALPHA);
        require!(self.target_mean_length > 0, ERR_BAD_TARGET_MEAN_LENGTH);
        require!(self.adjust_interval > 0, ERR_BAD_ADJUST_INTERVAL);
        require!(self.adjust_rate < FULL_BASIS_POINT, ERR_BAD_ADJUST_RATE);
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AccrualParameter {
    /// last updated alpha value
    pub alpha: Duration,
    /// minimum value that alpha could decrease to
    pub min_alpha: Duration,
    /// target weighted mean length of all pending bonds
    pub target_mean_length: Duration,
    /// alpha decreases each interval after mean length exceeds target
    pub adjust_interval: Duration,
    /// how much should alpha decrease in each interval
    pub adjust_rate: BasisPoint,
    /// when did mean length exceed target, 0 means mean length was below target
    pub exceeds_target_at: Timestamp,
    /// volume weighted mean bonding length
    pub mean_length: WeightedMeanLength,
}

impl AccrualParameter {
    pub fn new(
        init_alpha: Duration,
        min_alpha: Duration,
        target_mean_length: Duration,
        adjust_interval: Duration,
        adjust_rate: BasisPoint,
    ) -> AccrualParameter {
        Self {
            alpha: init_alpha,
            min_alpha,
            target_mean_length,
            adjust_interval,
            adjust_rate,
            exceeds_target_at: 0,
            mean_length: WeightedMeanLength::new(),
        }
    }

    pub fn current_alpha(&self, ts: Timestamp) -> Duration {
        let current_mean_length = self.mean_length.mean(ts);
        if current_mean_length < self.target_mean_length {
            self.alpha
        } else {
            // how long has the mean length exceeded target value
            let exceed_length = if self.exceeds_target_at == 0 {
                current_mean_length - self.target_mean_length
            } else {
                ts - self.exceeds_target_at
            };
            // how many adjustments shall be made
            let num_adjustments = exceed_length / self.adjust_interval;

            let mut adjusted = self.alpha;
            for _ in 0..num_adjustments {
                adjusted = apply_basis_point(adjusted, FULL_BASIS_POINT - self.adjust_rate);
            }

            max(self.min_alpha, adjusted)
        }
    }

    pub fn weighted_mean_insert(&mut self, amount: Balance, ts: Timestamp) {
        let alpha_before_insertion = self.current_alpha(ts);

        let old_mean_length = self.mean_length.mean(ts);
        self.mean_length.insert(amount, ts); // this could only make mean length shorter
        let new_mean_length = self.mean_length.mean(ts);

        // if mean length drops below target, reset exceeds_target_at
        if old_mean_length > self.target_mean_length && new_mean_length < self.target_mean_length {
            self.alpha = alpha_before_insertion;
            self.exceeds_target_at = 0;
        }
        // if mean length is above target but exceeds_target_at is 0
        // this means the mean length grows above target naturally, we'll need to find the
        // correct value for exceeds_target_at.
        if self.exceeds_target_at == 0 && new_mean_length > self.target_mean_length {
            self.exceeds_target_at = ts - (old_mean_length - self.target_mean_length);
        }
    }

    pub fn weighted_mean_remove(&mut self, amount: Balance, length: Duration, ts: Timestamp) {
        let alpha_before_removal = self.current_alpha(ts);

        let old_mean_length = self.mean_length.mean(ts);
        self.mean_length.remove(amount, length, ts);
        let new_mean_length = self.mean_length.mean(ts);

        // if mean length drops below target, reset exceeds_target_at
        if old_mean_length > self.target_mean_length && new_mean_length < self.target_mean_length {
            self.alpha = alpha_before_removal;
            self.exceeds_target_at = 0;
        }
        if new_mean_length > self.target_mean_length {
            // if this action makes the mean length above target, then exceeds_target_at should be now
            if old_mean_length < self.target_mean_length {
                self.exceeds_target_at = ts;
            }
            // the mean length grows above target naturally
            // need to find the correct exceeds_target_at
            if self.exceeds_target_at == 0 {
                self.exceeds_target_at = ts - (old_mean_length - self.target_mean_length);
            }
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct WeightedMeanLength {
    /// sum of (bond amount * bond length) for all pending bonds
    weighted_sum: BigDecimal,
    /// sum of bond amount for all pending bonds
    total_weight: u128,
    /// last update timestamp
    updated_at: Timestamp,
}

impl WeightedMeanLength {
    fn new() -> Self {
        Self {
            weighted_sum: BigDecimal::from(0u128),
            total_weight: 0,
            updated_at: 0,
        }
    }

    /// Get volume weighted mean bonding length in ms at given timestamp
    pub fn mean(&self, ts: Timestamp) -> Duration {
        require!(ts >= self.updated_at, ERR_BAD_TIMESTAMP);
        if self.total_weight == 0 {
            return 0;
        }
        let time_since_last_update = ts - self.updated_at;
        (self.weighted_sum / self.total_weight.into() + time_since_last_update.into())
            .round_u128()
            .try_into()
            .unwrap()
    }

    fn update(&mut self, ts: Timestamp) {
        let expected_mean = self.mean(ts);
        self.weighted_sum = BigDecimal::from(expected_mean) * self.total_weight.into();
        self.updated_at = ts;
    }

    fn insert(&mut self, amount: Balance, ts: Timestamp) {
        self.update(ts);
        self.total_weight += amount;
    }

    fn remove(&mut self, amount: Balance, length: Duration, ts: Timestamp) {
        self.update(ts);
        self.weighted_sum = self.weighted_sum - BigDecimal::from(amount) * length.into();
        self.total_weight -= amount;
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::ONE_NEAR;

    use crate::utils::tests::{HALF_DAY_MS, ONE_DAY_MS};

    use super::*;

    const INIT_ALPHA: u64 = 3 * ONE_DAY_MS;

    #[test]
    fn test_weighted_mean_length() {
        let mut va = WeightedMeanLength::new();

        // ========== Day 1 ==========
        // - alice bonds 1000
        // - bob bonds 2000
        let ts = ONE_DAY_MS;
        va.insert(1000 * ONE_NEAR, ts);
        va.insert(2000 * ONE_NEAR, ts);

        // current va should be 0
        assert_eq!(va.mean(ts), 0);

        // ========== Day 2 ==========
        // - current va should be 1 day
        // - charles bonds 1000
        let ts = 2 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), ONE_DAY_MS);

        va.insert(1000 * ONE_NEAR, ts);

        // - va should be updated to 3/4 (0.75) day
        assert_eq!(va.mean(ts), 3 * ONE_DAY_MS / 4);

        // ========== Day 6 ==========
        // - current va should be 19/4 (4.75) days
        // - alice bonds 2000
        let ts = 6 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 19 * ONE_DAY_MS / 4);

        va.insert(2000 * ONE_NEAR, ts);

        // - va should be updated to 19/6 (3.167) days
        assert_eq!(va.mean(ts), 19 * ONE_DAY_MS / 6);

        // ========== Day 11 ==========
        // - current va should be 49/6 (8.167) days
        // - bob cancels 2000, length is 10 days
        let ts = 11 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 49 * ONE_DAY_MS / 6);

        va.remove(2000 * ONE_NEAR, 10 * ONE_DAY_MS, ts);

        // - va should be updated to 29/4 (7.25) days
        assert_eq!(va.mean(ts), 29 * ONE_DAY_MS / 4);

        // ========== Day 16 ==========
        // - current va should be 49/4 (12.25) days
        // - alice commits 1000, length 15 days
        let ts = 16 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 49 * ONE_DAY_MS / 4);

        va.remove(1000 * ONE_NEAR, 15 * ONE_DAY_MS, ts);

        // - va should be updated to 34/3 (11.33) days
        assert_eq!(va.mean(ts), 34 * ONE_DAY_MS / 3);

        // ========== Day 21 ==========
        // - current va should be 49/3 (16.33) days
        let ts = 21 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 49 * ONE_DAY_MS / 3);
    }

    fn prepare_accrual_param() -> AccrualParameter {
        AccrualParameter::new(
            INIT_ALPHA,
            0,
            15 * ONE_DAY_MS,
            ONE_DAY_MS,
            100, // 1%
        )
    }

    #[test]
    fn test_accrual_param_basic() {
        let mut accrual = prepare_accrual_param();

        // first insert 100 near at day 0
        accrual.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 1
        let ts = ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);

        // day 14
        let ts = 14 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);

        // day 15
        let ts = 15 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);

        // day 16
        let ts = 16 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA * 99 / 100);

        // day 18
        let ts = 18 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 251501500); // 3 days * 0.99^3
        accrual.weighted_mean_insert(100 * ONE_NEAR, ts); // insert another 100 near at day 18, the mean length should be 9 days now

        // day 19
        let ts = 19 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 251501500);

        // day 20
        let ts = 20 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 251501500);
        // remove the second 100 near at day 20, this makes the mean length 20 days
        accrual.weighted_mean_remove(100 * ONE_NEAR, 2 * ONE_DAY_MS, ts);
        // alpha won't be affected immediately
        assert_eq!(accrual.current_alpha(ts), 251501500);

        // day 21
        let ts = 21 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 251501500 * 99 / 100);
    }

    #[test]
    fn test_alpha_when_mean_length_below_target() {
        let mut accrual = prepare_accrual_param();

        // first insert 100 near at day 0
        accrual.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 14
        let ts = 14 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);
        // insert another 100 near
        accrual.weighted_mean_insert(100 * ONE_NEAR, ts);
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);

        // day 22
        let ts = 22 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);
        // remove first 100 near, whose length is 22 days
        accrual.weighted_mean_remove(100 * ONE_NEAR, 22 * ONE_DAY_MS, ts);
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);

        // day 29
        let ts = 29 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);
    }

    #[test]
    fn test_alpha_when_mean_length_above_target() {
        let mut accrual = prepare_accrual_param();

        // first insert 100 near at day 0
        accrual.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 16
        let ts = 16 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 99 * INIT_ALPHA / 100);

        // day 17.5
        let ts = 17 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 254041920); // INIT_ALPHA * 0.99^2

        // insert 2 near
        accrual.weighted_mean_insert(ONE_NEAR, ts);
        accrual.weighted_mean_insert(ONE_NEAR, ts);
        assert_eq!(accrual.current_alpha(ts), 254041920); // INIT_ALPHA * 0.99^2

        // day 20.5
        let ts = 20 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 246496620); // INIT_ALPHA * 0.99^5
                                                          // remove second 1 near
        accrual.weighted_mean_remove(ONE_NEAR, 3 * ONE_DAY_MS, ts);
        assert_eq!(accrual.current_alpha(ts), 246496620); // INIT_ALPHA * 0.99^5

        // day 21
        let ts = 21 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 244031653); // INIT_ALPHA * 0.99^6

        // day 40
        let ts = 40 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 201611286); // INIT_ALPHA * 0.99^25

        // remove first 100 near, this SHALL NOT affect alpha
        accrual.weighted_mean_remove(100 * ONE_NEAR, 40 * ONE_DAY_MS, ts);
        assert_eq!(accrual.current_alpha(ts), 201611286); // INIT_ALPHA * 0.99^25

        // day 41
        let ts = 41 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 199595173); // INIT_ALPHA * 0.99^26
    }

    #[test]
    fn test_alpha_when_mean_length_increased_above_target() {
        let mut accrual = prepare_accrual_param();

        // first insert 100 near at day 0
        accrual.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 14, insert another 100 near
        let ts = 14 * ONE_DAY_MS;
        accrual.weighted_mean_insert(100 * ONE_NEAR, ts);

        // day 16, remove second 100 near
        let ts = 16 * ONE_DAY_MS;
        accrual.weighted_mean_remove(100 * ONE_NEAR, 2 * ONE_DAY_MS, ts);
        assert_eq!(accrual.current_alpha(ts), INIT_ALPHA);

        // day 17
        let ts = 17 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 99 * INIT_ALPHA / 100); // remember the target was exceeded at day 16
    }

    #[test]
    fn test_alpha_when_mean_length_decreased_below_target() {
        let mut accrual = prepare_accrual_param();

        // first insert 100 near at day 0
        accrual.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 12, insert another 1 near
        let ts = 12 * ONE_DAY_MS;
        accrual.weighted_mean_insert(ONE_NEAR, ts);

        // day 20.5, current mean length is 20.38 days
        let ts = 20 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 246496620); // init_alpha * 0.99^5
                                                          // remove first 100 near
        accrual.weighted_mean_remove(100 * ONE_NEAR, 20 * ONE_DAY_MS + HALF_DAY_MS, ts);

        // day 23
        let ts = 23 * ONE_DAY_MS;
        assert_eq!(accrual.current_alpha(ts), 246496620); // should be same as day 20.5, when mean length decreased below target
    }
}
