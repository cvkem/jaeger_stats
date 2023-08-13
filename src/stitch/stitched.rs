use std::collections::HashMap;

use crate::{aux::{LinearRegression, write_string_to_file, floats_ref_to_string, format_float_opt}, StitchList};

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
    pub call_chain: HashMap<String, StitchedSet>,
}


impl StitchedSet {

    pub fn csv_output(&self, header: &str) -> Vec<String> {
        self.0.iter().enumerate().map(|(idx, line)| line.to_csv_string(header, idx)).collect()
    }

}

impl StitchedLine {

    pub fn to_csv_string(&self, header: &str ,idx: usize) -> String {
        // Produce the CSV_output
        let values = floats_ref_to_string(&self.data, "; ");

        let other_columns = 1 + idx * 4;
        let other_columns = (0..other_columns).fold(String::with_capacity(other_columns), |oc, _| oc + ";");
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
                    .process_operation
                    .push((cc_key.to_string(), StitchedSet(stitched_set)))
            });

        stitched
    }

    fn add_table_tail_separator(buffer: &mut Vec<String>) {
        (0..3).for_each(|_| buffer.push(String::new())) // empty lines translate to newlines
    }
    
    /// Read all stitched data and write it out to a CSV files
    /// TODO: refactor to separate the CSV-output phase from the actual transposition and structuring of the data.
    pub fn write_csv(self, path: &Path) {
        let mut csv_string = Vec::new();

        // add a table-of-contents of a few lines to be filled out later by
        // overwriting the line based on the position in the csv_strnig buffer.
        csv_string.push("Table of Contents of this file (starting rows of sections):".to_owned());
        (0..6).for_each(|_| csv_string.push(String::new()));


        csv_string.push("List of stitched data-files (numbered) and comments (unnumbered):".to_owned());
        csv_string[1] = format!(
            "\trow {}: Column_numbering (based on the input-files)",
            csv_string.len()
        );
        csv_string.append(&mut self.sources.csv_output());
        Self::add_table_tail_separator(&mut csv_string);

        csv_string[2] = format!("\trow {}: Basic statistics per input file:", csv_string.len());
        csv_string.append(&mut self.basic.csv_output(""));
        Self::add_table_tail_separator(&mut csv_string);

        csv_string[3] = format!(
            "\trow {}: Statistics per BSP/operation combination:",
            csv_string.len()
        );
        self.process_operation
            .iter()
            .for_each(|(label, stitched_set)| csv_string.append(&mut stitched_set.csv_output(label)));
        Self::add_table_tail_separator(&mut csv_string);

        csv_string[4] = format!("\trow {}: Statistics per call-chain (path from the external end-point to the actual BSP/operation (detailled information):", csv_string.len());
        self.call_chain
            .iter()
            .for_each(|(label, stitched_set)| csv_string.append(&mut stitched_set.csv_output(label)));
        Self::add_table_tail_separator(&mut csv_string);

        match write_string_to_file(path.to_str().unwrap(), csv_string.join("\n")) {
            Ok(()) => (),
            Err(err) => println!(
                "Writing file '{}' failed with Error: {err:?}",
                path.display()
            ),
        };
    }
}
