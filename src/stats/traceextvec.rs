use super::TraceExt;
use std::collections::HashSet;

pub struct TraceExtVec<'a>(pub &'a [TraceExt]);

impl<'a> TraceExtVec<'a> {
    pub fn num_files(&self) -> usize {
        let mut unique = HashSet::new();

        self.0.iter().for_each(|tre| {
            _ = unique.insert(tre.trace.source_file_id);
        });

        unique.len()
    }
}
