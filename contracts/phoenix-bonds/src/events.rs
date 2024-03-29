use near_sdk::{json_types::U128, log, AccountId};
use serde::Serialize;
use serde_json::json;

const EVENT_STANDARD: &str = "phoenix_bonds";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[must_use = "Don't forget to `.emit()` this event"]
pub enum Event {
    // user events
    Bond {
        account_id: AccountId,
        note_id: u32,
        bond_amount: U128,
        linear_amount: U128,
    },
    Cancel {
        account_id: AccountId,
        note_id: u32,
        bond_amount: U128,
        refund_linear: U128,
    },
    Commit {
        account_id: AccountId,
        note_id: u32,
        bond_amount: U128,
        pnear_amount: U128,
    },
    Redeem {
        account_id: AccountId,
        pnear_amount: U128,
        redeemed_linear: U128,
    },
    // lost and found events
    LostFoundInsert {
        account_id: AccountId,
        amount: U128,
    },
    LostFoundClaim {
        account_id: AccountId,
        amount: U128,
    },
    // treasury withdraw event
    TreasuryWithdrawn {
        near_amount: U128,
        linear_amount: U128,
    },
}

impl Event {
    pub fn emit(&self) {
        let data = json!(self);
        let event_json = json!({
            "standard": EVENT_STANDARD,
            "version": EVENT_STANDARD_VERSION,
            "event": data["event"],
            "data": [data["data"]]
        })
        .to_string();
        log!("EVENT_JSON:{}", event_json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils;

    fn alice() -> AccountId {
        AccountId::new_unchecked("alice".to_string())
    }

    #[test]
    fn bond() {
        Event::Bond {
            account_id: alice(),
            note_id: 1,
            bond_amount: U128(1000),
            linear_amount: U128(1000),
        }
        .emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"standard":"phoenix_bonds","version":"1.0.0","event":"bond","data":[{"account_id":"alice","note_id":1,"bond_amount":"1000","linear_amount":"1000"}]}"#
        );
    }
}
