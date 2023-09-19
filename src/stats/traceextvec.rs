use super::TraceExt;
use std::collections::HashSet;

pub struct TraceExtVec<'a>(pub &'a [TraceExt]);

impl<'a> TraceExtVec<'a> {
    /// count the number of files over the current set of traces
    pub fn num_files(&self) -> usize {
        let mut unique = HashSet::new();

        self.0.iter().for_each(|tre| {
            _ = unique.insert(tre.trace.source_file_id);
        });

        unique.len()
    }

    /// count the number of traces that report missing spans
    pub fn num_incomplete_traces(&self) -> usize {
        self.0
            .iter()
            .filter(|tr| !tr.trace.missing_span_ids.is_empty())
            .count()
    }
}
