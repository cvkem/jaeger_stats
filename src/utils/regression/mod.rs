mod exponential_regr;
mod linear_regr;

#[derive(Debug)]
pub struct DataPoint {
    x: f64,
    y: f64,
}

pub type DataSet = Vec<DataPoint>;

type Averages = (f64, f64);

pub use exponential_regr::ExponentialRegression;
pub use linear_regr::LinearRegression;
