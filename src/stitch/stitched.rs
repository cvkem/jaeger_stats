use std::collections::HashMap;

use crate::{aux::floats_to_string, StitchList};

use super::{
    call_chain_reporter::CCReportItems,
    csv_file::CsvFileBuffer,
    method_stats_reporter::POReportItems,
    stitch_list::StitchSources,
    stitch_tables::{BASIC_REPORT_ITEMS, CALL_CHAIN_REPORT_ITEMS, PROC_OPER_REPORT_ITEMS},
    stitched_set::StitchedSet,
};
use std::{mem, path::Path};

#[derive(Default)]
pub struct Stitched {
    /// the list of input-files (one per analysis) that are used. This list also
    pub sources: StitchSources,
    pub basic: StitchedSet,
    pub process_operation: Vec<(String, StitchedSet)>,
    pub call_chain: Vec<(String, StitchedSet)>,
}

impl Stitched {
    /// build a stitched dataset based on a StitchList
    pub fn build(mut stitch_list: StitchList) -> Self {
        let sources = mem::take(&mut stitch_list.lines);
        let mut stitched = Self {
            sources,
            ..Self::default()
        };

        let data = stitch_list.read_data();

        // add the basic report items as defined in stitch_tables::BASIC_REPORT_ITEMS.
        BASIC_REPORT_ITEMS
            .iter()
            .for_each(|sr| stitched.basic.0.push(sr.extract_stitched_line(&data)));

        POReportItems::get_keys(&data)
            .into_iter()
            .for_each(|po_key| {
                let key_data = POReportItems::extract_dataset(&data, &po_key);
                let stitched_set = PROC_OPER_REPORT_ITEMS
                    .0
                    .iter()
                    .map(|por| por.extract_stitched_line(&key_data))
                    .collect();
                stitched
                    .process_operation
                    .push((po_key.to_string(), StitchedSet(stitched_set)))
            });

        CCReportItems::get_keys(&data)
            .into_iter()
            .for_each(|cc_key| {
                let key_data = CCReportItems::extract_dataset(&data, &cc_key);
                let stitched_set = CALL_CHAIN_REPORT_ITEMS
                    .0
                    .iter()
                    .map(|por| por.extract_stitched_line(&key_data))
                    .collect();
                stitched
                    .call_chain
                    .push((cc_key.to_string(), StitchedSet(stitched_set)))
            });

        stitched
    }

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

    pub fn full_data_header(&self, table_type: &str) -> String {
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
    pub fn write_csv(self, path: &Path) {
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
                    floats_to_string(stitched_set.summary_avg(), " ;")
                ))
            });

        csv.add_section("Slope summary per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation", true));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    floats_to_string(stitched_set.summary_slopes(), " ;")
                ))
            });

        csv.add_section("Scaled Slope summary per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation", true));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    floats_to_string(stitched_set.summary_scaled_slopes(), " ;")
                ))
            });
        csv.add_section("Basic statistics per input file");
        csv.add_line(self.full_data_header("Input-files"));
        csv.append(&mut self.basic.csv_output(""));

        csv.add_section("Statistics per BSP/operation combination:");
        csv.add_line(self.full_data_header("BSP/operation"));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| csv.append(&mut stitched_set.csv_output(label)));

        csv.add_section("Statistics per call-chain (path from the external end-point to the actual BSP/operation (detailled information):");
        csv.add_line(self.full_data_header("Full call-chain"));
        self.call_chain
            .iter()
            .for_each(|(label, stitched_set)| csv.append(&mut stitched_set.csv_output(label)));

        csv.write_file(path);
    }

    /// Take the process-operation data out of the record and return as a hashmap
    pub fn process_operation_as_hashmap(&mut self) -> HashMap<String, StitchedSet> {
        mem::take(&mut self.process_operation).into_iter().collect()
    }

    /// Take the call_chain data out of the record and return as a hashmap
    pub fn call_chain_as_hashmap(&mut self) -> HashMap<String, StitchedSet> {
        mem::take(&mut self.call_chain).into_iter().collect()
    }
}
