use super::call_descriptor::CallDescriptorStats;
use crate::{utils::AggregateData, Metric};

pub type EdgeValueSelector = fn(Option<&CallDescriptorStats>) -> Option<f64>;

pub fn edge_value_to_selector(edge_value: Metric) -> EdgeValueSelector {
    match edge_value {
        Metric::Count => |cds| cds.map(|ips| ips.count as f64),
        Metric::AvgDurationMillis => |cds| cds.and_then(|ips| ips.avg_duration_millis.get_value()),
        Metric::P75Millis => |cds| cds.and_then(|ips| ips.p75_millis.get_value()),
        Metric::P90Millis => |cds| cds.and_then(|ips| ips.p90_millis.get_value()),
        Metric::P95Millis => |cds| cds.and_then(|ips| ips.p95_millis.get_value()),
        Metric::P99Millis => |cds| cds.and_then(|ips| ips.p99_millis.get_value()),
        Metric::MaxDurationMillis => unimplemented!(),
        Metric::MedianDurationMillis => unimplemented!(),
        metric => panic!(
            "Uncovered metric '{}' provided as edge-value",
            metric.to_str()
        ),
    }
}
