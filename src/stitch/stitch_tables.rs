use super::{
    call_chain_reporter::{CCReportItem, CCReportItems},
    proc_oper_stats_reporter::{POReportItem, POReportItems},
    stats_rec_reporter::SRReportItem,
};
use crate::utils::TimeStats;
use lazy_static::lazy_static;

lazy_static! {
    /// NOTE/TODO: the interface of StatsRec is different from the interface of ProcOperStats and CCStats
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
        SRReportItem::new("avg_duration_millis", |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_avg_millis()
        )),
        SRReportItem::new("median_duration_millis", |stats_rec| TimeStats(&stats_rec.duration_micros).get_median_millis()),
        SRReportItem::new("median_p75_millis", |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.75)),
        SRReportItem::new("median_p90_millis", |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.90)),
        SRReportItem::new("median_p95_millis", |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.95)),
        SRReportItem::new("median_p99_millis", |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.99)),
        SRReportItem::new("max_duration_millis", |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_max_millis()
        ))
    ];
}

lazy_static! {
    pub static ref PROC_OPER_REPORT_ITEMS: POReportItems = POReportItems(vec![
        // The downstream analysis assumes that the first Report item is the Count measure!!
        POReportItem::new("count", |&(pov, _, _)| Some(pov.count as f64)),
        POReportItem::new("Occurance percentage", |&(pov, _, num_traces)| Some(
            pov.count as f64 / num_traces as f64
        )),
        POReportItem::new("rate (avg)", |&(pov, num_files, _)| pov
            .get_avg_rate(num_files)),
        POReportItem::new("min_millis", |&(pov, _, _)| Some(pov.get_min_millis())),
        POReportItem::new("avg_duration_millis", |&(pov, _, _)| Some(pov.get_avg_millis())),
        POReportItem::new("median_duration_millis", |&(pov, _, _)| pov.get_median_millis()),
        POReportItem::new("median_p75_millis", |&(pov, _, _)| pov.get_p_millis(0.75)),
        POReportItem::new("median_p90_millis", |&(pov, _, _)| pov.get_p_millis(0.90)),
        POReportItem::new("median_p95_millis", |&(pov, _, _)| pov.get_p_millis(0.95)),
        POReportItem::new("median_p99_millis", |&(pov, _, _)| pov.get_p_millis(0.99)),
        POReportItem::new("max_millis", |&(pov, _, _)| Some(pov.get_max_millis())),
        POReportItem::new("frac_not_http_ok", |&(pov, _, _)| Some(
            pov.get_frac_not_http_ok()
        )),
        POReportItem::new("frac_error_logs", |&(pov, _, _)| Some(
            pov.get_frac_error_log()
        )),
    ]);
}

lazy_static! {
    pub static ref CALL_CHAIN_REPORT_ITEMS: CCReportItems = CCReportItems(vec![
        // The downstream analysis assumes that the first Report item is the Count measure!!
        CCReportItem::new("count", |&(ccv, _, _)| Some(ccv.count as f64)),
        CCReportItem::new("Occurance percentage", |&(ccv, _, num_traces)| Some(
            ccv.count as f64 / num_traces as f64
        )),
        CCReportItem::new("rate (avg)", |&(ccv, num_files, _)| ccv
            .get_avg_rate(num_files)),
        CCReportItem::new("min_millis", |&(ccv, _, _)| Some(ccv.get_min_millis())),
        CCReportItem::new("avg_duration_millis", |&(ccv, _, _)| Some(ccv.get_avg_millis())),
        CCReportItem::new("median_duration_millis", |&(ccv, _, _)| ccv.get_median_millis()),
        CCReportItem::new("median_p75_millis", |&(ccv, _, _)| ccv.get_p_millis(0.75)),
        CCReportItem::new("median_p90_millis", |&(ccv, _, _)| ccv.get_p_millis(0.90)),
        CCReportItem::new("median_p95_millis", |&(ccv, _, _)| ccv.get_p_millis(0.95)),
        CCReportItem::new("median_p99_millis", |&(ccv, _, _)| ccv.get_p_millis(0.99)),
        CCReportItem::new("max_millis", |&(ccv, _, _)| Some(ccv.get_max_millis())),
        CCReportItem::new("frac_not_http_ok", |&(ccv, _, _)| Some(
            ccv.get_frac_not_http_ok()
        )),
        CCReportItem::new("frac_error_logs", |&(ccv, _, _)| Some(
            ccv.get_frac_error_log()
        )),
    ]);
}
