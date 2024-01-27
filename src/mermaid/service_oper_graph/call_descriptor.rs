use super::{link_type::LinkType, loc::Loc};
use crate::{
    mermaid::trace_data::TraceDataStats,
    utils::{AggregateData, AverageData},
};

#[derive(Debug)]
pub struct CallDescriptorStats {
    /// the number of times this call is performed (cumulative over multiple inbound calls)
    pub count: u64,

    /// statistics aggregated over all paths
    pub avg_duration_millis: AverageData,
    pub p75_millis: AverageData,
    pub p90_millis: AverageData,
    pub p95_millis: AverageData,
    pub p99_millis: AverageData,
}

impl CallDescriptorStats {
    fn new(data: &TraceDataStats) -> Self {
        let count = data.count;
        Self {
            count: data.count,
            avg_duration_millis: AverageData::new(data.count, Some(data.avg_duration_millis)),
            p75_millis: AverageData::new(data.count, data.p75_millis),
            p90_millis: AverageData::new(data.count, data.p90_millis),
            p95_millis: AverageData::new(data.count, data.p95_millis),
            p99_millis: AverageData::new(data.count, data.p90_millis),
        }
    }

    /// Update all aggregate statistics in place
    fn update(&mut self, data: &TraceDataStats) {
        self.avg_duration_millis
            .add(data.count, Some(data.avg_duration_millis));
        self.p75_millis.add(data.count, data.p75_millis);
        self.p90_millis.add(data.count, data.p90_millis);
        self.p95_millis.add(data.count, data.p95_millis);
        self.p99_millis.add(data.count, data.p99_millis);
    }
}

/// Collection of data belonging to a specific inbound call (coming from a specific location, i.e. the (Service-)Operation that holds this call-descriptor)
/// The (to_service, to_oper) define a position in the ServiceOperGraph, where the first index is the service and the second is the operation.
#[derive(Debug)]
pub struct CallDescriptor {
    /// Defines the Service this CallDescriptor is calling into
    pub to_service: usize,
    /// Defines the Operation within to_service this CallDescriptor is calling into
    pub to_oper: usize,

    /// Statistics for all (outbound) calls from this location
    pub stats: CallDescriptorStats,
    /// Statistics for all (outbound) calls that originate from the currently selected inbound-call-path (if selected)
    pub inbound_path_stats: Option<CallDescriptorStats>,

    /// LinkType determines how this edge (call) will be displayed
    pub line_type: LinkType,
}

impl CallDescriptor {
    /// create a new CallDescriptor where 'stats' are populated with the provided 'data' and 'inbound_path_stats' are set to None.
    pub fn new(loc: Loc, data: &TraceDataStats) -> Self {
        Self {
            to_service: loc.service_idx,
            to_oper: loc.oper_idx,
            stats: CallDescriptorStats::new(data),
            inbound_path_stats: None,
            line_type: LinkType::Default,
        }
    }

    /// Update all statistics based on 'data'
    pub fn update(&mut self, data: &TraceDataStats) {
        self.stats.update(data)
    }

    pub fn add_inbound_stats(&mut self, data: &TraceDataStats) {
        // the imperative way
        match self.inbound_path_stats.as_mut() {
            Some(cds) => cds.update(data),
            None => self.inbound_path_stats = Some(CallDescriptorStats::new(data)),
        }

        // // the functional update
        // self.inbound_path_stats = Some(self
        //     .inbound_path_stats
        //     .as_mut()
        //     .map_or_else(
        //         || CallDescriptorStats::new(data) ,
        //         |v| {v.update(data); *v}))  // Ugly as we update in place and then return the modified value (mix of functional and imperative)
    }
}
