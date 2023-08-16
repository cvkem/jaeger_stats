use super::{
    call_chain_reporter::{CCReportItem, CCReportItems},
    method_stats_reporter::{MethodStatsReporter, POReportItem, POReportItems},
    stats_rec_reporter::SRReportItem,
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
