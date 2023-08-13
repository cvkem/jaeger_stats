use super::{
    call_chain_reporter::{CCReportItem, CCReportItems, CallChainReporter},
    method_stats_reporter::{MethodStatsReporter, POReportItem, POReportItems},
    stats_rec_reporter::{SRReportItem, StatsRecReporterCSV},
};
use crate::{aux::TimeStats, stats::StatsRec};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BASIC_REPORT_ITEMS: Vec<SRReportItem> = vec![
        SRReportItem::new("num_files", |stats_rec| Some(stats_rec.num_files as f64)),
        SRReportItem::new("rate (req/sec)", |stats_rec| TimeStats(
            &stats_rec.duration_micros
        )
        .get_avg_rate(stats_rec.num_files)),
        SRReportItem::new("num_traces", |stats_rec| Some(
            stats_rec.trace_id.len() as f64
        )),
        SRReportItem::new("min_duration_millis", |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_min_millis()
        )),
        SRReportItem::new("median_duration_millis", |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_median_millis()
        )),
        SRReportItem::new("avg_duration_millis", |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_avg_millis()
        )),
        SRReportItem::new("max_duration_millis", |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_max_millis()
        ))
    ];
}

lazy_static! {
    pub static ref PROC_OPER_REPORT_ITEMS: POReportItems = POReportItems(vec![
        POReportItem::new("count", |&(msv, _, _)| Some(msv.count as f64)),
        POReportItem::new("Occurance percentage", |&(msv, _, num_traces)| Some(
            msv.count as f64 / num_traces as f64
        )),
        POReportItem::new("rate (avg)", |&(msv, num_files, _)| msv
            .get_avg_rate(num_files)),
        POReportItem::new("min_millis", |&(msv, _, _)| Some(msv.get_min_millis())),
        POReportItem::new("median_millis", |&(msv, _, _)| Some(
            msv.get_median_millis()
        )),
        POReportItem::new("avg_millis", |&(msv, _, _)| Some(msv.get_avg_millis())),
        POReportItem::new("max_millis", |&(msv, _, _)| Some(msv.get_max_millis())),
        POReportItem::new("frac_not_http_ok", |&(msv, _, _)| Some(
            msv.get_frac_not_http_ok()
        )),
        POReportItem::new("frac_error_logs", |&(msv, _, _)| Some(
            msv.get_frac_error_log()
        )),
    ]);
}

lazy_static! {
    pub static ref CALL_CHAIN_REPORT_ITEMS: CCReportItems = CCReportItems(vec![
        CCReportItem::new("count", |&(ccv, _, _)| Some(ccv.count as f64)),
        CCReportItem::new("Occurance percentage", |&(ccv, _, num_traces)| Some(
            ccv.count as f64 / num_traces as f64
        )),
        CCReportItem::new("rate (avg)", |&(ccv, num_files, _)| ccv
            .get_avg_rate(num_files)),
        CCReportItem::new("min_millis", |&(ccv, _, _)| Some(ccv.get_min_millis())),
        CCReportItem::new("median_millis", |&(ccv, _, _)| Some(
            ccv.get_median_millis()
        )),
        CCReportItem::new("avg_millis", |&(ccv, _, _)| Some(ccv.get_avg_millis())),
        CCReportItem::new("max_millis", |&(ccv, _, _)| Some(ccv.get_max_millis())),
        CCReportItem::new("frac_not_http_ok", |&(ccv, _, _)| Some(
            ccv.get_frac_not_http_ok()
        )),
        CCReportItem::new("frac_error_logs", |&(ccv, _, _)| Some(
            ccv.get_frac_error_log()
        )),
    ]);
}

///
///TODO The remainder of this document is legacy code to be discarded after CSV-output and tables have been extracted
///

fn add_table_tail_separator(buffer: &mut Vec<String>) {
    (0..3).for_each(|_| buffer.push(String::new())) // empty lines translate to newlines
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
pub fn append_basic_stats(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("# Basic statistics over alle stitched files".to_owned());

    let mut reporter = StatsRecReporterCSV::new(buffer, data, &*BASIC_REPORT_ITEMS);
    reporter.append_report();

    add_table_tail_separator(buffer);
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
pub fn append_method_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("# Method table".to_owned());

    // Build a reporter that handles shows the items defined in the report_items. Each item is a data-column.
    let mut reporter = MethodStatsReporter::new(buffer, data, &*PROC_OPER_REPORT_ITEMS);

    // Find all keys and generate an output line for each of these keys.
    let keys = reporter.get_keys();
    keys.into_iter().for_each(|key| reporter.append_report(key));

    add_table_tail_separator(buffer);
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
pub fn append_callchain_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("# Call-chain table".to_owned());
    // build the stack of reports that need to be calculated

    // Build a reporter that handles shows the items defined in the report_items. Each item is a data-column.
    let mut reporter = CallChainReporter::new(buffer, data, &*CALL_CHAIN_REPORT_ITEMS);

    // Find all keys and generate an output line for each of these keys.
    let keys = reporter.get_keys();
    keys.into_iter().for_each(|k| reporter.append_report(k));

    add_table_tail_separator(buffer);
}
