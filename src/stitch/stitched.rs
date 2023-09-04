use std::collections::HashMap;

use crate::{
    utils::{self, CsvFileBuffer},
    StitchList,
};

use super::{
    anomalies::{Anomalies, AnomalyParameters},
    call_chain_reporter::CCReportItems,
    dataseries::DataSeries,
    proc_oper_stats_reporter::POReportItems,
    stitch_list::StitchSources,
    stitch_tables::{BASIC_REPORT_ITEMS, CALL_CHAIN_REPORT_ITEMS, PROC_OPER_REPORT_ITEMS},
    stitched_set::StitchedSet,
};
use std::{mem, path::Path};

#[derive(Debug)]
pub struct StitchParameters {
    pub drop_count: usize,
    pub anomaly_pars: AnomalyParameters,
}

#[derive(Default)]
pub struct Stitched {
    /// the list of input-files (one per analysis) that are used. This list also
    pub sources: StitchSources,
    pub basic: StitchedSet,
    pub process_operation: Vec<(String, StitchedSet)>,
    pub call_chain: Vec<(String, Vec<(String, StitchedSet)>)>,
}

impl Stitched {
    /// build a stitched dataset based on a StitchList.
    /// The contents of the Stiched dataset are defined in a series of tables:
    ///    1. `stitched_tables::BASIC_REPORT_ITEMS`: Some basic statistics for this dataset.
    ///    2. `stitched_tables::PROC_OPER_REPORT_ITEMS`:  A report on the level of Process/Operation.
    ///    3. `stitched_tables::CALL_CHAIN_REPORT_ITEMS`:  A detailed report where we compute separate statistics for each call-chain (call-path) that lead to a specific Process/Operation.
    pub fn build(mut stitch_list: StitchList, pars: &StitchParameters) -> Self {
        let sources = mem::take(&mut stitch_list.lines);
        let mut stitched = Self {
            sources,
            ..Self::default()
        };

        // this method reads the data in the original format, so data contains one column (StatsRec) per dataset
        let mut data = stitch_list.read_data();

        let num_dropped = DataSeries(&mut data).drop_low_volume_traces(pars.drop_count);
        println!(
            "Based on drop_count={} we have dropped {num_dropped} Processes over all datasets.",
            pars.drop_count
        );

        // add the basic report items as defined in stitch_tables::BASIC_REPORT_ITEMS.
        BASIC_REPORT_ITEMS.iter().for_each(|sr| {
            stitched
                .basic
                .0
                .push(sr.extract_stitched_line(&data, &pars.anomaly_pars))
        });

        POReportItems::get_keys(&data)
            .into_iter()
            .for_each(|po_key| {
                let key_data = POReportItems::extract_dataset(&data, &po_key);
                let stitched_set = PROC_OPER_REPORT_ITEMS
                    .0
                    .iter()
                    .map(|por| por.extract_stitched_line(&key_data, &pars.anomaly_pars))
                    .collect();
                stitched
                    .process_operation
                    .push((po_key.to_string(), StitchedSet(stitched_set)))
            });

        CCReportItems::get_keys(&data)
            .into_iter()
            .for_each(|(proc_oper, cc_keys)| {
                let call_chains = cc_keys
                    .into_iter()
                    .map(|cc_key| {
                        let key_data = CCReportItems::extract_dataset(&data, &cc_key);
                        let stitched_set = CALL_CHAIN_REPORT_ITEMS
                            .0
                            .iter()
                            .map(|por| por.extract_stitched_line(&key_data, &pars.anomaly_pars))
                            .collect();
                        (cc_key.to_string(), StitchedSet(stitched_set))
                    })
                    .collect();
                stitched.call_chain.push((proc_oper, call_chains))
            });

        stitched
    }

    /// Generate a header for a summary line showing as all metrics over a single statistic.
    /// When the statistic is 'Average' we already have a 'Count' column. However, when reporting over another Statistic an (average) Count column is prefixed to
    /// indicate the reliability of the computed statistic.
    pub fn summary_header(&self, table_type: &str, extra_count: bool) -> String {
        let col_headers = if self.process_operation.is_empty() {
            "NO DATA".to_owned()
        } else {
            self.process_operation[0]
                .1
                .summary_header(extra_count)
                .join("; ")
        };
        format!("{table_type}; {}", col_headers)
    }

    /// A full data-header is used when showing a time-series followed by the Linear-regression parameters of that time-series.
    pub fn full_data_header(&self, table_type: &[&str]) -> String {
        let table_type = table_type.join("; ");
        let col_headers =
            if self.process_operation.is_empty() || self.process_operation[0].1 .0.is_empty() {
                "NO DATA".to_owned()
            } else {
                self.process_operation[0].1 .0[0].headers()
            };
        format!("{table_type}; {}", col_headers)
    }

    /// Read all stitched data and write it out to a CSV files
    /// TODO: refactor to separate the CSV-output phase from the actual transposition and structuring of the data.
    pub fn write_csv(&self, path: &Path) {
        let mut csv = CsvFileBuffer::new();

        csv.add_empty_lines(2);
        csv.add_toc(10);

        csv.add_section("List of stitched data-files (numbered) and comments (unnumbered):");
        csv.append(&mut self.sources.csv_output());

        csv.add_section("Summary_statistics per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation", false));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    utils::floats_to_string(stitched_set.summary_avg(), " ;")
                ))
            });

        csv.add_section("Slope summary per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation", true));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    utils::floats_to_string(stitched_set.summary_slopes(), " ;")
                ))
            });

        csv.add_section("Scaled Slope summary per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation", true));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    utils::floats_to_string(stitched_set.summary_scaled_slopes(), " ;")
                ))
            });

        csv.add_section("Last-deviation-scaled summary per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation", true));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    utils::floats_to_string(stitched_set.summary_last_deviation_scaled(), " ;")
                ))
            });

        csv.add_section("Basic statistics per input file");
        csv.add_line(self.full_data_header(&["Input-files"]));
        csv.append(&mut self.basic.csv_output(&[""]));

        csv.add_section("Statistics per BSP/operation combination:");
        csv.add_line(self.full_data_header(&["BSP/operation"]));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| csv.append(&mut stitched_set.csv_output(&[&label])));

        csv.add_section(
            "Summary_statistics call-chain decending on count and grouped by BSP/operation",
        );
        csv.add_line(self.summary_header("BSP/operation", false));
        self.call_chain.iter().for_each(|(po_label, call_chains)| {
            csv.add_empty_lines(1);
            csv.add_line(self.summary_header(&format!("PROC_OPER: {po_label}"), false));
            call_chains.iter().for_each(|(cc_label, stitched_set)| {
                csv.add_line(format!(
                    "{cc_label}; {}",
                    utils::floats_to_string(stitched_set.summary_avg(), " ;")
                ))
            });
        });

        csv.add_section("Statistics per call-chain (path from the external end-point to the actual BSP/operation (detailled information):");
        csv.add_line(self.full_data_header(&["Full call-chain", "Process/Oper"]));
        self.call_chain.iter().for_each(|(po_label, call_chains)| {
            call_chains.iter().for_each(|(cc_label, stitched_set)| {
                csv.append(&mut stitched_set.csv_output(&[&cc_label, &po_label]))
            });
        });

        csv.write_file(path);
    }

    /// Add the anomalies on the Process/Operation-level to the 'csv'.
    fn add_process_operation_anomalies(
        &self,
        csv: &mut CsvFileBuffer,
        pars: &AnomalyParameters,
    ) -> usize {
        let mut num_anomalies = 0;

        let metrics: Vec<_> = PROC_OPER_REPORT_ITEMS
            .0
            .iter()
            .map(|por| por.label)
            .collect();
        metrics.iter().for_each(|metric| {
            csv.add_section(&format!("{metric} (Proces/Operation-level)"));

            csv.add_line(Anomalies::report_stats_line_header_str().to_owned());

            self.process_operation.iter().for_each(|(po, lines)| {
                lines
                    .0
                    .iter()
                    .filter(|s| s.label[..] == **metric)
                    .for_each(|line| {
                        if let Some(anomalies) = line.anomalies(pars) {
                            num_anomalies += 1;
                            csv.add_line(anomalies.report_stats_line(po))
                        }
                    })
            });
            csv.add_empty_lines(2);
        });

        num_anomalies
    }

    /// Add the anomalies on the Process/Operation-level to the 'csv'.
    fn add_call_chain_anomalies(&self, csv: &mut CsvFileBuffer, pars: &AnomalyParameters) -> usize {
        let mut num_anomalies = 0;

        let metrics: Vec<_> = CALL_CHAIN_REPORT_ITEMS
            .0
            .iter()
            .map(|ccr| ccr.label)
            .collect();
        metrics.iter().for_each(|metric| {
            csv.add_section(&format!("{metric} (Call-Chain-level)"));

            self.call_chain.iter().for_each(|(po_label, call_chains)| {
                csv.add_empty_lines(1);
                csv.add_line(format!("PROC_OPER: {po_label}"));
                csv.add_line(Anomalies::report_stats_line_header_str().to_owned());
                call_chains.iter().for_each(|(cc_label, stitched_set)| {
                    stitched_set
                        .0
                        .iter()
                        .filter(|s| s.label[..] == **metric)
                        .for_each(|line| {
                            if let Some(anomalies) = line.anomalies(pars) {
                                num_anomalies += 1;
                                csv.add_line(anomalies.report_stats_line(cc_label))
                            }
                        })
                })
            });
            csv.add_empty_lines(2);
        });

        num_anomalies
    }

    /// Filter the anonalies out of the full dataset based on three criteria:
    ///    1. Overall slope more than 1,05 (more than 5% increase per day)
    ///    2. Short term slope significant higher than the average slope over the full dataset (velocity of increase is ramping up)
    ///    3. The deviation for today is 2x higher than average L1-deviation
    /// The reporting happens per Measure and subsequently per BSP and the most important measures are handled first.
    /// On each line all three criteria are shown (with value and with a flag which values exceed the bound)
    pub fn write_anomalies_csv(&self, path: &Path, pars: &AnomalyParameters) -> usize {
        let mut csv = CsvFileBuffer::new();

        let mut num_anomalies = 0;

        csv.add_empty_lines(2);
        csv.add_toc(PROC_OPER_REPORT_ITEMS.0.len() + CALL_CHAIN_REPORT_ITEMS.0.len() + 2);

        num_anomalies += self.add_process_operation_anomalies(&mut csv, pars);
        num_anomalies += self.add_call_chain_anomalies(&mut csv, pars);

        if num_anomalies > 0 {
            csv.write_file(path);
        }
        num_anomalies
    }

    /// Take the process-operation data out of the record and return as a hashmap
    pub fn process_operation_as_hashmap(&mut self) -> HashMap<String, StitchedSet> {
        mem::take(&mut self.process_operation).into_iter().collect()
    }

    // /// Take the call_chain data out of the record and return as a hashmap
    // pub fn call_chain_as_hashmap(&mut self) -> HashMap<String, StitchedSet> {
    //     mem::take(&mut self.call_chain).into_iter().collect()
    // }
}
