use super::{
    call_chain_reporter::{CCReportItem, CCReportItems},
    proc_oper_stats_reporter::{POReportItem, POReportItems},
    stats_rec_reporter::SRReportItem,
};
use crate::{
    utils::{self, TimeStats},
    Metric,
};
use lazy_static::lazy_static;

lazy_static! {
    /// NOTE/TODO: the interface of StatsRec is different from the interface of ProcOperStats and CCStats
    pub static ref BASIC_REPORT_ITEMS: Vec<SRReportItem> = vec![
        SRReportItem::new(Metric::NumFiles, |stats_rec| Some(stats_rec.num_files as f64)),
        SRReportItem::new(Metric::Rate, |stats_rec| {
            let dt: Vec<_> = stats_rec.start_dt.iter().map(|dt| utils::datetime_to_micros(*dt)).collect();
            TimeStats(&dt)
                .get_avg_rate(stats_rec.num_files)
        }),
        SRReportItem::new(Metric::NumTraces, |stats_rec| Some(
            stats_rec.trace_id.len() as f64
        )),
        SRReportItem::new(Metric::NumEndpoints, |stats_rec| Some(stats_rec.num_endpoints as f64)),
        SRReportItem::new(Metric::NumIncompleteTraces, |stats_rec| Some(stats_rec.num_incomplete_traces as f64)),
        SRReportItem::new(Metric::NumCallChains, |stats_rec| Some(stats_rec.num_call_chains as f64)),
        SRReportItem::new(Metric::InitNumUnrootedCallChains, |stats_rec| Some(stats_rec.init_num_unrooted_cc as f64)),
        SRReportItem::new(Metric::NumFixes, |stats_rec| Some(stats_rec.num_fixes as f64)),
        SRReportItem::new(Metric::NumUnrootedCallChainsAfterFixes, |stats_rec| Some(stats_rec.num_unrooted_cc_after_fixes as f64)),
        SRReportItem::new(Metric::MinDurationMillis, |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_min_millis()
        )),
        SRReportItem::new(Metric::AvgDurationMillis, |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_avg_millis()
        )),
        SRReportItem::new(Metric::MedianDurationMillis, |stats_rec| TimeStats(&stats_rec.duration_micros).get_median_millis()),
        SRReportItem::new(Metric::P75Millis, |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.75)),
        SRReportItem::new(Metric::P90Millis, |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.90)),
        SRReportItem::new(Metric::P95Millis, |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.95)),
        SRReportItem::new(Metric::P99Millis, |stats_rec| TimeStats(&stats_rec.duration_micros).get_p_millis(0.99)),
        SRReportItem::new(Metric::MaxDurationMillis, |stats_rec| Some(
            TimeStats(&stats_rec.duration_micros).get_max_millis()
        ))
    ];
}

lazy_static! {
    pub static ref PROC_OPER_REPORT_ITEMS: POReportItems = POReportItems(vec![
        // The downstream analysis assumes that the first Report item is the Count measure!!
        POReportItem::new(Metric::Count, |&(pov, _, _)| Some(pov.count as f64)),
        POReportItem::new(Metric::OccurancePercentage, |&(pov, _, num_traces)| Some(
            pov.count as f64 / num_traces as f64
        )),
        POReportItem::new(Metric::Rate, |&(pov, num_files, _)| pov
            .get_avg_rate(num_files)),
        POReportItem::new(Metric::MinDurationMillis, |&(pov, _, _)| Some(pov.get_min_millis())),
        POReportItem::new(Metric::AvgDurationMillis, |&(pov, _, _)| Some(pov.get_avg_millis())),
        POReportItem::new(Metric::MedianDurationMillis, |&(pov, _, _)| pov.get_median_millis()),
        POReportItem::new(Metric::MaxDurationMillis, |&(pov, _, _)| Some(pov.get_max_millis())),
        POReportItem::new(Metric::P75Millis, |&(pov, _, _)| pov.get_p_millis(0.75)),
        POReportItem::new(Metric::P90Millis, |&(pov, _, _)| pov.get_p_millis(0.90)),
        POReportItem::new(Metric::P95Millis, |&(pov, _, _)| pov.get_p_millis(0.95)),
        POReportItem::new(Metric::P99Millis, |&(pov, _, _)| pov.get_p_millis(0.99)),
        POReportItem::new(Metric::FracNotHttpOk, |&(pov, _, _)| Some(
            pov.get_frac_not_http_ok()
        )),
        POReportItem::new(Metric::FracErrorLogs, |&(pov, _, _)| Some(
            pov.get_frac_error_log()
        )),
    ]);
}

lazy_static! {
    pub static ref CALL_CHAIN_REPORT_ITEMS: CCReportItems = CCReportItems(vec![
        // The downstream analysis assumes that the first Report item is the Count measure!!
        CCReportItem::new(Metric::Count, |&(ccv, _, _)| Some(ccv.count as f64)),
        CCReportItem::new(Metric::OccurancePercentage, |&(ccv, _, num_traces)| Some(
            ccv.count as f64 / num_traces as f64
        )),
        CCReportItem::new(Metric::Rate, |&(ccv, num_files, _)| ccv
            .get_avg_rate(num_files)),
        CCReportItem::new(Metric::MaxDurationMillis, |&(ccv, _, _)| Some(ccv.get_min_millis())),
        CCReportItem::new(Metric::AvgDurationMillis, |&(ccv, _, _)| Some(ccv.get_avg_millis())),
        CCReportItem::new(Metric::MedianDurationMillis, |&(ccv, _, _)| ccv.get_median_millis()),
        CCReportItem::new(Metric::P75Millis, |&(ccv, _, _)| ccv.get_p_millis(0.75)),
        CCReportItem::new(Metric::P90Millis, |&(ccv, _, _)| ccv.get_p_millis(0.90)),
        CCReportItem::new(Metric::P95Millis, |&(ccv, _, _)| ccv.get_p_millis(0.95)),
        CCReportItem::new(Metric::P99Millis, |&(ccv, _, _)| ccv.get_p_millis(0.99)),
        CCReportItem::new(Metric::MaxDurationMillis, |&(ccv, _, _)| Some(ccv.get_max_millis())),
        CCReportItem::new(Metric::FracNotHttpOk, |&(ccv, _, _)| Some(
            ccv.get_frac_not_http_ok()
        )),
        CCReportItem::new(Metric::FracErrorLogs, |&(ccv, _, _)| Some(
            ccv.get_frac_error_log()
        )),
    ]);
}
