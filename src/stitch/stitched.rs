use std::collections::HashMap;

use crate::{
    aux::{
        floats_ref_to_string, floats_to_string, format_float_opt, write_string_to_file,
        LinearRegression,
    },
    stitch::csv_file::CsvFileBuffer,
    StitchList,
};

use super::{
    call_chain_reporter::CCReportItems,
    method_stats_reporter::POReportItems,
    stitch_list::StitchSources,
    stitch_tables::{
        append_method_table, BASIC_REPORT_ITEMS, CALL_CHAIN_REPORT_ITEMS, PROC_OPER_REPORT_ITEMS,
    },
};
use std::{mem, path::Path};

#[derive(Debug)]
pub struct StitchedLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub lin_reg: LinearRegression,
}

#[derive(Default, Debug)]
pub struct StitchedSet(Vec<StitchedLine>);

#[derive(Default)]
pub struct Stitched {
    /// the list of input-files (one per analysis) that are used. This list also
    pub sources: StitchSources,
    pub basic: StitchedSet,
    pub process_operation: Vec<(String, StitchedSet)>,
    pub call_chain: Vec<(String, StitchedSet)>,
}

impl StitchedSet {
    pub fn csv_output(&self, header: &str) -> Vec<String> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, line)| line.to_csv_string(header, idx))
            .collect()
    }

    pub fn summary_header(&self) -> Vec<String> {
        self.0.iter().map(|sl| sl.label.to_owned()).collect()
    }

    pub fn summary_avg(&self) -> Vec<Option<f64>> {
        self.0.iter().map(|sl| sl.avg()).collect()
    }
}

impl StitchedLine {
    pub fn avg(&self) -> Option<f64> {
        let values: Vec<_> = self.data.iter().filter_map(|x| *x).collect();
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }

    pub fn to_csv_string(&self, header: &str, idx: usize) -> String {
        // Produce the CSV_output
        let values = floats_ref_to_string(&self.data, "; ");

        let other_columns = 1 + idx * 4;
        let other_columns =
            (0..other_columns).fold(String::with_capacity(other_columns), |oc, _| oc + ";");
        format!(
            "{header}; {}; {values}; ; ; {}; {}; {};{other_columns};{}; {}; {};",
            self.label,
            format_float_opt(self.lin_reg.slope),
            format_float_opt(self.lin_reg.y_intercept),
            format_float_opt(self.lin_reg.R_squared),
            format_float_opt(self.lin_reg.slope),
            format_float_opt(self.lin_reg.y_intercept),
            format_float_opt(self.lin_reg.R_squared),
        )
    }
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

    /// Read all stitched data and write it out to a CSV files
    /// TODO: refactor to separate the CSV-output phase from the actual transposition and structuring of the data.
    pub fn write_csv(self, path: &Path) {
        let mut csv = CsvFileBuffer::new();

        csv.add_empty_lines(2);
        csv.add_toc(10);

        csv.add_section("List of stitched data-files (numbered) and comments (unnumbered):");
        csv.append(&mut self.sources.csv_output());

        csv.add_section("Summary_statistics per BSP-operation");
        let col_headers = if self.process_operation.is_empty() {
            "NO DATA".to_owned()
        } else {
            self.process_operation[0].1.summary_header().join("; ")
        };
        csv.add_line(format!("BSP/operation; {}", col_headers));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| {
                csv.add_line(format!(
                    "{label}; {}",
                    floats_to_string(stitched_set.summary_avg(), " ;")
                ))
            });

        csv.add_section("Basic statistics per input file");
        csv.append(&mut self.basic.csv_output(""));

        csv.add_section("Statistics per BSP/operation combination:");
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| csv.append(&mut stitched_set.csv_output(label)));

        csv.add_section("Statistics per call-chain (path from the external end-point to the actual BSP/operation (detailled information):");
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
