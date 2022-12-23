use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    store::{UnorderedSet, Vector},
    IntoStorageKey,
};
use std::cmp::min;

pub trait Active {
    fn is_active(&self) -> bool;
}

/// Maintains a list of items as well as a set
/// of active items.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ActiveVector<T: Active + BorshSerialize + BorshDeserialize> {
    /// all items
    items: Vector<T>,
    /// active item indexes
    active_items: UnorderedSet<u32>,
}

impl<T: Active + BorshSerialize + BorshDeserialize> ActiveVector<T> {
    pub fn new<S>(items_key: S) -> Self
    where
        S: IntoStorageKey,
    {
        let item_prefix = items_key.into_storage_key();
        let items_prefix = item_prefix.as_slice();
        let active_prefix = [items_prefix, "_active".as_bytes()].concat();
        Self {
            items: Vector::new(items_prefix),
            active_items: UnorderedSet::new(active_prefix),
        }
    }

    pub fn append(&mut self, item: T) -> u32 {
        let index = self.items.len();
        if item.is_active() {
            self.active_items.insert(index);
        }

        self.items.push(item);

        index
    }

    pub fn len(&self) -> u32 {
        self.items.len()
    }

    pub fn active_len(&self) -> u32 {
        self.active_items.len()
    }

    pub fn get(&self, index: u32) -> Option<&T> {
        self.items.get(index)
    }

    pub fn get_items(&self, offset: u32, limit: u32) -> Vec<&T> {
        (offset..min(self.items.len(), offset + limit))
            .map(|index| self.items.get(index).unwrap())
            .collect()
    }

    pub fn get_active_item_indices(&self) -> Vec<u32> {
        self.active_items.iter().copied().collect()
    }

    pub fn get_item_indices(&self) -> Vec<u32> {
        (0..self.items.len()).collect()
    }

    pub fn update(&mut self, index: u32, item: T) {
        let active = item.is_active();
        if active && !self.active_items.contains(&index) {
            self.active_items.insert(index);
        }
        if !active && self.active_items.contains(&index) {
            self.active_items.remove(&index);
        }

        self.items.replace(index, item);
    }
}
