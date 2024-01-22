use super::AggregateData;

pub struct AverageData {
    count: u64,
    cumulator: f64,
}

impl AverageData {
    /// create a new instance with the provided initial values.
    pub fn new(count: u64, value: f64) -> Self {
        let cumulator = value;
        Self { count, cumulator }
    }
}
impl AggregateData for AverageData {
    fn add(&mut self, count: u64, value: f64) {
        self.count += count;
        self.cumulator += value * count as f64;
    }

    /// get the aggregate value
    fn get_value(&self) -> f64 {
        self.cumulator / self.count as f64
    }

    /// get the count of values
    fn get_count(&self) -> u64 {
        self.count
    }
}
