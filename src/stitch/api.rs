use crate::{BestFit, Stitched, StitchedLine, StitchedSet};
use log::{error, info};
use serde::Serialize;
use serde_json;
use std::{error::Error, fs, io, time::Instant};

#[derive(Serialize)]
pub struct ProcessListItem {
    pub idx: usize,
    pub name: String,
    pub rank: f64,
}

pub fn get_process_list(data: &Stitched, metric: &str) -> Vec<ProcessListItem> {
    let mut proc_list: Vec<_> = data
        .process_operation
        .iter()
        .enumerate()
        .map(|(idx, po)| {
            let line =
                po.1.get_metric_stitched_line(metric)
                    .unwrap_or_else(|| panic!("Could not find ranking-metric '{}'", metric));
            let rank = line.periodic_growth().unwrap_or(-1000.0);

            ProcessListItem {
                idx: idx + 1,
                name: po.0.to_owned(),
                rank,
            }
        })
        .collect();
    proc_list.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());

    // renumber for new ordering
    proc_list
        .iter_mut()
        .enumerate()
        .for_each(|(idx, pli)| pli.idx = idx + 1);

    proc_list
}

#[derive(Serialize, Debug)]
pub struct ChartLine {
    label: String,
    data: Vec<Option<f64>>,
}

#[derive(Serialize, Debug)]
pub struct ChartDataParameters {
    pub title: String,
    pub labels: Vec<String>,
    pub lines: Vec<ChartLine>,
}

impl ChartDataParameters {
    pub fn new(process: &str, st_line: &StitchedLine) -> Self {
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
        ChartDataParameters {
            title: process.to_owned(),
            labels,
            lines,
        }
    }
}

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
            .map(|sl| ChartDataParameters::new(proc, sl)),
        None => {
            error!("Could not find process '{process}'");
            None
        }
    }
}
