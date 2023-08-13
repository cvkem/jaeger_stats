use std::collections::HashMap;

use crate::{aux::LinearRegression, StitchList};

use super::{
    call_chain_reporter::CCReportItems,
    method_stats_reporter::POReportItems,
    stitch_list::StitchSources,
    stitch_tables::{BASIC_REPORT_ITEMS, CALL_CHAIN_REPORT_ITEMS, PROC_OPER_REPORT_ITEMS},
};
use std::mem;

pub struct StitchedLine {
    pub label: String,
    pub data: Vec<Option<f64>>,
    pub lin_reg: LinearRegression,
}

type StitchedSet = Vec<StitchedLine>;

#[derive(Default)]
struct Stitched {
    /// the list of input-files (one per analysis) that are used. This list also
    pub sources: StitchSources,
    pub basic: StitchedSet,
    pub process_operation: Vec<(String, StitchedSet)>,
    pub call_chain: HashMap<String, StitchedSet>,
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
            .for_each(|sr| stitched.basic.push(sr.extract_stitched_line(&data)));

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
                    .push((po_key.to_string(), stitched_set))
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
                    .push((cc_key.to_string(), stitched_set))
            });

        stitched
    }
}
