use std::{
    error::Error,
    fmt::Debug,
    fs::{self, File},
    io::{BufReader, Read},
    path::Path,
};

use super::jaeger::JaegerTrace;
use crate::utils::{self, Chapter};

use encoding_rs::Encoding;

/// check the Byte Order Mark (= BOM) of the file to find the current encoding.
fn check_bom<P: AsRef<Path>>(path: P) -> Result<&'static Encoding, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut buffer = [0_u8; 5];

    match f.read(&mut buffer) {
        Ok(num) if num >= 3 => (),
        Ok(num) => println!("Did read only {num} bytes. At least 3 needed"),
        Err(err) => println!("Failed reading BOM with error {err}"),
    }
    if let Some((enc, _size)) = Encoding::for_bom(&buffer) {
        Ok(enc)
    } else {
        Err("No BOM")?
    }
}

pub fn read_jaeger_trace_file<P: AsRef<Path> + Copy + Debug>(
    path: P,
) -> Result<JaegerTrace, Box<dyn Error>> {
    let jt = match check_bom(path) {
        Ok(encoding) => {
            // an encoding is found, so we need to decode and to drop the BOM as serde can not handle it.
            // beware, this consumes quite a bit of memory as the data is present 3 times (raw, decoded and as json)
            let file_size = fs::metadata(path)?.len();
            utils::report(
                Chapter::Details,
                format!(
                    "File {path:?}: Found encoding {encoding:?} for a file with size: {file_size}"
                ),
            );
            let f = File::open(path)?;
            let mut reader = BufReader::new(f);
            let mut buffer = Vec::with_capacity(file_size.try_into()?);
            reader.read_to_end(&mut buffer)?;
            let (s, malformed) = encoding.decode_with_bom_removal(buffer.as_slice());
            if malformed {
                utils::report(
                    Chapter::Issues,
                    format!("File {:?} returned a signal Malformed", path),
                );
            }
            serde_json::from_str(&s)?
        }
        Err(err) => {
            utils::report(
                Chapter::Details,
                format!("File {path:?}: Failed to find encoding: {err:?}"),
            );
            // Open the file in read-only mode with buffer.
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            println!("About to read trace via serde");
            // Read the JSON contents of the file as an instance of `User`.
            serde_json::from_reader(reader)?
        }
    };

    // Return the `Jaeger_trace`.
    Ok(jt)
}
