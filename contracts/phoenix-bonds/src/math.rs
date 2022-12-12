use crate::{
    types::{Timestamp, ONE_PNEAR},
    utils::linear2near,
    *,
};
use near_bigdecimal::*;

impl PhoenixBonds {
    pub(crate) fn reserve_pool_near_amount(&self, linear_price: Balance) -> Balance {
        let protocol_owned_near_amount = linear2near(self.linear_balance, linear_price);
        protocol_owned_near_amount
            - self.pending_pool_near_amount
            - self.permanent_pool_near_amount
            - self.treasury_pool_near_amount
    }

    pub(crate) fn pnear_price(&self, linear_price: Balance) -> Balance {
        if self.pnear_total_supply() == 0 {
            return ONE_PNEAR;
        }

        (BigDecimal::from(self.reserve_pool_near_amount(linear_price)) * ONE_PNEAR.into()
            / self.pnear_total_supply().into())
        .round_u128()
    }

    pub(crate) fn accrued_amount(
        &self,
        value: Balance,
        length: Duration,
        current_timestamp: Timestamp,
    ) -> Balance {
        let alpha = self.accural_param.current_alpha(current_timestamp);
        (BigDecimal::from(value) * length.into() / (length + alpha).into()).round_u128()
    }

    /// Cap of pNEAR that a bond note is worth.
    /// Note that it's meaningless to call this on a committed/cancelled note.
    pub(crate) fn note_cap(&self, note: &BondNote, linear_price: Balance) -> Balance {
        let bond_amount = note.bond_amount();
        let amount_to_treasury = apply_basis_point(bond_amount, self.tau);
        near2pnear(
            bond_amount - amount_to_treasury,
            self.pnear_price(linear_price),
        )
    }

    /// How many pNEAR can a bond note get if committed now
    pub(crate) fn note_accrued_pnear(
        &self,
        note: &BondNote,
        linear_price: Balance,
        current_timestamp: Timestamp,
    ) -> Balance {
        let cap = self.note_cap(note, linear_price);
        self.accrued_amount(cap, note.length(current_timestamp), current_timestamp)
    }
}

// -- Public view methods

#[near_bindgen]
impl PhoenixBonds {
    pub fn get_pnear_price(&self, linear_price: U128) -> U128 {
        self.pnear_price(linear_price.0).into()
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::ONE_NEAR;

    use crate::{bond_note::tests::new_note, tests::new_contract, utils::tests::ONE_DAY_MS};

    use super::*;

    const ONE_LINEAR: Balance = ONE_NEAR;
    const ONE_PNEAR: Balance = ONE_NEAR;

    fn alice() -> AccountId {
        AccountId::new_unchecked("alice".into())
    }

    #[test]
    fn test_reserve_pool_near_amount() {
        let contract = new_contract(
            1_000_000 * ONE_LINEAR,
            900_000 * ONE_NEAR, // pending
            30_000 * ONE_NEAR,  // permanent
            10_000 * ONE_NEAR,  // treasury
            0,
            0,
        );
        let linear_price = 6 * ONE_NEAR / 5; // 1.2

        assert_eq!(
            contract.reserve_pool_near_amount(linear_price),
            260_000 * ONE_NEAR
        );
    }

    #[test]
    fn test_pnear_price() {
        let mut contract = new_contract(
            1_000_000 * ONE_LINEAR,
            900_000 * ONE_NEAR, // pending
            30_000 * ONE_NEAR,  // permanent
            10_000 * ONE_NEAR,  // treasury
            0,
            0,
        );
        let linear_price = 6 * ONE_NEAR / 5; // 1.2

        // pnear price is 1 when total supply is zero
        assert_eq!(contract.pnear_price(linear_price), ONE_NEAR);

        // mint some pnear
        contract.mint_pnear(&alice(), 130_000 * ONE_PNEAR, None);

        assert_eq!(contract.pnear_price(linear_price), 2 * ONE_NEAR);
    }

    #[test]
    fn test_accrued_amount() {
        // this is effectively to test y = x * t / (t + a)

        let alpha = 30 * ONE_DAY_MS;
        let contract = new_contract(0, 0, 0, 0, alpha, 0);

        let t = 0;
        assert_eq!(contract.accrued_amount(ONE_PNEAR, t, 0), 0);

        let t = ONE_DAY_MS;
        assert_eq!(
            contract.accrued_amount(ONE_PNEAR, t, 0),
            32258064516129032258065
        );

        let t = 15 * ONE_DAY_MS;
        assert_eq!(
            contract.accrued_amount(ONE_PNEAR, t, 0),
            333333333333333333333333
        );

        let t = 30 * ONE_DAY_MS;
        assert_eq!(contract.accrued_amount(ONE_PNEAR, t, 0), ONE_PNEAR / 2);

        let t = 60 * ONE_DAY_MS;
        assert_eq!(
            contract.accrued_amount(ONE_PNEAR, t, 0),
            666666666666666666666667
        );
    }

    #[test]
    fn test_note_cap() {
        let mut contract = new_contract(
            1_000_000 * ONE_LINEAR,
            900_000 * ONE_NEAR, // pending
            30_000 * ONE_NEAR,  // permanent
            10_000 * ONE_NEAR,  // treasury
            0,
            0,
        );
        contract.mint_pnear(&alice(), 60_000 * ONE_PNEAR, None);
        let note = new_note(1000 * ONE_NEAR, ONE_DAY_MS);

        let linear_price = ONE_NEAR; // pnear price will be 1
        assert_eq!(contract.note_cap(&note, linear_price), note.bond_amount());

        let linear_price = 6 * ONE_NEAR / 5; // pnear price will be 4.33
        assert_eq!(
            contract.note_cap(&note, linear_price),
            230769230769230769230769249
        );
    }

    #[test]
    fn test_note_accrued_pnear() {
        let mut contract = new_contract(
            1_000_000 * ONE_LINEAR,
            900_000 * ONE_NEAR, // pending
            30_000 * ONE_NEAR,  // permanent
            10_000 * ONE_NEAR,  // treasury
            30 * ONE_DAY_MS,
            0,
        );
        contract.mint_pnear(&alice(), 60_000 * ONE_PNEAR, None);
        let bond_time = ONE_DAY_MS;
        let note = new_note(1000 * ONE_NEAR, bond_time);

        let linear_price = ONE_NEAR; // pnear price will be 1, cap is 1000 pnear
        let current_time = 11 * ONE_DAY_MS;
        assert_eq!(
            contract.note_accrued_pnear(&note, linear_price, current_time),
            250 * ONE_PNEAR
        );

        let linear_price = 6 * ONE_NEAR / 5; // pnear price will be 4.33, cap is 230.77 pnear
        let current_time = 30 * ONE_DAY_MS;
        assert_eq!(
            contract.note_accrued_pnear(&note, linear_price, current_time),
            113428943937418513689700139
        );
    }
}
