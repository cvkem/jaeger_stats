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
                        println!("Loading of entry '{key}' failed with error: {err:?}");
                        None
                    }
                }
            } else {
                println!(
                    "Could not find '{}' so no call-chain available",
                    path.display()
                );
                None
            }
        })
    }

    /// extract a refernce to an EndPointCChains
    pub fn get_cchain_key(&mut self, key: &str) -> Option<&EndPointCChains> {
        self.get_cchain_key_aux(key).as_ref()
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
