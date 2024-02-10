use super::call_descriptor::CallDescriptorStats;
use crate::{utils::AggregateData, EdgeValue};

pub type EdgeValueSelector = fn(Option<&CallDescriptorStats>) -> Option<f64>;

pub fn edge_value_to_selector(edge_value: EdgeValue) -> EdgeValueSelector {
    match edge_value {
        EdgeValue::Count => |cds| cds.map(|ips| ips.count as f64),
        EdgeValue::AvgMillis => |cds| cds.and_then(|ips| ips.avg_duration_millis.get_value()),
        EdgeValue::P75Millis => |cds| cds.and_then(|ips| ips.p75_millis.get_value()),
        EdgeValue::P90Millis => |cds| cds.and_then(|ips| ips.p90_millis.get_value()),
        EdgeValue::P95Millis => |cds| cds.and_then(|ips| ips.p95_millis.get_value()),
        EdgeValue::P99Millis => |cds| cds.and_then(|ips| ips.p99_millis.get_value()),
        EdgeValue::MaxMillis => unimplemented!(),
        EdgeValue::MedianMillis => unimplemented!(),
    }
}
