use crate::{
    mermaid, stats::StatsRec, utils, utils::TimeStats,
    EdgeValue,
};
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
                        let count = ccv.count as u64;
                        let avg_duration_millis = TimeStats(&ccv.duration_micros).get_avg_millis();
                        let p75_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.75);
                        let p90_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.90);
                        let p95_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.95);
                        let p99_millis = TimeStats(&ccv.duration_micros).get_p_millis(0.99);

                        // TODO: Made switch to aggregator at the wrong site. this is still a tree. Move it to get_diagram
                        mermaid::TraceData::new(
                            &key,
                            ccv.rooted,
                            cck.is_leaf,
                            count,
                            avg_duration_millis,
                            p75_millis,
                            p90_millis,
                            p95_millis,
                            p99_millis,
                        )
                    })
                    .collect();
                (service_oper.clone(), trace_data)
            })
            .collect();

        let diagram =
            //TODO: do aggregation here
            mermaid::TracePaths(trace_tree).get_diagram(proc_oper, call_chain_key, edge_value, scope, compact);

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
