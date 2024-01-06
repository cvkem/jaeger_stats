//! NOT USED yet. DEAD CODE
//! This was aimed at getting dot-graphs. However we switched to using Mermaid.
//!
//! Build a graph out of the Call-chain infomration.
//!
mod build_graph;
mod fix_callchain;
mod id_mapper;
mod process_node;

pub use build_graph::build_graph;
