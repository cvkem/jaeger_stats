use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::Path
};

use crate::raw_jaeger::JaegerTrace;

pub fn read_jaeger_trace_file<P: AsRef<Path>>(path: P) -> Result<JaegerTrace, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let jt = serde_json::from_reader(reader)?;

    // Return the `Jaeger_trace`.
    Ok(jt)
}
