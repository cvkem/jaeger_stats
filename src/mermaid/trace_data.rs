use crate::stats::CChainStatsKey;

#[derive(Clone, Copy)]
/// Contains the contains the count and the statistics for a TraceData struct
pub struct TraceDataStats {
    /// The number of times this path is traversed
    pub count: u64,
    pub rate: Option<f64>,
    pub avg_duration_millis: f64,
    pub p75_millis: Option<f64>,
    pub p90_millis: Option<f64>,
    pub p95_millis: Option<f64>,
    pub p99_millis: Option<f64>,
    // add min-milis, max-millis & median-millis
}

// pub struct TraceStep {
//     pub service: String,
//     pub operation: String,
// }

//type TracePath = Vec<TraceStep>;
type TracePath = CChainStatsKey;

pub struct TraceData {
    /// unique key constructed by the concatenation of all steps of the trace.
    pub full_key: String,
    pub trace_path: TracePath,
    // /// Key constructed from the inbound calls only (outbound calls, such as GET and POST are omitted). This key is likely unique, but is not guaranteed to be unique
    // #[serde(alias = "inboud_process_key")]
    // // fix as we had a typo in the past. I doubt whether this alias is needed.
    // pub inbound_process_key: String,
    /// This process refers back to the (identified) root of the full trace
    pub rooted: bool,
    /// This Call-chain ends at a leaf, and thus covers a full chain (provided it is also rooted)
    pub is_leaf: bool,
    /// the statistical data about this step in a trace
    pub data: TraceDataStats,
}

impl TraceData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        full_key: &str,
        rooted: bool,
        is_leaf: bool,
        count: u64,
        rate: Option<f64>,
        avg_duration_millis: f64,
        p75_millis: Option<f64>,
        p90_millis: Option<f64>,
        p95_millis: Option<f64>,
        p99_millis: Option<f64>,
    ) -> Self {
        let full_key = full_key.to_owned();
        let trace_path = CChainStatsKey::parse(&full_key).unwrap_or_else(|err| {
            panic!("Failed to parse CChainKey: '{full_key}'\nGot error '{err:?}'")
        });
        let data = TraceDataStats {
            count,
            rate,
            avg_duration_millis,
            p75_millis,
            p90_millis,
            p95_millis,
            p99_millis,
        };
        Self {
            full_key,
            trace_path,
            rooted,
            is_leaf,
            data,
        }
    }
}
