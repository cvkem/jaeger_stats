use super::{
    call_chain_reporter::{CCReportItem, CallChainReporter},
    key::Key,
    method_stats_reporter::{MSReportItem, MethodStatsReporter},
    stats_rec_reporter::{SRReportItem, StatsRecReporter},
};
use crate::{
    aux::{format_float, TimeStats},
    stats::StatsRec,
};

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
pub fn append_basic_stats(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("\n\n# Basic statistics over alle stitched files".to_owned());
    let mut report_items = Vec::new();
    report_items.push(SRReportItem::new("num_files", |stats_rec| {
        Some(stats_rec.num_files as f64)
    }));
    report_items.push(SRReportItem::new("rate (req/sec)", |stats_rec| {
        TimeStats(&stats_rec.duration_micros).get_avg_rate(stats_rec.num_files)
    }));
    report_items.push(SRReportItem::new("num_traces", |stats_rec| {
        Some(stats_rec.trace_id.len() as f64)
    }));
    report_items.push(SRReportItem::new("min_duration_millis", |stats_rec| {
        Some(TimeStats(&stats_rec.duration_micros).get_min_millis())
    }));
    report_items.push(SRReportItem::new("median_duration_millis", |stats_rec| {
        Some(TimeStats(&stats_rec.duration_micros).get_median_millis())
    }));
    report_items.push(SRReportItem::new("avg_duration_millis", |stats_rec| {
        Some(TimeStats(&stats_rec.duration_micros).get_avg_millis())
    }));
    report_items.push(SRReportItem::new("max_duration_millis", |stats_rec| {
        Some(TimeStats(&stats_rec.duration_micros).get_max_millis())
    }));

    let mut reporter = StatsRecReporter::new(buffer, data, report_items);
    reporter.append_report()
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
pub fn append_method_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("\n\n# Method table".to_owned());

    // build the stack of reports that need to be calculated
    let mut report_items = Vec::new();
    report_items.push(MSReportItem::new("count", |msv, _, _| {
        Some(msv.count as f64)
    }));
    report_items.push(MSReportItem::new(
        "Occurance percentage",
        |msv, _, num_traces| Some(msv.count as f64 / num_traces as f64),
    ));
    report_items.push(MSReportItem::new("rate (avg)", |msv, num_files, _| {
        msv.get_avg_rate(num_files)
    }));
    //    report_items.push(MSReportItem::new("rate (median)", |msv, num_files, _| msv.get_median_rate_str(num_files)));
    report_items.push(MSReportItem::new("min_millis", |msv, _, _| {
        Some(msv.get_min_millis())
    }));
    report_items.push(MSReportItem::new("median_millis", |msv, _, _| {
        Some(msv.get_median_millis())
    }));
    report_items.push(MSReportItem::new("avg_millis", |msv, _, _| {
        Some(msv.get_avg_millis())
    }));
    report_items.push(MSReportItem::new("max_millis", |msv, _, _| {
        Some(msv.get_max_millis())
    }));
    report_items.push(MSReportItem::new("frac_not_http_ok", |msv, _, _| {
        Some(msv.get_frac_not_http_ok())
    }));
    report_items.push(MSReportItem::new("frac_error_logs", |msv, _, _| {
        Some(msv.get_frac_error_log())
    }));

    // Build a reporter that handles shows the items defined in the report_items. Each item is a data-column.
    let mut reporter = MethodStatsReporter::new(buffer, data, report_items);

    // Find all keys and generate an output line for each of these keys.
    let keys = reporter.get_keys();
    keys.into_iter()
        .for_each(|Key { process, operation }| reporter.append_report(process, operation));
}

/// Find all potential 'method/operation' key, loop over these keys and write a csv-line per metric
pub fn append_callchain_table(buffer: &mut Vec<String>, data: &Vec<Option<StatsRec>>) {
    buffer.push("\n\n# Call-chain table".to_owned());
    // build the stack of reports that need to be calculated
    let mut report_items = Vec::new();
    report_items.push(CCReportItem::new("count", |msv, _, _| {
        Some(msv.count as f64)
    }));
    report_items.push(CCReportItem::new(
        "Occurance percentage",
        |msv, _, num_traces| Some(msv.count as f64 / num_traces as f64),
    ));
    report_items.push(CCReportItem::new("rate (avg)", |msv, num_files, _| {
        msv.get_avg_rate(num_files)
    }));
    //    report_items.push(CCReportItem::new("rate (median)", |msv, num_files, _| msv.get_median_rate_str(num_files)));
    report_items.push(CCReportItem::new("min_millis", |msv, _, _| {
        Some(msv.get_min_millis())
    }));
    report_items.push(CCReportItem::new("avg_millis", |msv, _, _| {
        Some(msv.get_avg_millis())
    }));
    report_items.push(CCReportItem::new("median_millis", |msv, _, _| {
        Some(msv.get_median_millis())
    }));
    report_items.push(CCReportItem::new("max_millis", |msv, _, _| {
        Some(msv.get_max_millis())
    }));

    report_items.push(CCReportItem::new("http_not_ok_count", |msv, _, _| {
        Some(msv.get_frac_not_http_ok())
    }));
    report_items.push(CCReportItem::new("num_error_logs", |msv, _, _| {
        Some(msv.get_frac_error_log())
    }));

    // Build a reporter that handles shows the items defined in the report_items. Each item is a data-column.
    let mut reporter = CallChainReporter::new(buffer, data, report_items);

    // Find all keys and generate an output line for each of these keys.
    let keys = reporter.get_keys();
    keys.into_iter().for_each(|k| reporter.append_report(k));
}
