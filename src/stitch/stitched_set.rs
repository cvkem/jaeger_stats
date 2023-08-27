use super::stitched_line::StitchedLine;
use std::iter;

#[derive(Default, Debug)]
pub struct StitchedSet(pub Vec<StitchedLine>);

impl StitchedSet {
    pub fn csv_output(&self, header: &str) -> Vec<String> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, line)| line.to_csv_string(header, idx))
            .collect()
    }

    pub fn summary_header(&self, extra_count: bool) -> Vec<String> {
        let headers = self.0.iter().map(|sl| sl.label.to_owned());
        if extra_count {
            let count_label = self
                .0
                .first()
                .map(|sl| sl.label.to_uppercase())
                .unwrap_or("NO DATA".to_owned());
            iter::once(count_label).chain(headers).collect()
        } else {
            headers.collect()
        }
    }

    pub fn full_data_header(&self) -> String {
        if self.0.is_empty() {
            "NO DATA".to_owned()
        } else {
            self.0[0].headers()
        }
    }

    pub fn summary_avg(&self) -> Vec<Option<f64>> {
        self.0.iter().map(|sl| sl.data_avg).collect()
    }

    pub fn summary_slopes(&self) -> Vec<Option<f64>> {
        // NOTE: here we assume first line always is a count
        let count = self.0.first().and_then(|data| data.data_avg);
        iter::once(count)
            .chain(
                self.0
                    .iter()
                    .map(|sl| sl.lin_reg.as_ref().map(|lr| lr.slope)),
            )
            .collect()
    }

    pub fn summary_last_deviation_scaled(&self) -> Vec<Option<f64>> {
        // NOTE: here we assume first line always is a count
        let count = self.0.first().and_then(|data| data.data_avg);
        iter::once(count)
            .chain(self.0.iter().map(|sl| sl.last_deviation_scaled()))
            .collect()
    }

    pub fn summary_scaled_slopes(&self) -> Vec<Option<f64>> {
        // NOTE: here we assume first line always is a count
        let count = self.0.first().and_then(|sl| sl.data_avg);
        iter::once(count)
            .chain(self.0.iter().map(|sl| sl.scaled_slope()))
            .collect()
    }
}
