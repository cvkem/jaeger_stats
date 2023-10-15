use super::super::Stitched;
use std::{cmp::Ordering, collections::HashMap};

pub struct InboundPrefixIdxItem {
    pub prefix: String,
    pub idx: i64,
}

pub struct InboundPrefixIdx(Vec<InboundPrefixIdxItem>);

impl InboundPrefixIdx {
    /// get the map of inbound-processes that maps the prefix to a process-id.
    pub fn new(data: &Stitched, proc_oper: &str) -> Self {
        let po_items: Vec<_> = data
            .call_chain
            .iter()
            .filter(|(k, _v)| k == proc_oper)
            .map(|(_k, v)| v)
            .next()
            .unwrap_or_else(|| {
                panic!("There should be a least on instance of proc_oper: {proc_oper}")
            })
            .iter()
            .enumerate()
            .map(|(idx, ccd)| {
                let parts: Vec<_> = ccd.full_key.split("&").collect();
                if parts.len() != 3 {
                    panic!(
                        "full-key was split in {} parts, i.e. {parts:?} (3 parts expected)",
                        parts.len()
                    );
                }
                let prefix = parts[0].trim();
                let idx = (idx + 1) as i64;
                (ccd.is_leaf, prefix, idx)
            })
            .collect();
        let mut inbound_idx_map = HashMap::new();
        // First insert tails.
        po_items
            .iter()
            .filter(|(is_leaf, _, _)| *is_leaf)
            .for_each(|(_, prefix, idx)| {
                _ = inbound_idx_map.insert((*prefix).to_string(), *idx);
            });
        // Next overwrite with the non-tails
        po_items
            .iter()
            .filter(|(is_leaf, _, _)| !*is_leaf)
            .for_each(|(_, prefix, idx)| {
                _ = inbound_idx_map.insert((*prefix).to_string(), *idx);
            });
        let mut inbound_idx_list = InboundPrefixIdx(
            inbound_idx_map
                .into_iter()
                .map(|(prefix, idx)| InboundPrefixIdxItem { prefix, idx })
                .collect(),
        );
        inbound_idx_list
            .0
            .sort_by(|a, b| match (a.prefix.len(), b.prefix.len()) {
                (al, bl) if al > bl => Ordering::Less,
                (al, bl) if al < bl => Ordering::Greater,
                _ => Ordering::Equal,
            });
        inbound_idx_list
    }

    /// the the matching prefix, or return 0
    pub fn get_idx(&self, full_key: &str) -> i64 {
        match self
            .0
            .iter()
            .filter(|iit| full_key.starts_with(&iit.prefix))
            .next()
        {
            Some(iit) => iit.idx,
            None => 0, // no match found
        }
    }
}
