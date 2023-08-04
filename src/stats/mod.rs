//!  Computing statistics and call-chains over the traces.

mod traceext;
mod stats;
mod method_stats;
mod rate;
pub mod call_chain;  // already defines its public interface
pub mod json;  // already defines its public interface


pub use {
    call_chain::CChainEndPointCache,
    method_stats::{MethodStats, MethodStatsValue},
    stats::{Stats, StatsRec, chained_stats, set_comma_float},
    traceext::{build_trace_ext, write_stats_to_csv_file, TraceExt}
};
