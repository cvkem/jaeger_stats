use super::StitchedSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CallChainData {
    /// unique key constructed by the concatenation of all steps of the trace.
    pub full_key: String,
    /// Key constructed from the inbound calls only (outbound calls, such as GET and POST are omitted). This key is likely unique, but is not guaranteed to be unique
    pub inbound_process_key: String,
    /// This process refers back to the (identified) root of the full trace
    pub rooted: bool,
    /// This Call-chain ends at a leaf, and thus covers a full chain (provided it is rooted)
    pub is_leaf: bool,
    /// The set of actual data associated with the Call-chain.
    pub data: StitchedSet,
}

impl CallChainData {
    pub fn new(
        full_key: String,
        inbound_process_key: String,
        rooted: bool,
        is_leaf: bool,
        data: StitchedSet,
    ) -> Self {
        Self {
            full_key,
            inbound_process_key,
            rooted,
            is_leaf,
            data,
        }
    }

    pub fn chain_type(&self) -> &str {
        match (self.rooted, self.is_leaf) {
            (true, true) => "end2end",
            (true, false) => "partial",
            (false, true) => "unrooted-leaf",
            (false, false) => "floating",
        }
    }

    /// Get a subset of selected data-points for each of the stiched lines in the stitched set, or None if the selection does not contain any f64 values (only None)
    /// assume that the size of the selection was checked by the upstream process (the caller).
    pub fn get_selection(&self, selection: &[bool]) -> Option<Self> {
        self.data
            .get_selection(selection)
            .map(|data| CallChainData {
                full_key: self.full_key.to_owned(),
                inbound_process_key: self.inbound_process_key.to_owned(),
                rooted: self.rooted,
                is_leaf: self.is_leaf,
                data,
            })
    }
}
