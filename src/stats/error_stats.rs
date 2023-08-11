use crate::processed::{Span, Spans, SpansExt};

///  returns a tuple with the number of none-http-ok and the number of spans with error-lines
pub fn get_span_error_information(span: &Span) -> (Vec<i16>, Vec<String>) {
    let http_code = match span.http_status_code {
        Some(http_code) if http_code != 200 => vec![http_code],
        _ => Vec::with_capacity(0),
    };
    let logs = span
        .logs
        .iter()
        .filter_map(|log| {
            if log.level == "ERROR" {
                Some(log.msg.to_owned())
            } else {
                None
            }
        })
        .collect();
    (http_code, logs)
}

/// get the error information over a full call-chaing
pub fn get_cchain_error_information(idx: usize, spans: &Spans) -> (Vec<i16>, Vec<String>) {
    let zipped = SpansExt(spans).chain_apply_forward(idx, &get_span_error_information);
    let (http_codes, logs): (Vec<Vec<i16>>, Vec<Vec<String>>) = zipped.into_iter().unzip();
    let http_codes = http_codes.into_iter().flatten().collect();
    let logs = logs.into_iter().flatten().collect();
    (http_codes, logs)
}
