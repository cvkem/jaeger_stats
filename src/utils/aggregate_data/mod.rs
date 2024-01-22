mod additive_data;
mod average_data;

pub use additive_data::AdditiveData;
pub use average_data::AverageData;

/// Keep track of the Aggregate data while hidding logic for aggregation
pub trait AggregateData {
    /// this function allows to get a copy with the same aggregation rules (basically preventing the need for a separate factory method)
    /// However, next function does not allow making a trait-object out of it, so omitting it.
    //    fn proto(&self) -> impl AggregateData;
    /// Add a value and the count to allow for a weighted aggregate computtion
    fn add(&mut self, count: u64, value: f64);
    /// get the aggregate value
    fn get_value(&self) -> f64;
    /// get the count of values
    fn get_count(&self) -> u64;
}
