use std::collections::HashMap;

pub struct CountedPrefix(pub HashMap<String, f64>);

impl CountedPrefix {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// add a new prefix or increment the existing with 'count'
    pub fn add(&mut self, prefix: &str, count: f64) {
        self.0
            .entry(prefix.to_owned())
            .and_modify(|v| *v += count)
            .or_insert(count);
    }
}
