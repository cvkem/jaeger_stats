use super::AggregateData;

///Aggregator for data that needs a weighted average based on count.
#[derive(Debug)]
pub struct AverageData {
    count: u64,
    cumulator: Option<f64>,
}

impl AverageData {
    /// create a new instance with the provided initial values.
    pub fn new(count: u64, value: Option<f64>) -> Self {
        let mut data = Self {
            count: 0,
            cumulator: None,
        };
        // run via add to prevent duplication of code (and risk of inconsistencies)
        data.add(count, value);
        data
    }
}

impl AggregateData for AverageData {
    fn add(&mut self, count: u64, value: Option<f64>) {
        if let Some(value) = value {
            self.count += count;
            self.cumulator = Some(self.cumulator.map_or(value, |v| v + value * count as f64));
        }
    }

    /// get the aggregate value
    fn get_value(&self) -> Option<f64> {
        match self.cumulator {
            Some(cumulator) => Some(cumulator / self.count as f64),
            None => None,
        }
    }

    /// get the count of values
    fn get_count(&self) -> u64 {
        self.count
    }
}
