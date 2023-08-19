use super::{
    call::Call, call_chain::CallChain, cchain_cache::EndPointCChain, cchain_stats::CChainStatsKey,
};
use crate::utils::read_lines;
use std::{error::Error, path::Path};

/// get the file-name for a specific key (excluding path)
pub fn cchain_filename(key: &str) -> String {
    format!("{key}.cchain")
}

const LEAF_LABEL_WITH_SPACE: &str = " *LEAF*";

pub const LEAF_LABEL: &str = "*LEAF*"; // LEAF_LABEL_WITH_SPACE.trim();

/// build a call-chain-key based on parameters.
/// This is a separate function as this allows us to put in another caching_process than contained in the CallChainStatsKey.
pub fn call_chain_key(call_chain: &CallChain, caching_process: &str, is_leaf: bool) -> String {
    let call_chain_str = call_chain.iter().fold(String::new(), |a, b| {
        let sep = if !a.is_empty() { " | " } else { "" };
        a + sep + &b.to_string()
    });
    let leaf_str = if is_leaf { LEAF_LABEL_WITH_SPACE } else { "" };
    call_chain_str + " & " + caching_process + "& " + leaf_str // using '&' as separator as a ';' separator would break the CSV-files
}

/// read a cchain-file and parse it
pub fn read_cchain_file(path: &Path) -> Result<EndPointCChain, Box<dyn Error>> {
    Ok(read_lines(path)?
        .filter_map(|l| {
            let l = l.unwrap();
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                None
            } else {
                Some(CChainStatsKey::parse(l).unwrap())
            }
        })
        .collect())
}

/// the label shows whether cached processes are in the call-chain and if so returns a suffix to represent it.
pub fn caching_process_label(caching_process: &Vec<String>, call_chain: &CallChain) -> String {
    if caching_process.is_empty() {
        return "".to_owned();
    }
    let mut cached = Vec::new();

    call_chain.iter().for_each(
        |Call {
             process, method, ..
         }| {
            match &method[..] {
                "GET" | "POST" | "HEAD" | "QUERY" => (), // ignore these methods as the inbound call has been matched already. (prevent duplicates of cached names)
                _ => {
                    if caching_process.iter().any(|s| *s == *process) {
                        cached.push(process.to_owned())
                    }
                }
            }
        },
    );
    if !cached.is_empty() {
        format!(" [{}]", cached.join(", "))
    } else {
        "".to_owned()
    }
}
