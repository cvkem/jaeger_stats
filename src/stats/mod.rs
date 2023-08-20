//!  Computing statistics and call-chains over the traces.

pub mod call_chain; // already defines its public interface
mod error_stats;
pub mod file;
mod proc_oper_stats;
mod stats;
mod traceext; // already defines its public interface // already defines its public interface

pub use {
    call_chain::CChainEndPointCache,
    proc_oper_stats::{ProcOperStats, ProcOperStatsValue},
    stats::{chained_stats, Stats, StatsRec, Version},
    traceext::{build_trace_ext, write_stats_to_csv_file, TraceExt},
};
