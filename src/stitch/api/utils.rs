use super::{
    inbound_prefix_idx::InboundPrefixIdx,
    types::{ChartDataParameters, ChartLine, ProcessList, ProcessListItem, Table},
};
use crate::{BestFit, Stitched, StitchedLine, StitchedSet};
use log::error;
use regex::Regex;
use std::cmp::Ordering;

const DEFAULT_RANK: f64 = -1.0; // indicates growth not defined

/// Map a numberal string like for example "02" to the string "Febr".
fn get_month_description(month: &str) -> &str {
    match month {
        "01" => "Jan",
        "02" => "Febr",
        "03" => "March",
        "04" => "April",
        "05" => "May",
        "06" => "June",
        "07" => "July",
        "08" => "Aug",
        "09" => "Sept",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        month => panic!("Invalid month {month} expecting string with exactly two digits and zero-prefixed for jan-sept.")
    }
}

/// find the list of labels for the graphs by extracting them from the source-list descriptions.
/// The description is assume to contain a sub-string yyyymmdd, so for example "20231008". For this string the output Sept 8 is produced.
pub fn get_label_list(data: &Stitched) -> Vec<String> {
    // TODO: we could guard against multiple matches being present
    let re = Regex::new(r"(\d{4})(\d{2})(\d{2})").expect("Failed to create regexp for dates");

    data.sources
        .0
        .iter()
        .filter(|src| src.column.is_some())
        .enumerate()
        .map(
            |(idx, src)| match re.captures(&src.description).map(|caps| caps.extract()) {
                Some((_full, [_year, month, day])) => {
                    let month = get_month_description(month);
                    // remove the 0-prefix if it exists
                    let day = if day.chars().next() == Some('0') {
                        &day[1..]
                    } else {
                        day
                    };
                    format!("{month}-{day}")
                }
                None => format!("{}", idx),
            },
        )
        .collect()
}

/// get the rank of this stitched set based on the growth of the 'metric'.
fn get_stitched_set_rank(stitch_set: &StitchedSet, metric: &str) -> f64 {
    // rank on the periodic-growth of the selected metric
    let line = stitch_set
        .get_metric_stitched_line(metric)
        .unwrap_or_else(|| panic!("Could not find ranking-metric '{}'", metric));
    line.periodic_growth().unwrap_or(DEFAULT_RANK)
}

/// get the average count of this process or call-chain over a measurements.
fn get_stitched_set_count(stitch_set: &StitchedSet) -> i64 {
    let metric = "count";
    let line = stitch_set
        .get_metric_stitched_line(metric)
        .unwrap_or_else(|| panic!("Could not find metric '{}'", metric));
    match line.data_avg {
        Some(avg) => avg.round() as i64,
        None => 0,
    }
}

/// Reorder the list of processes based on the rank field and renumber if the 'metric' is set.
fn reorder_and_renumber(mut proc_list: ProcessList, metric: &str) -> ProcessList {
    if !metric.is_empty() {
        proc_list.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(Ordering::Equal));

        // renumber for new ordering
        proc_list
            .iter_mut()
            .enumerate()
            .for_each(|(idx, pli)| pli.idx = (idx + 1) as i64);
    }

    proc_list
}

/// Reorder the list of processes based on the rank field and renumber if the 'metric' is set.
fn rank_lexicographic(mut proc_list: ProcessList) -> ProcessList {
    proc_list.sort_by(|a, b| a.key.cmp(&b.key));
    let list_len = proc_list.len();

    // renumber for new ordering
    proc_list.iter_mut().enumerate().for_each(|(idx, pli)| {
        pli.idx = (idx + 1) as i64;
        pli.rank = (list_len - idx) as f64;
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
            let avg_count = get_stitched_set_count(&po.1);

            ProcessListItem {
                idx: (idx + 1) as i64,
                key: po.0.to_owned(),
                display: po.0.to_owned(),
                rank,
                avg_count,
                chain_type: String::new(),
                inbound_idx: 0,
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

                    if ccd.inbound_process_key.is_empty() {
                        println!(
                            "EMPTY inbound-key on {:?} for {:?}",
                            ccd.inbound_process_key, ccd.full_key
                        );
                    }

                    let avg_count = get_stitched_set_count(&ccd.data);

                    ProcessListItem {
                        idx: (idx + 1) as i64,
                        key: ccd.full_key.to_owned(),
                        display: ccd.inbound_process_key.to_owned(),
                        rank,
                        avg_count,
                        chain_type: ccd.chain_type().to_owned(),
                        inbound_idx: 0,
                    }
                })
                .collect()
        }
        None => {
            error!("Could not find section for proces_oper = '{proc_oper}'");
            Vec::new()
        }
    };

    reorder_and_renumber(proc_list, metric)
}

/// get an ordered list of call-chains ranked based on 'metric' that are end2end process (from end-point to leaf-process of the call-chain).
fn get_call_chain_list_end2end(
    data: &Stitched,
    proc_oper: &str,
    metric: &str,
    all_chains: bool,
    inbound_idx_filter: Option<i64>,
) -> ProcessList {
    let re = Regex::new(proc_oper).expect("Failed to create regex for proc_oper");

    let inbound_prefix_idx = InboundPrefixIdx::new(data, proc_oper);

    let proc_list = data
        .call_chain
        .iter()
        .filter(|(k, _ccd)| k != proc_oper) // these are already reported as inbound chains
        .flat_map(|(_k, ccd_vec)| {
            ccd_vec
                .iter()
                .filter(|ccd| all_chains || ccd.is_leaf)
                .filter(|ccd| re.find(&ccd.full_key).is_some())
                .filter_map(|ccd| {
                    // provide a rank based on the reverse of the index, as the highest rank should be in first position.
                    let rank = if metric.is_empty() {
                        DEFAULT_RANK // will be rewritten before returning this value
                    } else {
                        get_stitched_set_rank(&ccd.data, metric)
                    };
                    let avg_count = get_stitched_set_count(&ccd.data);

                    let inbound_idx = inbound_prefix_idx.get_idx(&ccd.full_key);
                    if inbound_idx_filter.is_none() || inbound_idx_filter == Some(inbound_idx) {
                        Some(ProcessListItem {
                            idx: 0, // will be rewritten
                            key: ccd.full_key.to_owned(),
                            display: ccd.inbound_process_key.to_owned(),
                            rank,
                            avg_count,
                            chain_type: ccd.chain_type().to_owned(),
                            inbound_idx,
                        })
                    } else {
                        None // inbound_idx does not match the filter
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
pub fn get_call_chain_list(
    data: &Stitched,
    proc_oper: &str,
    metric: &str,
    scope: &str,
    inbound_idx: Option<i64>,
) -> ProcessList {
    match scope {
        "inbound" | "" => get_call_chain_list_inbound(data, proc_oper, metric), // default option
        "end2end" => get_call_chain_list_end2end(data, proc_oper, metric, false, inbound_idx),
        "all" => get_call_chain_list_end2end(data, proc_oper, metric, true, inbound_idx),
        scope => {
            error!("Unknown scope '{scope}' expected either 'inbound', 'end2end' or 'all'.");
            Vec::new()
        }
    }
}

impl ChartDataParameters {
    pub fn new(process: &str, metric: &str, labels: Vec<String>, st_line: &StitchedLine) -> Self {
        // let labels = st_line
        //     .data
        //     .iter()
        //     .enumerate()
        //     .map(|(idx, _)| format!("{}", idx))
        //     .collect();
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
            if growth.is_none() {
                match st_line.best_fit {
                    BestFit::LinRegr | BestFit::ExprRegr => error!(
                        "Could not find growth value for best-fit model {:?} ",
                        st_line.best_fit
                    ),
                    BestFit::None => (),
                }
            };
            let best_fit = match st_line.best_fit {
                BestFit::ExprRegr => format!(
                    "Exponential ({:.1}%), R2={:.2}",
                    growth.unwrap_or(DEFAULT_RANK),
                    st_line
                        .exp_regr
                        .as_ref()
                        .expect("missing exp_regr")
                        .R_squared
                ),
                BestFit::LinRegr => format!(
                    "Linear ({:.1}%), R2={:.2}",
                    growth.unwrap_or(DEFAULT_RANK),
                    st_line
                        .lin_regr
                        .as_ref()
                        .expect("missing lin_regr")
                        .R_squared
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
    labels: Vec<String>,
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
            .map(|sl| ChartDataParameters::new(proc, metric, labels, sl)),
        None => {
            error!("Could not find process '{process}'");
            None
        }
    }
}

// PROCESS OPTIMIZED VERSION, but less generic.
// /// the the chart-data for a specific call-chain (within a process context)
// /// the process can not be derived from the call-chain as we only have a string-key with inbound processes,
// pub fn get_call_chain_chart_data(
//     data: &Stitched,
//     process_key: &str,
//     call_chain_key: &str,
//     metric: &str,
//     scope: &str,  // when scope is "end2end" we should not filter on process (to be implemented)
// ) -> Option<ChartDataParameters> {
//     match data
//         .call_chain
//         .iter()
//         .filter(|(proc, _)| proc == process_key)
//         .next()
//     {
//         Some((proc, ccd_vec)) => match ccd_vec
//             .iter()
//             .filter(|ccd| ccd.inboud_process_key == call_chain_key)
//             .next()
//         {
//             Some(ccd) => ccd
//                 .data
//                 .get_metric_stitched_line(metric)
//                 .map(|sl| ChartDataParameters::new(proc, metric, sl)),
//             None => {
//                 error!("Could not find call-chain '{call_chain_key}' within list for process '{process_key}'");
//                 None
//             }
//         },
//         None => {
//             error!("Could not find process '{process_key}'");
//             None
//         }
//     }
// }

/// the the chart-data for a specific call-chain and metric (within a process context)
/// the process can not be derived from the call-chain as we only have a string-key with inbound processes,
pub fn get_call_chain_chart_data(
    data: &Stitched,
    labels: Vec<String>,
    call_chain_key: &str,
    metric: &str,
) -> Option<ChartDataParameters> {
    let proc: Vec<_> = data
        .call_chain
        .iter()
        .flat_map(|(_k, ccd_vec)| ccd_vec.iter().filter(|ccd| ccd.full_key == call_chain_key))
        .collect();
    match proc.len() {
        0 => {
            error!("Could not find call-chain '{call_chain_key}'");
            None
        }
        n => {
            if n > 1 {
                error!("Observed {n} matches for key {call_chain_key}. Returning first match only");
            };
            proc[0]
                .data
                .get_metric_stitched_line(metric)
                .map(|sl| ChartDataParameters::new(call_chain_key, metric, labels, sl))
        }
    }
}

pub fn get_file_stats(data: &Stitched) -> Table {
    // let values = BASIC_REPORT_ITEMS
    //     .iter()
    //     .map(|sr| sr.extract_data(&data.basic))
    //     .collect();
    let values = data
        .basic
        .0
        .iter()
        .map(|sl| ChartLine {
            label: sl.label.to_owned(),
            data: sl.data.clone(),
        })
        .collect();

    Table {
        column_labels: get_label_list(data),
        data: values,
    }
}
