

#[derive(Debug)]
struct DataPoint {
    x: f64,
    y: f64,
}

type DataSet = Vec<DataPoint>;

type Averages = (f64, f64);

#[derive(Debug)]
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
    let slope_denum = data.iter().fold(0.0, |acc, dp| acc + f64::powi(dp.x - avg_xy.0, 2));
    let slope = slope_num / slope_denum;
    let y_intercept = avg_xy.1 - avg_xy.0 * slope;
    (slope, y_intercept)
}

fn get_R_squared(data: &DataSet, avg_xy: &Averages, slope: f64, y_intercept: f64) -> f64 {
    let sum_expect_sqr: f64 = data
        .iter()
        .map(|dp| {
            let expect = y_intercept + dp.x * slope;
            f64::powi(dp.y - expect, 2)
        })
        .sum();
    if sum_expect_sqr.abs() < 1e-100 {
        // Safeguard for horizontal lines 
        return 1.0
    } else {
        let sum_avg_sqr: f64 = data.iter().map(|dp| f64::powi(dp.y - avg_xy.1, 2)).sum();

        let res = 1.0 - sum_expect_sqr / sum_avg_sqr;
        res
    }
}



#[cfg(test)]
mod tests {
    use crate::aux::LinearRegression;

    fn test_float(val: Option<f64>, expect: Option<f64>) -> bool {
        match val {
            Some(val) => match expect {
                Some(expect) => f64::abs(val - expect) < 1e-6,
                None => false
            },
            None => expect.is_none()
        }
    }

    #[test]
    fn horizontal_line() {
        let input = vec![Some(1.0), Some(1.0)];

        let lr = LinearRegression::new(&input);
        println!("{lr:?}");

        assert!(test_float(lr.slope, Some(0.0)), "Slope incorrect");
        assert!(test_float(lr.y_intercept, Some(1.0)), "y_intersect incorrect");
        assert!(test_float(lr.R_squared, Some(1.0)), "R_squared incorrect");
    }

    #[test]
    fn horizontal_line_R_non_opt() {
        let input = vec![Some(1.0), Some(1.1), Some(1.0)];

        let lr = LinearRegression::new(&input);

        assert!(test_float(lr.slope, Some(0.0)), "Slope incorrect");
        assert!(test_float(lr.y_intercept, Some(1.03333333333333)), "y_intersect incorrect");
        assert!(test_float(lr.R_squared, Some(0.0)), "R_squared incorrect: {:?}", lr.R_squared);
    }

    #[test]
    fn nearly_horizontal_line() {
        let input = vec![Some(1.0), Some(1.0), Some(1.1)];

        let lr = LinearRegression::new(&input);
        println!("{lr:?}");

        assert!(test_float(lr.slope, Some(0.050000000000000044)), "Slope incorrect");
        assert!(test_float(lr.y_intercept, Some(0.9333333333333333)), "y_intersect incorrect");
        assert!(test_float(lr.R_squared, Some(0.75)), "R_squared incorrect: {:?}", lr.R_squared);
    }

    #[test]
    fn lectures_test() {
        // exmple taken from source: https://www.ncl.ac.uk/webtemplate/ask-assets/external/maths-resources/statistics/regression-and-correlation/coefficient-of-determination-r-squared.html
        let input = vec![None, Some(2.0), Some(4.0), Some(6.0), None, Some(7.0)];

        let lr = LinearRegression::new(&input);

        // Expected solution 
        // y = 0.143+1.229    and r2 = 0.895
        println!("{lr:?}");
        assert!(test_float(lr.slope, Some(1.2285714285714286)), "Slope incorrect: {:?}", lr.slope);
        assert!(test_float(lr.y_intercept, Some(0.14285714285714235)), "y_intersect incorrect: {:?}", lr.y_intercept);
        assert!(test_float(lr.R_squared, Some(0.8953995157384989)), "R_squared incorrect: {:?}", lr.R_squared);
    }


}