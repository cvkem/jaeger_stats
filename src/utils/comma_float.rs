use std::sync::Mutex;

static COMMA_FLOAT: Mutex<bool> = Mutex::new(false);

pub fn set_comma_float(val: bool) {
    let mut guard = COMMA_FLOAT.lock().unwrap();
    *guard = val
}

/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
pub fn format_float(val: f64) -> String {
    let s = format!("{}", val);
    if *COMMA_FLOAT.lock().unwrap() {
        s.replace('.', ",")
    } else {
        s
    }
}

/// format_float will format will replace the floating point '.' with a comma ',' such that the excel is readable in the Dutch Excel :-(
pub fn format_float_opt(val: Option<f64>) -> String {
    match val {
        Some(v) => format_float(v),
        None => "--".to_owned(),
    }
}

/// write a series of floats to a string without consuming them.
pub fn floats_ref_to_string(values: &[Option<f64>], sep: &str) -> String {
    values
        .iter()
        .map(|v| match v {
            Some(v) => format_float(*v),
            None => "".to_string(),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

/// write a series of floats to a string and consume the original input (could consume it with into_iter() which might be slightly more efficient)
pub fn floats_to_string(values: Vec<Option<f64>>, sep: &str) -> String {
    // floats_ref_to_string(&values, sep)
    values
        .into_iter()
        .map(|v| match v {
            Some(v) => format_float(v),
            None => "".to_string(),
        })
        .collect::<Vec<_>>()
        .join(sep)
}
