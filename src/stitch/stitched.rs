use std::{collections::HashMap, error::Error, fs, io};

use crate::{
    string_hash,
    utils::{self, CsvFileBuffer},
    view_api::Version,
    ServiceOperString, StitchList,
};

use super::{
    anomalies::{Anomalies, AnomalyParameters},
    call_chain_data::CallChainData,
    call_chain_reporter::CCReportItems,
    dataseries::DataSeries,
    legacy::LegacyStitched,
    proc_oper_stats_reporter::POReportItems,
    stitch_list::StitchSources,
    stitch_tables::{BASIC_REPORT_ITEMS, CALL_CHAIN_REPORT_ITEMS, PROC_OPER_REPORT_ITEMS},
    stitched_set::StitchedSet,
};
use serde::{Deserialize, Serialize};
use serde_json::{self};
use std::{mem, path::Path};

#[derive(Debug)]
pub struct StitchParameters {
    pub drop_count: usize,
    pub anomaly_pars: AnomalyParameters,
}

type ServiceOperList = Vec<(ServiceOperString, StitchedSet)>;

#[derive(Default, Serialize, Deserialize)]
pub struct Stitched {
    pub version: Version,
    /// the list of input-files (one per analysis) that are used.
    pub sources: StitchSources,
    pub basic: StitchedSet,
    pub service_operation: ServiceOperList,
    ///  call-chain is keyed by the Service/Operation and the values is a series of call-chains that end in this process/Oper
    /// The values is a Vector call-chains represent all different paths (call-chains) that terminate in de Process/Oper of the key of this vector.
    pub call_chain: Vec<(ServiceOperString, Vec<CallChainData>)>,
}

impl Stitched {
    pub fn new(
        sources: StitchSources,
        basic: StitchedSet,
        service_operation: ServiceOperList,
        call_chain: Vec<(ServiceOperString, Vec<CallChainData>)>,
    ) -> Self {
        Self {
            version: Version::new(0, 3),
            sources,
            basic,
            service_operation,
            call_chain,
        }
    }

    /// build a stitched dataset based on a StitchList.
    /// The contents of the Stiched dataset are defined in a series of tables:
    ///    1. `stitched_tables::BASIC_REPORT_ITEMS`: Some basic statistics for this dataset.
    ///    2. `stitched_tables::PROC_OPER_REPORT_ITEMS`:  A report on the level of Process/Operation.
    ///    3. `stitched_tables::CALL_CHAIN_REPORT_ITEMS`:  A detailed report where we compute separate statistics for each call-chain (call-path) that lead to a specific Process/Operation.
    pub fn build(mut stitch_list: StitchList, pars: &StitchParameters) -> Self {
        let sources = mem::take(&mut stitch_list.lines);

        // this method reads the data in the original format, so data contains one column (StatsRec) per dataset
        let mut data = stitch_list.read_data();

        //TODO: check if drop works as expected.
        let num_dropped = DataSeries(&mut data).drop_low_volume_traces(pars.drop_count);
        println!(
            "Based on drop_count={} we have dropped {num_dropped} Processes over all datasets.",
            pars.drop_count
        );

        // add the basic report items as defined in stitch_tables::BASIC_REPORT_ITEMS.
        let basic = StitchedSet(
            BASIC_REPORT_ITEMS
                .iter()
                .map(|sr| sr.extract_stitched_line(&data, &pars.anomaly_pars))
                .collect(),
        );

        let service_operation = POReportItems::get_keys(&data)
            .into_iter()
            .map(|po_key| {
                let key_data = POReportItems::extract_dataset(&data, &po_key);
                let stitched_set = PROC_OPER_REPORT_ITEMS
                    .0
                    .iter()
                    .map(|por| por.extract_stitched_line(&key_data, &pars.anomaly_pars))
                    .collect();
                (po_key.to_string(), StitchedSet(stitched_set))
            })
            .collect();

        let call_chain = CCReportItems::get_keys(&data)
            .into_iter()
            .map(|(proc_oper, cc_keys)| {
                let call_chains = cc_keys
                    .into_iter()
                    .map(|(cc_key, rooted)| {
                        let key_data = CCReportItems::extract_dataset(&data, &cc_key);
                        let stitched_set = CALL_CHAIN_REPORT_ITEMS
                            .0
                            .iter()
                            .map(|por| por.extract_stitched_line(&key_data, &pars.anomaly_pars))
                            .collect();
                        CallChainData {
                            full_key: cc_key.call_chain_key(),
                            inbound_process_key: cc_key.inbound_call_chain_key(),
                            rooted,
                            is_leaf: cc_key.is_leaf,
                            data: StitchedSet(stitched_set),
                        }
                    })
                    .collect();
                (proc_oper, call_chains)
            })
            .collect();

        Stitched::new(sources, basic, service_operation, call_chain)
    }

    // read from file (json or bincode)
    pub fn from_file(file_name: &str) -> Result<Self, Box<dyn Error>> {
        //let keep = path.clone().into_string().unwrap();
        let path_str = Path::new(file_name);
        let f = fs::File::open(path_str)?;
        let reader = io::BufReader::new(f);

        let Some(ext) = path_str.extension() else {
            panic!("Failed to find extension of '{}'", path_str.display());
        };

        let stitched: Self = match ext.to_str().unwrap() {
            "json" => {
                if let Ok(stitched) = serde_json::from_reader(reader) {
                    stitched
                } else {
                    println!("WARN: Fallback to Legacy-format to load data!!");
                    let sl = LegacyStitched::from_json(file_name)?;
                    sl.try_into()?
                }
            }
            "bincode" => {
                if let Ok(stitched) = bincode::deserialize_from(reader) {
                    stitched
                } else {
                    println!("WARN: Fallback to Legacy-format to load data!!");
                    let sl = LegacyStitched::from_bincode(file_name)?;
                    sl.try_into()?
                }
            }
            ext => panic!(
                "Unknown extension '{ext}'of inputfile {}",
                path_str.display()
            ),
        };
        Ok(stitched)
    }

    /// write the 'stitched' dataset to json
    pub fn to_json(&self, file_name: &str) {
        let path_str = Path::new(file_name);
        let f = fs::File::create(path_str).expect("Failed to open file");
        let writer = io::BufWriter::new(f);
        // on a large dataset to_write pretty takes 15.5 seconds while to_write takes 12 sec (so 30% extra for pretty printing to make it human readible)

        let Some(ext) = path_str.extension() else {
            panic!("Failed to find extension of '{}'", path_str.display());
        };

        match ext.to_str().unwrap() {
            "json" => match serde_json::to_writer_pretty(writer, self) {
                Ok(()) => (),
                Err(err) => panic!("failed to Serialize '{file_name}' to JSON!! {err:?}"),
            },
            "bincode" => match bincode::serialize_into(writer, self) {
                Ok(()) => (),
                Err(err) => panic!("failed to Serialize '{file_name}' to BINCODE!! {err:?}"),
            },
            ext => panic!(
                "Unknown extension '{ext}'of inputfile {}",
                path_str.display()
            ),
        };
    }

    /// Generate a header for a summary line showing as all metrics over a single statistic.
    /// When the statistic is 'Average' we already have a 'Count' column. However, when reporting over another Statistic an (average) Count column is prefixed to
    /// indicate the reliability of the computed statistic.
    pub fn summary_header(&self, table_type: &[&str], extra_count: bool) -> String {
        let col_headers = if self.service_operation.is_empty() {
            "NO DATA".to_owned()
        } else {
            self.service_operation[0]
                .1
                .summary_header(extra_count)
                .join("; ")
        };
        format!("{}; {}", table_type.join("; "), col_headers)
    }

    /// A full data-header is used when showing a time-series followed by the Linear-regression parameters of that time-series.
    pub fn full_data_header(&self, table_type: &[&str]) -> String {
        let table_type = table_type.join("; ");
        let col_headers =
            if self.service_operation.is_empty() || self.service_operation[0].1 .0.is_empty() {
                "NO DATA".to_owned()
            } else {
                self.service_operation[0].1 .0[0].headers()
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

        csv.add_section("Summary_statistics per Process/Operation");
        csv.add_line(self.summary_header(&["Process/Operation"], false));
        self.service_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    utils::floats_to_string(stitched_set.summary_avg(), " ;")
                ))
            });

        csv.add_section("Slope summary per Process/Operation");
        csv.add_line(self.summary_header(&["Process/Operation"], true));
        self.service_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    utils::floats_to_string(stitched_set.summary_slopes(), " ;")
                ))
            });

        csv.add_section("Scaled Slope summary per Process/Operation");
        csv.add_line(self.summary_header(&["Process/Operation"], true));
        self.service_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    utils::floats_to_string(stitched_set.summary_scaled_slopes(), " ;")
                ))
            });

        csv.add_section("Last-deviation-scaled summary per Process/Operation");
        csv.add_line(self.summary_header(&["Process/Operation"], true));
        self.service_operation
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

        csv.add_section("Statistics per Process/Operation combination:");
        csv.add_line(self.full_data_header(&["Process/Operation"]));
        self.service_operation
            .iter()
            .for_each(|(label, stitched_set)| csv.append(&mut stitched_set.csv_output(&[&label])));

        csv.add_section(
            "Summary_statistics call-chain decending on count and grouped by Process/Operation",
        );
        csv.add_line(self.summary_header(
            &[
                "Full Call-chain (path)",
                "cc_hash",
                "rooted",
                "is_leaf",
                "Process/Operation",
                "Inbound_chain",
            ],
            false,
        ));
        self.call_chain.iter().for_each(|(po_label, call_chains)| {
            call_chains.iter().for_each(|ccd| {
                csv.add_line(format!(
                    "{}; {}; {}; {}; {}; {}; {}",
                    ccd.full_key,
                    string_hash(&ccd.full_key),
                    if ccd.rooted { "rooted" } else { "" },
                    if ccd.is_leaf { "leaf" } else { "" },
                    po_label,
                    ccd.inbound_process_key,
                    utils::floats_to_string(ccd.data.summary_avg(), " ;")
                ))
            });
        });

        csv.add_section("Statistics per call-chain (path from the external end-point to the actual Process/Operation (detailled information):");
        csv.add_line(self.full_data_header(&[
            "Full call-chain (path)",
            "cc_hash",
            "rooted",
            "is_leaf",
            "Final Process/Oper",
            "Inbound_chain",
        ]));
        self.call_chain.iter().for_each(|(po_label, call_chains)| {
            call_chains.iter().for_each(|ccd| {
                csv.append(&mut ccd.data.csv_output(&[
                    &ccd.full_key,
                    &string_hash(&ccd.full_key),
                    if ccd.rooted { "rooted" } else { "" },
                    if ccd.is_leaf { "leaf" } else { "" },
                    &po_label,
                    &ccd.inbound_process_key,
                ]))
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
            .map(|por| por.metric)
            .collect();
        metrics.iter().for_each(|metric| {
            csv.add_section(&format!("{} (Proces/Operation-level)", metric.to_str()));

            csv.add_line(Anomalies::report_stats_line_header_str().to_owned());

            self.service_operation.iter().for_each(|(po, lines)| {
                lines
                    .0
                    .iter()
                    .filter(|s| s.metric == *metric)
                    .for_each(|line| {
                        if let Some(anomalies) = line.anomalies(pars) {
                            num_anomalies += 1;
                            csv.add_line(anomalies.report_stats_line(po, ""))
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
            .map(|ccr| ccr.metric)
            .collect();
        metrics.iter().for_each(|metric| {
            csv.add_section(&format!("{} (Call-Chain-level)", metric.to_str()));

            self.call_chain.iter().for_each(|(po_label, call_chains)| {
                csv.add_empty_lines(1);
                csv.add_line(format!("PROC_OPER: {po_label}"));
                csv.add_line(Anomalies::report_stats_line_header_str().to_owned());
                call_chains.iter().for_each(|ccd| {
                    ccd.data
                        .0
                        .iter()
                        .filter(|s| s.metric == *metric)
                        .for_each(|line| {
                            if let Some(anomalies) = line.anomalies(pars) {
                                num_anomalies += 1;
                                csv.add_line(
                                    anomalies
                                        .report_stats_line(&ccd.full_key, &ccd.inbound_process_key),
                                )
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
    /// The reporting happens per Measure and subsequently per Process and the most important measures are handled first.
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
        mem::take(&mut self.service_operation).into_iter().collect()
    }

    // /// Take the call_chain data out of the record and return as a hashmap
    // pub fn call_chain_as_hashmap(&mut self) -> HashMap<String, StitchedSet> {
    //     mem::take(&mut self.call_chain).into_iter().collect()
    // }
}
