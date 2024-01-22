use super::AggregateData;

pub struct AdditiveData {
    count: u64,
    cumulator: f64,
}

impl AdditiveData {
    /// create a new instance with the provided initial values.
    pub fn new(count: u64, value: f64) -> Self {
        let cumulator = value;
        Self { count, cumulator }
    }
}
impl AggregateData for AdditiveData {
    fn add(&mut self, count: u64, value: f64) {
        self.count += count;
        self.cumulator += value;
    }

    /// get the aggregate value
    fn get_value(&self) -> f64 {
        self.cumulator
    }

    /// get the count of values
    fn get_count(&self) -> u64 {
        self.count
    }
}
