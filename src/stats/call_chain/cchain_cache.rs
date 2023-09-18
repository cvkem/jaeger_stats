use super::{
    cchain_stats::CChainStatsKey,
    file::{cchain_filename, read_cchain_file},
};
use crate::utils;

use std::{collections::HashMap, mem, path::PathBuf};

/// An end-point has a set of call-chains that originate from this endpoint (each represented by a CChainStatsKey)
//pub type EndPointCChain = Vec<CChainStatsKey>;
pub struct EndPointCChains {
    dirty: bool,
    pub chains: Vec<CChainStatsKey>,
}

impl EndPointCChains {
    pub fn new(chains: Vec<CChainStatsKey>) -> Self {
        let dirty = false;
        Self { dirty, chains }
    }

    pub fn new_dirty(chains: Vec<CChainStatsKey>) -> Option<Self> {
        let dirty = true;
        Some(Self { dirty, chains })
    }

    /// udpate the entry with the provided chains. If updates are needed set dirty-flag to false
    pub fn update_chains(&mut self, check_chains: Vec<CChainStatsKey>) {
        let new_entries: Vec<_> = check_chains
            .into_iter()
            .filter(|check_key| self.chains.iter().any(|curr_key| *curr_key == *check_key))
            .collect();
        if !new_entries.is_empty() {
            self.chains.extend(new_entries);
            self.dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn write_call_chain(&self, mut cchain_file_base: PathBuf, k: &str) {
        // extract call-chains and write to file
        cchain_file_base.push(cchain_filename(k));
        let file_name = cchain_file_base.to_str().unwrap();
        let cchain_str = self
            .chains
            .iter()
            .map(|cc| cc.call_chain_key())
            .collect::<Vec<_>>()
            .join("\n");
        utils::write_string_to_file(file_name, cchain_str).expect("Failed to write cchain-files.");
    }
}

pub struct CChainEndPointCache {
    path: PathBuf,
    cache: HashMap<String, Option<EndPointCChains>>,
}

impl CChainEndPointCache {
    pub fn new(path: PathBuf) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| panic!("Failed to build a Call-Chain cache from folder '{}'. Did you set the -c flag correctly?", path.display()));
        Self {
            path,
            cache: HashMap::new(),
        }
    }

    /// internatal function that returns a mutable reference
    fn get_cchain_key_aux(&mut self, key: &str) -> &mut Option<EndPointCChains> {
        self.cache.entry(key.to_string()).or_insert_with(|| {
            let mut path = self.path.clone();
            path.push(cchain_filename(key));
            if path.is_file() {
                match read_cchain_file(&path) {
                    Ok(cchain_key) => Some(cchain_key),
                    Err(err) => {
                        utils::report(
                            utils::Chapter::Issues,
                            format!("Loading of entry '{key}' failed with error: {err:?}"),
                        );
                        None
                    }
                }
            } else {
                utils::report(
                    utils::Chapter::Details,
                    format!(
                        "Could not find '{}' so no call-chain available",
                        path.display()
                    ),
                );
                None
            }
        })
    }

    pub fn str_to_cache_key(s: &str) -> String {
        s.replace(&['/', '\\', ';', ':'][..], "_")
    }
    /// extract a reference to an EndPointCChains
    pub fn get_cchain_key(&mut self, key: &str) -> Option<&EndPointCChains> {
        self.get_cchain_key_aux(key).as_ref()
    }

    /// Create a new entry of update an existing entry with the provided cchains
    pub fn create_update_entry(&mut self, key: &str, cchains: Vec<CChainStatsKey>) {
        match self.get_cchain_key_aux(key) {
            Some(entry) => entry.update_chains(cchains),
            None => {
                _ = self
                    .cache
                    .insert(key.to_string(), EndPointCChains::new_dirty(cchains));
            }
        }
    }
}

impl Drop for CChainEndPointCache {
    /// write all dirty cache entries to a file
    fn drop(&mut self) {
        mem::take(&mut self.cache)
            .into_iter()
            .for_each(|(k, v)| match v {
                Some(v) => {
                    if v.is_dirty() {
                        v.write_call_chain(self.path.clone(), &k);
                    }
                }
                None => (),
            });
    }
}
