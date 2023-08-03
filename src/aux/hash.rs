use std::hash::{Hash, Hasher};

use fxhash::FxHasher;

pub fn hash<T: Hash>(t: &T) -> u64 {
    let mut s: FxHasher = Default::default();
    t.hash(&mut s);
    s.finish()
}

// pub fn hash_string(inp: &str) -> String {
//     let mut s: CityHasher = Default::default();
//     inp.hash(&mut s);
//     format!("{:x}", s.finish())
// }

pub fn string_hash<T: Hash>(t: &T) -> String {
    format!("{:x}", hash(t))
}
