struct DataPoint {
    x: f64,
    y: f64,
}

type DataSet = Vec<DataPoint>;

type Averages = (f64, f64);

pub struct LinearRegression {
    pub slope: Option<f64>,
    pub y_intercept: Option<f64>,
    pub R_squared: Option<f64>,
}

impl LinearRegression {
    pub fn new(data: &Vec<Option<f64>>) -> Self {
        let data = get_dataset(data);
        if data.len() < 2 {
            // insufficient data to compute a value
            Self {
                slope: None,
                y_intercept: None,
                R_squared: None,
            }
        } else {
            let avg_xy = get_average_xy(&data);
            let (slope, y_intercept) = get_slope_intercept(&data, &avg_xy);
            let R_squared = get_R_squared(&data, &avg_xy, slope, y_intercept);
            Self {
                slope: Some(slope),
                y_intercept: Some(y_intercept),
                R_squared: Some(R_squared),
            }
        }
    }
}

/// Get the averages over x and y for all filled values, where x = counting from 1 to N
fn get_dataset(data: &Vec<Option<f64>>) -> DataSet {
    data.iter()
        .enumerate()
        .filter_map(|(idx, val)| match val {
            Some(val) => Some(DataPoint {
                x: idx as f64 + 1.0,
                y: *val,
            }),
            None => None,
        })
        .collect()
}

/// Get the averages over x and y for all filled values, where x = counting from 1 to N
fn get_average_xy(data: &DataSet) -> Averages {
    let avg_x = data.iter().map(|dp| dp.x).sum::<f64>() / data.len() as f64;
    let avg_y = data.iter().map(|dp| dp.y).sum::<f64>() / data.len() as f64;
    (avg_x, avg_y)
}

fn get_slope_intercept(data: &DataSet, avg_xy: &Averages) -> (f64, f64) {
    let slope_num = data
        .iter()
        .fold(0.0, |acc, dp| acc + (dp.x - avg_xy.0) * (dp.y - avg_xy.1));
    let slope_denum = data.iter().fold(0.0, |acc, dp| acc + (dp.x - avg_xy.0));
    let slope = slope_num / slope_denum;
    let y_intercept = avg_xy.1 - avg_xy.0 * slope;
    (slope, y_intercept)
}

fn get_R_squared(data: &DataSet, avg_xy: &Averages, slope: f64, y_intercept: f64) -> f64 {
    let sqr = |x| x * x;
    let sum_exp_sqr: f64 = data
        .iter()
        .map(|dp| sqr(dp.y - y_intercept + dp.x * slope))
        .sum();
    let sum_avg_sqr: f64 = data.iter().map(|dp| sqr(dp.y - avg_xy.1)).sum();

    1.0 - sum_exp_sqr / sum_avg_sqr
}
