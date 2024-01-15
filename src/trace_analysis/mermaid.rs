use crate::{mermaid, stats::StatsRec, utils, EdgeValue};
use std::path::PathBuf;

impl StatsRec {
    /// get a mermaid diagram that depicts the current selection based on proc_oper and optionally a call-chain.
    pub fn write_mermaid_diagram(
        &self,
        folder: &PathBuf,
        proc_oper: &str,
        call_chain_key: Option<&str>,
        edge_value: EdgeValue,
        scope: mermaid::MermaidScope,
        compact: bool,
    ) {
        let trace_tree = self
            .stats
            .iter()
            .map(|(service_oper, oper_stats)| {
                let trace_data = oper_stats
                    .call_chain
                    .0
                    .iter()
                    .map(|(cck, ccv)| {
                        let key = cck.call_chain_key();
                        mermaid::TraceData::new(&key, ccv.rooted, cck.is_leaf, ccv.count as f64)
                    })
                    .collect();
                (service_oper.clone(), trace_data)
            })
            .collect();

        let diagram =
            mermaid::TraceTree(trace_tree).get_diagram(proc_oper, call_chain_key, scope, compact);

        write_diagram(folder, proc_oper, diagram);
    }
}

fn write_diagram(folder: &PathBuf, proc_oper: &str, diagram: String) {
    let mut file_path = folder.clone();

    let clean_proc_oper = proc_oper.replace(['/', '\\'], "_"); // The / is not allowed in a file-path
    file_path.push(format!("{}.mermaid", clean_proc_oper));
    let file_path = file_path.to_str().expect("invalid file-path");
    if let Err(err) = utils::write_string_to_file(file_path, diagram) {
        panic!("Writing to file '{file_path}' failed with error: {err:?}");
    };
}
