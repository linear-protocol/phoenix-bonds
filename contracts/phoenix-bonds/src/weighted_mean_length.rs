use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, require, Balance, PanicOnDefault,
};

use crate::types::{Duration, Timestamp};

const ERR_BAD_TIMESTAMP: &str = "Bad timestamp for computing mean";

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct WeightedMeanLength {
    /// sum of (bond amout * bond length) for all pending bond
    weighted_sum: u128,
    /// sum of bond amount for all pending bond
    total_weight: u128,
    /// last update timestamp
    updated_at: Timestamp,
}

impl WeightedMeanLength {
    pub fn new() -> Self {
        Self {
            weighted_sum: 0,
            total_weight: 0,
            updated_at: 0,
        }
    }

    /// Get volume weighted mean bonding length in ms at given timestamp
    pub fn mean(&self, ts: Timestamp) -> u128 {
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

    pub fn insert(&mut self, amount: Balance, ts: Timestamp) {
        self.update(ts);
        self.total_weight += amount;
    }

    pub fn remove(&mut self, amount: Balance, length: Duration, ts: Timestamp) {
        self.update(ts);
        self.weighted_sum -= amount * length as u128;
        self.total_weight -= amount;
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::ONE_NEAR;

    use crate::utils::tests::ONE_DAY_MS;

    use super::*;

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
}
