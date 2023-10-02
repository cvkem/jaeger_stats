use crate::{BestFit, Stitched, StitchedLine, StitchedSet};
use log::error;
use regex::Regex;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Serialize)]
pub struct ProcessListItem {
    pub idx: usize,
    pub key: String,
    pub display: String, // the display can be used in select-boxes, but is nog guaranteed to be unique (at least nog for a Call-chain list.)
    pub rank: f64,
}

type ProcessList = Vec<ProcessListItem>;

/// find the list of labels for the graphs by extracting them from the source-list descriptions.
pub fn get_label_list(data: &Stitched) -> Vec<String> {
    let re = Regex::new(r"[0-9]{8}").expect("Failed to create regexp for dates");

    data.sources
        .0
        .iter()
        .filter(|src| src.column.is_some())
        .enumerate()
        .map(|(idx, src)| match re.find(&src.description) {
            Some(label) => label.as_str().to_owned(),
            None => format!("{}", idx),
        })
        .collect()
}

/// get the rank of this stitched set based on the growth of the 'metric'.
fn get_stitched_set_rank(stitch_set: &StitchedSet, metric: &str) -> f64 {
    // rank on the periodic-growth of the selected metric
    let line = stitch_set
        .get_metric_stitched_line(metric)
        .unwrap_or_else(|| panic!("Could not find ranking-metric '{}'", metric));
    line.periodic_growth().unwrap_or(-1000.0)
}

/// Reorder the list of processes based on the rank field and renumber if the 'metric' is set.
fn reorder_and_renumber(mut proc_list: ProcessList, metric: &str) -> ProcessList {
    if !metric.is_empty() {
        proc_list.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(Ordering::Equal));

        // renumber for new ordering
        proc_list
            .iter_mut()
            .enumerate()
            .for_each(|(idx, pli)| pli.idx = idx + 1);
    }

    proc_list
}

/// Reorder the list of processes based on the rank field and renumber if the 'metric' is set.
fn rank_lexicographic(mut proc_list: ProcessList) -> ProcessList {
    proc_list.sort_by(|a, b| a.key.cmp(&b.key));
    let list_len = proc_list.len();

    // renumber for new ordering
    proc_list
        .iter_mut()
        .enumerate()
        .for_each(|(idx, pli)| {
            pli.idx = idx + 1;
            pli.rank = (list_len -idx) as f64;
    });

    proc_list
}

/// return a ranked list of processes where rank is based on the periodic-growth of the metric provided.
/// If metric is an empty string the data will be provided in the current order (lexicographic sort.)
pub fn get_process_list(data: &Stitched, metric: &str) -> ProcessList {
    let list_size = data.process_operation.len();
    let proc_list: Vec<_> = data
        .process_operation
        .iter()
        .enumerate()
        .map(|(idx, po)| {
            // provide a rank based on the reverse of the index, as the highest rank should be in first position.
            let rank = if metric.is_empty() {
                (list_size - idx) as f64
            } else {
                get_stitched_set_rank(&po.1, metric)
            };

            ProcessListItem {
                idx: idx + 1,
                key: po.0.to_owned(),
                display: po.0.to_owned(),
                rank,
            }
        })
        .collect();

    reorder_and_renumber(proc_list, metric)
}

/// get an ordered list of call-chains ranked based on 'metric' that are inbound on a point.
fn get_call_chain_list_inbound(data: &Stitched, proc_oper: &str, metric: &str) -> ProcessList {
    let proc_list = match data
        .call_chain
        .iter()
        .filter(|(k, _v)| k == proc_oper)
        .map(|(_k, v)| v)
        .next()
    {
        Some(ccd_vec) => {
            let list_size = ccd_vec.len();
            ccd_vec
                .iter()
                .enumerate()
                .map(|(idx, ccd)| {
                    // provide a rank based on the reverse of the index, as the highest rank should be in first position.
                    let rank = if metric.is_empty() {
                        (list_size - idx) as f64
                    } else {
                        get_stitched_set_rank(&ccd.data, metric)
                    };

                    if ccd.inboud_process_key.is_empty() {
                        println!(
                            "EMPTY inbound-key on {:?} for {:?}",
                            ccd.inboud_process_key, ccd.full_key
                        );
                    }

                    ProcessListItem {
                        idx: idx + 1,
                        key: ccd.full_key.to_owned(),
                        display: ccd.inboud_process_key.to_owned(),
                        rank,
                    }
                })
                .collect()
        },
        None => {
            error!("Could not find section for proces_oper = '{proc_oper}'");
            Vec::new()
        }
    };

    reorder_and_renumber(proc_list, metric)
}


/// get an ordered list of call-chains ranked based on 'metric' that are end2end process (from end-point to leaf-process of the call-chain).
fn get_call_chain_list_end2end(data: &Stitched, proc_oper: &str, metric: &str) -> ProcessList {
    let re = Regex::new(proc_oper).expect("Failed to create regex for proc_oper");

    let proc_list = data
        .call_chain
        .iter()
        .flat_map(|(_k, ccd_vec)| {
            ccd_vec
                .iter()
                .filter(|ccd| ccd.is_leaf)
                .filter(|ccd| re.find(&ccd.full_key).is_some())
                .map(|ccd| {
                    // provide a rank based on the reverse of the index, as the highest rank should be in first position.
                    let rank = if metric.is_empty() {
                         -1000.0  // will be rewritten
                    } else {
                        get_stitched_set_rank(&ccd.data, metric)
                    };

                    ProcessListItem {
                        idx: 0, // will be rewritten
                        key: ccd.full_key.to_owned(),
                        display: ccd.inboud_process_key.to_owned(),
                        rank,
                    }
                })
        })
        .collect();

    if metric.is_empty() {
        rank_lexicographic(proc_list)
    } else {
        reorder_and_renumber(proc_list, metric)
    }
}

/// get an ordered list of call-chains ranked based on 'metric' that are inbound on a point.
pub fn get_call_chain_list(data: &Stitched, proc_oper: &str, metric: &str, scope: &str) -> ProcessList {
    match scope {
        "inbound" | "" => get_call_chain_list_inbound(data, proc_oper, metric), // default option
        "end2end" => get_call_chain_list_end2end(data, proc_oper, metric),
        scope => {
            error!("Unknown scope '{scope}' expected either 'inbound' or 'end2end'.");
            Vec::new()
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ChartLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
}

#[derive(Serialize, Debug)]
pub struct ChartDataParameters {
    pub title: String,
    pub process: String,
    pub metric: String,
    pub description: Vec<(String, String)>,
    pub labels: Vec<String>,
    pub lines: Vec<ChartLine>,
}

impl ChartDataParameters {
    pub fn new(process: &str, metric: &str, st_line: &StitchedLine) -> Self {
        let labels = st_line
            .data
            .iter()
            .enumerate()
            .map(|(idx, _)| format!("{}", idx))
            .collect();
        let mut lines = Vec::new();
        lines.push(ChartLine {
            label: "Observed".to_string(),
            data: st_line.data.clone(),
        });
        if let Some(lin_regr) = &st_line.lin_regr {
            lines.push(ChartLine {
                label: format!("y= {:.2}*x + {:.2}", lin_regr.slope, lin_regr.y_intercept),
                data: st_line
                    .data
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Some(lin_regr.predict(idx as f64)))
                    .collect(),
            });
        };
        if let Some(exp_regr) = &st_line.exp_regr {
            lines.push(ChartLine {
                label: format!("y= {:.2} * {:.2}^x", exp_regr.a, exp_regr.b),
                data: st_line
                    .data
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| Some(exp_regr.predict(idx as f64)))
                    .collect(),
            });
        };
        let description = {
            let growth = st_line.periodic_growth().map(|v| v * 100.0);
            let best_fit = match st_line.best_fit {
                BestFit::ExprRegr => format!(
                    "Exponential ({:1}%)",
                    growth.expect("Exp. growth value missing")
                ),
                BestFit::LinRegr => format!(
                    "Lineair ({:1}%)",
                    growth.expect("Lin. growth value missing")
                ),
                BestFit::None => "None".to_string(),
            };
            vec![
                ("BestFit".to_owned(), best_fit),
                //TODO: more items can be added. Would be nice to state if this is inbound or outbound
            ]
        };
        ChartDataParameters {
            title: format!("{}  of {}", metric, process),
            process: process.to_owned(),
            metric: metric.to_owned(),
            description,
            labels,
            lines,
        }
    }
}

/// the the chart-data for a specific Process-operation combination
pub fn get_proc_oper_chart_data(
    data: &Stitched,
    process: &str,
    metric: &str,
) -> Option<ChartDataParameters> {
    match data
        .process_operation
        .iter()
        .filter(|(proc, _)| proc == process)
        .next()
    {
        Some((proc, st_set)) => st_set
            .get_metric_stitched_line(metric)
            .map(|sl| ChartDataParameters::new(proc, metric, sl)),
        None => {
            error!("Could not find process '{process}'");
            None
        }
    }
}

/// the the chart-data for a specific call-chain (within a process context)
/// the process can not be derived from the call-chain as we only have a string-key with inbound processes,
pub fn get_call_chain_chart_data(
    data: &Stitched,
    process_key: &str,
    call_chain_key: &str,
    metric: &str,
) -> Option<ChartDataParameters> {
    match data
        .call_chain
        .iter()
        .filter(|(proc, _)| proc == process_key)
        .next()
    {
        Some((proc, ccd_vec)) => match ccd_vec
            .iter()
            .filter(|ccd| ccd.inboud_process_key == call_chain_key)
            .next()
        {
            Some(ccd) => ccd
                .data
                .get_metric_stitched_line(metric)
                .map(|sl| ChartDataParameters::new(proc, metric, sl)),
            None => {
                error!("Could not find call-chain '{call_chain_key}' within list for process '{process_key}'");
                None
            }
        },
        None => {
            error!("Could not find process '{process_key}'");
            None
        }
    }
}
