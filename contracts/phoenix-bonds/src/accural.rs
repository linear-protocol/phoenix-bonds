use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    require, Balance, PanicOnDefault,
};
use std::{
    cmp::{max, min},
    convert::TryInto,
};

use crate::{
    types::{BasisPoint, Duration, Timestamp, FULL_BASIS_POINT},
    utils::{apply_basis_point, current_timestamp_ms},
};

const ERR_BAD_MIN_ALPHA: &str = "Min alpha lower init alpha";
const ERR_BAD_TIMESTAMP: &str = "Bad timestamp for computing mean";

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AccuralParameter {
    alpha: u64,
    min_alpha: u64,
    target_mean_length: u64,
    adjust_interval: Duration,
    adjust_rate: BasisPoint,
    last_updated_at: Timestamp,
    mean_length: WeightedMeanLength,
}

impl AccuralParameter {
    pub fn new(
        init_alpha: u64,
        min_alpha: u64,
        target_mean_length: u64,
        adjust_interval: Duration,
        adjust_rate: BasisPoint,
    ) -> AccuralParameter {
        require!(init_alpha >= min_alpha, ERR_BAD_MIN_ALPHA);

        Self {
            alpha: init_alpha,
            min_alpha,
            target_mean_length,
            adjust_interval,
            adjust_rate,
            last_updated_at: current_timestamp_ms(),
            mean_length: WeightedMeanLength::new(),
        }
    }

    pub fn current_alpha(&self, ts: Timestamp) -> u64 {
        let current_mean_length: u64 = self.mean_length.mean(ts).try_into().unwrap();
        if current_mean_length < self.target_mean_length {
            self.alpha
        } else {
            // how long has the mean length exceeded target value
            let exceed_length = min(
                current_mean_length - self.target_mean_length,
                ts - self.last_updated_at,
            );
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

        if old_mean_length >= self.target_mean_length as u128
            && new_mean_length < self.target_mean_length as u128
        {
            self.alpha = alpha_before_insertion;
            self.last_updated_at = ts;
        }
    }

    pub fn weighted_mean_remove(&mut self, amount: Balance, length: Duration, ts: Timestamp) {
        let alpha_before_removal = self.current_alpha(ts);

        let old_mean_length = self.mean_length.mean(ts);
        self.mean_length.remove(amount, length, ts);
        let new_mean_length = self.mean_length.mean(ts);

        if (old_mean_length >= self.target_mean_length as u128
            && new_mean_length < self.target_mean_length as u128)
            || (old_mean_length < self.target_mean_length as u128
                && new_mean_length >= self.target_mean_length as u128)
        {
            self.alpha = alpha_before_removal;
            self.last_updated_at = ts;
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
struct WeightedMeanLength {
    /// sum of (bond amout * bond length) for all pending bond
    weighted_sum: u128,
    /// sum of bond amount for all pending bond
    total_weight: u128,
    /// last update timestamp
    updated_at: Timestamp,
}

impl WeightedMeanLength {
    fn new() -> Self {
        Self {
            weighted_sum: 0,
            total_weight: 0,
            updated_at: 0,
        }
    }

    /// Get volume weighted mean bonding length in ms at given timestamp
    fn mean(&self, ts: Timestamp) -> u128 {
        require!(ts >= self.updated_at, ERR_BAD_TIMESTAMP);
        if self.total_weight == 0 {
            return 0;
        }
        self.weighted_sum / self.total_weight + (ts - self.updated_at) as u128
    }

    fn update(&mut self, ts: Timestamp) {
        let expected_mean = self.mean(ts);
        self.weighted_sum = expected_mean * self.total_weight;
        self.updated_at = ts;
    }

    fn insert(&mut self, amount: Balance, ts: Timestamp) {
        self.update(ts);
        self.total_weight += amount;
    }

    fn remove(&mut self, amount: Balance, length: Duration, ts: Timestamp) {
        self.update(ts);
        self.weighted_sum -= amount * length as u128;
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
        assert_eq!(va.mean(ts), ONE_DAY_MS as u128);

        va.insert(1000 * ONE_NEAR, ts);

        // - va should be updated to 3/4 (0.75) day
        assert_eq!(va.mean(ts), 3 * ONE_DAY_MS as u128 / 4);

        // ========== Day 6 ==========
        // - current va should be 19/4 (4.75) days
        // - alice bonds 2000
        let ts = 6 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 19 * ONE_DAY_MS as u128 / 4);

        va.insert(2000 * ONE_NEAR, ts);

        // - va should be updated to 19/6 (3.167) days
        assert_eq!(va.mean(ts), 19 * ONE_DAY_MS as u128 / 6);

        // ========== Day 11 ==========
        // - current va should be 49/6 (8.167) days
        // - bob cancels 2000, length is 10 days
        let ts = 11 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 49 * ONE_DAY_MS as u128 / 6);

        va.remove(2000 * ONE_NEAR, 10 * ONE_DAY_MS, ts);

        // - va should be updated to 29/4 (7.25) days
        assert_eq!(va.mean(ts), 29 * ONE_DAY_MS as u128 / 4);

        // ========== Day 16 ==========
        // - current va should be 49/4 (12.25) days
        // - alice commits 1000, length 15 days
        let ts = 16 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 49 * ONE_DAY_MS as u128 / 4);

        va.remove(1000 * ONE_NEAR, 15 * ONE_DAY_MS, ts);

        // - va should be updated to 34/3 (11.33) days
        assert_eq!(va.mean(ts), 34 * ONE_DAY_MS as u128 / 3);

        // ========== Day 21 ==========
        // - current va should be 49/3 (16.33) days
        let ts = 21 * ONE_DAY_MS;
        assert_eq!(va.mean(ts), 49 * ONE_DAY_MS as u128 / 3);
    }

    fn prepare_accural_param() -> AccuralParameter {
        AccuralParameter::new(
            INIT_ALPHA,
            0,
            15 * ONE_DAY_MS,
            ONE_DAY_MS,
            100, // 1%
        )
    }

    #[test]
    fn test_accural_param_basic() {
        let mut accural = prepare_accural_param();

        // first insert 100 near at day 0
        accural.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 1
        let ts = ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);

        // day 14
        let ts = 14 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);

        // day 15
        let ts = 15 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);

        // day 16
        let ts = 16 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA * 99 / 100);

        // day 18
        let ts = 18 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 251501500); // 3 days * 0.99^3
        accural.weighted_mean_insert(100 * ONE_NEAR, ts); // insert another 100 near at day 18, the mean length should be 9 days now

        // day 19
        let ts = 19 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 251501500);

        // day 20
        let ts = 20 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 251501500);
        // remove the second 100 near at day 20, this makes the mean length 20 days
        accural.weighted_mean_remove(100 * ONE_NEAR, 2 * ONE_DAY_MS, ts);
        // alpha won't be affected immediately
        assert_eq!(accural.current_alpha(ts), 251501500);

        // day 21
        let ts = 21 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 251501500 * 99 / 100);
    }

    #[test]
    fn test_alpha_when_mean_length_below_target() {
        let mut accural = prepare_accural_param();

        // first insert 100 near at day 0
        accural.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 14
        let ts = 14 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);
        // insert another 100 near
        accural.weighted_mean_insert(100 * ONE_NEAR, ts);
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);

        // day 22
        let ts = 22 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);
        // remove first 100 near, whose length is 22 days
        accural.weighted_mean_remove(100 * ONE_NEAR, 22 * ONE_DAY_MS, ts);
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);

        // day 29
        let ts = 29 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);
    }

    #[test]
    fn test_alpha_when_mean_length_above_target() {
        let mut accural = prepare_accural_param();

        // first insert 100 near at day 0
        accural.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 16
        let ts = 16 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 99 * INIT_ALPHA / 100);

        // day 17.5
        let ts = 17 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 254041920); // INIT_ALPHA * 0.99^2
                                                          // insert 1 near
        accural.weighted_mean_insert(ONE_NEAR, ts);
        assert_eq!(accural.current_alpha(ts), 254041920); // INIT_ALPHA * 0.99^2

        // day 20.5
        let ts = 20 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 246496620); // INIT_ALPHA * 0.99^5
                                                          // remove second 1 near
        accural.weighted_mean_remove(ONE_NEAR, 3 * ONE_DAY_MS, ts);
        assert_eq!(accural.current_alpha(ts), 246496620); // INIT_ALPHA * 0.99^5

        // day 21
        let ts = 21 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 244031653); // INIT_ALPHA * 0.99^6
    }

    #[test]
    fn test_alpha_when_mean_length_increased_above_target() {
        let mut accural = prepare_accural_param();

        // first insert 100 near at day 0
        accural.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 14, insert another 100 near
        let ts = 14 * ONE_DAY_MS;
        accural.weighted_mean_insert(100 * ONE_NEAR, ts);

        // day 16, remove second 100 near
        let ts = 16 * ONE_DAY_MS;
        accural.weighted_mean_remove(100 * ONE_NEAR, 2 * ONE_DAY_MS, ts);
        assert_eq!(accural.current_alpha(ts), INIT_ALPHA);

        // day 17
        let ts = 17 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 99 * INIT_ALPHA / 100); // remember the target was exceeded at day 16
    }

    #[test]
    fn test_alpha_when_mean_length_decreased_below_target() {
        let mut accural = prepare_accural_param();

        // first insert 100 near at day 0
        accural.weighted_mean_insert(100 * ONE_NEAR, 0);

        // day 12, insert another 1 near
        let ts = 12 * ONE_DAY_MS;
        accural.weighted_mean_insert(ONE_NEAR, ts);

        // day 20.5, current mean length is 20.38 days
        let ts = 20 * ONE_DAY_MS + HALF_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 246496620); // init_alpha * 0.99^5
                                                          // remove first 100 near
        accural.weighted_mean_remove(100 * ONE_NEAR, 20 * ONE_DAY_MS + HALF_DAY_MS, ts);

        // day 23
        let ts = 23 * ONE_DAY_MS;
        assert_eq!(accural.current_alpha(ts), 246496620); // should be same as day 20.5, when mean length decreased below target
    }
}
