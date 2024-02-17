#[allow(non_camel_case_types)]
#[derive(thiserror::Error, Debug)]
pub enum ViewError {
    // correct propagation requires a clean-up of upstream error-handling
    // example: https://kerkour.com/rust-error-handling
    // #[error("Failed loading file")]
    // load_failure(#[from] dyn error::Error),
    #[error("Failed loading file {0} with error {1}")]
    load_failure(String, String),

    #[error("the file `{0}` is not found")]
    does_not_exist(String),

    #[error("Mismatch in length of the selection which contains {0} elements, while the original dataset has {1} columns.")]
    selection_failure(usize, usize),
}
