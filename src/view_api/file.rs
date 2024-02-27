use super::Viewer;
use crate::{stitch::StitchedDataSet, trace_analysis::TraceDataSet, ViewError};

/// load and build a viewer for a file, which is either based on a TraceDataSet or a StitchedDataset.
pub fn load_viewer(file_name: &str) -> Result<Box<dyn Viewer>, ViewError> {
    match TraceDataSet::from_file(file_name) {
        Ok(tds) => Ok(tds as Box<dyn Viewer>),
        Err(err_tds) => match StitchedDataSet::from_file(file_name) {
            Ok(sds) => Ok(sds),
            Err(err_sds) => Err(ViewError::load_error(
                err_tds.to_string(),
                err_sds.to_string(),
            )),
        },
    }
}
