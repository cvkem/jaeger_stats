use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Counted<T: Hash + Eq + PartialEq>(HashMap<T, u32>);

impl<T: Hash + Eq + PartialEq> Counted<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_item(&mut self, item: T) -> u32 {
        let cnt = self.0.entry(item).and_modify(|cnt| *cnt += 1).or_insert(1);
        *cnt
    }

    pub fn add_items(&mut self, items: Vec<T>) {
        items.into_iter().for_each(|item| {
            self.add_item(item);
        })
    }
}
