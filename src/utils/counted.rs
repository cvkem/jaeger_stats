use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

/// Maintain a counted list of occurances of Items of type T via a hashmap.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Counted<T: Hash + Eq + PartialEq>(HashMap<T, usize>);

impl<T: Hash + Eq + PartialEq> Counted<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// get the current count of an item. Returns 0 if the item does not exist.
    pub fn get_count(&self, item: T) -> usize {
        *self.0.get(&item).unwrap_or(&0)
    }

    /// The basis method to add 1 to the count of an item
    pub fn add_item(&mut self, item: T) -> usize {
        let cnt = self.0.entry(item).and_modify(|cnt| *cnt += 1).or_insert(1);
        *cnt
    }

    /// Compute counts over a vector of keys (each key counts as 1 item)
    pub fn add_items(&mut self, items: Vec<T>) {
        items.into_iter().for_each(|item| {
            self.add_item(item);
        })
    }

    /// this method allows to a specific positive count to an item.
    pub fn add_item_count(&mut self, item: T, count: usize) -> usize {
        let cnt = self
            .0
            .entry(item)
            .and_modify(|cnt| *cnt += count)
            .or_insert(count);
        *cnt
    }
}
