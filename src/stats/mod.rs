//!  Computing statistics and call-chains over the traces.

pub mod call_chain; // already defines its public interface
pub mod json;
mod method_stats;
mod rate;
mod stats;
mod traceext; // already defines its public interface

pub use {
    call_chain::CChainEndPointCache,
    method_stats::{MethodStats, MethodStatsValue},
    stats::{chained_stats, set_comma_float, Stats, StatsRec},
    traceext::{build_trace_ext, write_stats_to_csv_file, TraceExt},
};
