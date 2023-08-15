use std::collections::HashMap;

use crate::{aux::{LinearRegression, write_string_to_file, floats_ref_to_string, format_float_opt, floats_to_string}, StitchList, stitch::csv_file::CsvFileBuffer};

use super::{
    call_chain_reporter::CCReportItems,
    method_stats_reporter::POReportItems,
    stitch_list::StitchSources,
    stitch_tables::{BASIC_REPORT_ITEMS, CALL_CHAIN_REPORT_ITEMS, PROC_OPER_REPORT_ITEMS, append_method_table},
};
use std::{mem, path::Path};

#[derive(Debug)]
pub struct StitchedLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub lin_reg: LinearRegression,
}

#[derive(Default, Debug)]
pub struct StitchedSet (Vec<StitchedLine>);



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
        self.0.iter().enumerate().map(|(idx, line)| line.to_csv_string(header, idx)).collect()
    }

    pub fn summary_header(&self) -> Vec<String> {
        self.0.iter().map(|sl| sl.label.to_owned()).collect()
    }

    pub fn full_data_header(&self) -> String {
        if self.0.is_empty() {
             "NO DATA".to_owned()
        } else {
            self.0[0].headers()
        }
    }

    pub fn summary_avg(&self) -> Vec<Option<f64>>{
        self.0.iter().map(|sl| sl.avg()).collect()
    }

    pub fn summary_slopes(&self) -> Vec<Option<f64>>{
        self.0.iter().map(|sl| sl.avg()).collect()
    }

    pub fn summary_scaled_slopes(&self) -> Vec<Option<f64>>{
        self.0.iter().map(|sl| sl.scaled_slope()).collect()
    }

}

impl StitchedLine {

    pub fn avg(&self) -> Option<f64> {
        let values: Vec<_> = self.data.iter().filter_map(|x| *x).collect();
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>()/values.len() as f64)
        }
    }

 
    pub fn scaled_slope(&self) -> Option<f64> {
        // this should be changed to a map-chain
        if let Some(avg) = self.avg() {
            if let Some(slope) = self.lin_reg.slope {
                if avg.abs() > 1e-10 {
                    Some(slope/avg)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// return the headers that correspond to a data-row (assuming this Line is representative for data availability over the full dataset).
    pub fn headers(&self) -> String {
        let columns = self.data.iter().enumerate().map(|(idx, x)| if x.is_some() { format!("{}", idx+1) } else { format!("_{}", idx+1)}).collect::<Vec<_>>().join("; ");
        format!("label; {columns}; ; slope, y_intercept; R_squared; scaled_slope")
    }

    /// Show the current line as a string in the csv-format with a ';' separator
    pub fn to_csv_string(&self, header: &str ,idx: usize) -> String {
        // Produce the CSV_output
        let values = floats_ref_to_string(&self.data, "; ");

        let scaled_slope = if let Some(avg) = self.avg() {
            if let Some(slope) = self.lin_reg.slope {
                Some(self.lin_reg.slope.unwrap()/avg)
            } else {
                None
            }
        } else {
            None
        };

        format!(
            "{header}; {}; {values}; ; ; {}; {}; {}; {}",
            self.label,
            format_float_opt(self.lin_reg.slope),
            format_float_opt(self.lin_reg.y_intercept),
            format_float_opt(self.lin_reg.R_squared),
            format_float_opt(scaled_slope),
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


    pub fn summary_header(&self, table_type: &str) -> String {
        let col_headers = if self.process_operation.is_empty() { "NO DATA".to_owned() }  else { self.process_operation[0].1.summary_header().join("; ") };
        format!("{table_type}; {}", col_headers)
    }

    pub fn full_data_header(&self, table_type: &str) -> String {
        let col_headers = if self.process_operation.is_empty() || self.process_operation[0].1.0.is_empty()
         { "NO DATA".to_owned() }  else { self.process_operation[0].1.0[0].headers() };
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
        csv.add_line(self.summary_header("BSP/operation"));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| csv.add_line(format!("{label}; {}", floats_to_string(stitched_set.summary_avg(), " ;"))));

        csv.add_section("Slope summary per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation"));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| csv.add_line(format!("{label}; {}", floats_to_string(stitched_set.summary_slopes(), " ;"))));

        csv.add_section("Scaled Slope summary per BSP-operation");
        csv.add_line(self.summary_header("BSP/operation"));
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| csv.add_line(format!("{label}; {}", floats_to_string(stitched_set.summary_scaled_slopes(), " ;"))));    
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
        mem::take(&mut self.process_operation)
            .into_iter()
            .collect()
    }


    /// Take the call_chain data out of the record and return as a hashmap
    pub fn call_chain_as_hashmap(&mut self) -> HashMap<String, StitchedSet> {
        mem::take(&mut self.call_chain)
            .into_iter()
            .collect()
    }

}
