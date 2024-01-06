pub struct TraceData {
    /// unique key constructed by the concatenation of all steps of the trace.
    pub full_key: String,
    // /// Key constructed from the inbound calls only (outbound calls, such as GET and POST are omitted). This key is likely unique, but is not guaranteed to be unique
    // #[serde(alias = "inboud_process_key")]
    // // fix as we had a typo in the past. I doubt whether this alias is needed.
    // pub inbound_process_key: String,
    /// This process refers back to the (identified) root of the full trace
    pub rooted: bool,
    /// This Call-chain ends at a leaf, and thus covers a full chain (provided it is rooted)
    pub is_leaf: bool,
    /// The number of times this path is traversed
    pub count: f64,
}

impl TraceData {
    pub fn new(full_key: &String, rooted: bool, is_leaf: bool, count: f64) -> Self {
        let full_key = full_key.clone();
        Self {
            full_key,
            rooted,
            is_leaf,
            count,
        }
    }
}
