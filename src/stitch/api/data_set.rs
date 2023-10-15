use super::super::Stitched;
use super::{
    types::{ChartDataParameters, ProcessList, Table},
    utils,
};
use log::{error, info};
use std::{error, path::Path};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // correct propagation requires a clean-up of upstream error-handling
    // example: https://kerkour.com/rust-error-handling
    // #[error("Failed loading file")]
    // load_failure(#[from] dyn error::Error),
    #[error("Failed loading file {0} with error {1}")]
    load_failure(String, String),

    #[error("the file `{0}` is not found")]
    does_not_exist(String),
}

pub struct StitchedDataSet {
    /// current dataset used for most of the operations
    current: Box<Stitched>,
    /// The original dataset in case we are working on a selection of the original data, or None if current is the original dataset
    original: Option<Box<Stitched>>,
}

impl StitchedDataSet {
    pub fn new(data: Stitched) -> Self {
        Self {
            current: Box::new(data),
            original: None,
        }
    }

    pub fn from_file(file_name: &str) -> Result<Self, Error> {
        if Path::new(file_name).exists() {
            info!("Trying to load the file {file_name}");

            match Stitched::from_json(file_name) {
                Ok(data) => {
                    info!("Ready loading file");
                    Ok(Self::new(data))
                }
                Err(err) => Err(Error::load_failure(
                    file_name.to_owned(),
                    format!("{err:?}"),
                )),
            }
        } else {
            let msg = format!("ERROR: File '{file_name} does not exist");
            error!("{msg}");
            Err(Error::does_not_exist(file_name.to_owned()))
        }
    }

    pub fn get_process_list(&self, metric: &str) -> ProcessList {
        utils::get_process_list(&self.current, metric)
    }

    pub fn get_call_chain_list(
        &self,
        proc_oper: &str,
        metric: &str,
        scope: &str,
        inbound_idx: Option<i64>,
    ) -> ProcessList {
        utils::get_call_chain_list(&self.current, proc_oper, metric, scope, inbound_idx)
    }

    pub fn get_proc_oper_chart_data(
        &self,
        process: &str,
        metric: &str,
    ) -> Option<ChartDataParameters> {
        utils::get_proc_oper_chart_data(&self.current, process, metric)
    }

    pub fn get_call_chain_chart_data(
        &self,
        call_chain_key: &str,
        metric: &str,
    ) -> Option<ChartDataParameters> {
        utils::get_call_chain_chart_data(&self.current, call_chain_key, metric)
    }

    pub fn get_file_stats(&self) -> Table {
        utils::get_file_stats(&self.current)
    }

    // TODO: implement
    // get_label_list,
}
