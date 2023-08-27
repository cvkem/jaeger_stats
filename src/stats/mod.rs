//!  Computing statistics and call-chains over the traces.

pub mod call_chain; // already defines its public interface
mod error_stats;
pub mod file;
mod operation_stats;
mod proc_oper_stats;
mod stats_rec;
mod traceext; // already defines its public interface // already defines its public interface

pub use {
    call_chain::CChainEndPointCache,
    operation_stats::OperationStats,
    proc_oper_stats::{ProcOperStats, ProcOperStatsValue},
    stats_rec::{chained_stats, StatsRec, Version},
    traceext::{build_trace_ext, write_stats_to_csv_file, TraceExt},
};
