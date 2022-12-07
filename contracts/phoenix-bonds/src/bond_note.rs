use std::convert::TryInto;

use crate::{
    active_vector::{Active, ActiveVector},
    types::{Duration, StorageKey, Timestamp},
    PhoenixBond, *,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
    json_types::U128,
    near_bindgen, require,
    serde::Serialize,
    AccountId, Balance, PanicOnDefault,
};

pub const ERR_BOND_NOTE_NOT_EXIST: &str = "Bond note doesn't exist";
pub const ERR_BOND_WRONG_STATE_TO_CANCEL: &str = "Bond in wrong state to cancel";
pub const ERR_BOND_WRONG_STATE_TO_COMMIT: &str = "Bond in wrong state to commit";
pub const ERR_WRONG_TIMESTAMP: &str = "Wrong timestamp when computing note length";

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum BondStatus {
    Pending,
    Committed,
    Cancelled,
}

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct BondNote {
    id: u32,
    account_id: AccountId,
    bond_amount: Balance,
    committed_pnear_amount: Balance,
    created_at: Timestamp,
    settled_at: Timestamp,
    status: BondStatus,
}

impl BondNote {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn bond_amount(&self) -> Balance {
        self.bond_amount
    }

    pub fn status(&self) -> BondStatus {
        self.status.clone()
    }

    pub fn length(&self, ts: Timestamp) -> Duration {
        require!(ts >= self.created_at, ERR_WRONG_TIMESTAMP);
        match self.status {
            BondStatus::Pending => ts - self.created_at,
            _ => self.settled_at - self.created_at,
        }
    }

    pub fn cancel(&mut self) {
        require!(
            self.status == BondStatus::Pending,
            ERR_BOND_WRONG_STATE_TO_CANCEL
        );
        self.status = BondStatus::Cancelled;
        self.settled_at = current_timestamp_ms();
    }

    pub fn commit(&mut self, pnear_amount: Balance) {
        require!(
            self.status == BondStatus::Pending,
            ERR_BOND_WRONG_STATE_TO_COMMIT
        );
        self.status = BondStatus::Committed;
        self.committed_pnear_amount = pnear_amount;
        self.settled_at = current_timestamp_ms();
    }
}

impl Active for BondNote {
    fn is_active(&self) -> bool {
        self.status == BondStatus::Pending
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct BondNotes {
    notes: LookupMap<AccountId, ActiveVector<BondNote>>,
}

impl BondNotes {
    pub fn new() -> Self {
        Self {
            notes: LookupMap::new(StorageKey::BondNotes),
        }
    }

    pub fn get_user_note(&self, account_id: &AccountId, note_id: u32) -> BondNote {
        self.notes
            .get(account_id)
            .expect(ERR_BOND_NOTE_NOT_EXIST)
            .get(note_id)
            .expect(ERR_BOND_NOTE_NOT_EXIST)
            .clone()
    }

    pub fn user_note_len(&self, account_id: &AccountId) -> u32 {
        self.notes.get(account_id).map(|v| v.len()).unwrap_or(0)
    }

    pub fn user_pending_note_len(&self, account_id: &AccountId) -> u32 {
        self.notes
            .get(account_id)
            .map(|v| v.active_len())
            .unwrap_or(0)
    }

    pub fn get_user_pending_note_ids(&self, account_id: &AccountId) -> Vec<u32> {
        self.notes
            .get(account_id)
            .map(|v| v.get_active_item_indices())
            .unwrap_or_default()
    }

    pub fn insert_new_note(&mut self, account_id: &AccountId, bond_amount: Balance) -> BondNote {
        let mut user_notes = self
            .notes
            .get(account_id)
            .unwrap_or_else(|| ActiveVector::new(StorageKey::UserNotes(account_id.clone())));

        let note_id = user_notes.len();
        let note = BondNote {
            id: note_id,
            account_id: account_id.clone(),
            bond_amount,
            committed_pnear_amount: 0,
            created_at: current_timestamp_ms(),
            settled_at: 0,
            status: BondStatus::Pending,
        };

        user_notes.append(note.clone());
        self.notes.insert(account_id, &user_notes);

        note
    }

    pub fn save_user_note(&mut self, account_id: &AccountId, note_id: u32, bond_note: BondNote) {
        let mut user_notes = self.notes.get(account_id).unwrap();
        user_notes.update(note_id, bond_note);
        self.notes.insert(account_id, &user_notes);
    }
}

// === contract view functions for bond note

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BondNoteInfo {
    id: u32,
    account_id: AccountId,
    #[serde(with = "u128_dec_format")]
    bond_amount: Balance,
    #[serde(with = "u128_dec_format")]
    committed_pnear_amount: Balance,
    created_at: Timestamp,
    settled_at: Timestamp,
    status: BondStatus,

    #[serde(with = "u128_dec_format")]
    cap: Balance,
    #[serde(with = "u128_dec_format")]
    accrued_pnear: Balance,
}

impl PhoenixBond {
    pub(crate) fn build_note_info(&self, note: &BondNote, linear_price: Balance) -> BondNoteInfo {
        BondNoteInfo {
            id: note.id,
            account_id: note.account_id.clone(),
            bond_amount: note.bond_amount,
            committed_pnear_amount: note.committed_pnear_amount,
            created_at: note.created_at,
            settled_at: note.settled_at,
            status: note.status.clone(),
            cap: self.note_cap(note, linear_price),
            accrued_pnear: self.note_accrued_pnear(note, linear_price, current_timestamp_ms()),
        }
    }
}

#[near_bindgen]
impl PhoenixBond {
    pub fn get_bond_note(
        &self,
        account_id: AccountId,
        note_id: u32,
        linear_price: U128,
    ) -> BondNoteInfo {
        let note = self.bond_notes.get_user_note(&account_id, note_id);
        self.build_note_info(&note, linear_price.0)
    }

    pub fn notes_count(&self, account_id: AccountId) -> u32 {
        self.bond_notes.user_note_len(&account_id)
    }

    pub fn pending_notes_count(&self, account_id: AccountId) -> u32 {
        self.bond_notes.user_pending_note_len(&account_id)
    }

    pub fn list_pending_notes(
        &self,
        account_id: AccountId,
        linear_price: U128,
        offset: u32,
        limit: u32,
    ) -> Vec<BondNoteInfo> {
        self.bond_notes
            .get_user_pending_note_ids(&account_id)
            .iter()
            .skip(offset.try_into().unwrap())
            .take(limit.try_into().unwrap())
            .map(|id| self.bond_notes.get_user_note(&account_id, *id))
            .map(|note| self.build_note_info(&note, linear_price.0))
            .collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn new_note(bond_amount: Balance, created_at: Timestamp) -> BondNote {
        BondNote {
            id: 0,
            account_id: AccountId::new_unchecked("foo".into()),
            bond_amount,
            committed_pnear_amount: 0,
            created_at,
            settled_at: 0,
            status: BondStatus::Pending,
        }
    }
}
