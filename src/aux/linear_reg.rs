
pub struct LinearRegression{
    pub slope: Option<f64>,
    pub intersect_y: Option<f64>,
    pub R_squared: Option<f64>,
}

impl LinearRegression {
    pub fn new(data: &Vec<Option<f64>>) -> Self {
        let slope = None;
        let intersect_y = None;
        let R_squared = None;
        Self{ slope, intersect_y, R_squared }
    }
}