use super::AggregateData;

/// Aggregator for data that is additive.
/// Actually this is a dummy for data that does not need a weighted average to be taken. For that type of data use AverageData.
#[derive(Debug)]
pub struct AdditiveData {
    count: u64,
    cumulator: Option<f64>,
}

impl AdditiveData {
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

impl AggregateData for AdditiveData {
    fn add(&mut self, count: u64, value: Option<f64>) {
        if let Some(value) = value {
            self.count += count;
            self.cumulator = Some(self.cumulator.map_or(value, |v| v + value));
        };
    }

    /// get the aggregate value
    fn get_value(&self) -> Option<f64> {
        self.cumulator
    }

    /// get the count of values
    fn get_count(&self) -> u64 {
        self.count
    }
}
