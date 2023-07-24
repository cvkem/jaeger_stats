


pub fn calc_rate(data: &Vec<i64>, num_outliers: i32) -> Option<f64> {
    assert!(num_outliers > 0);
    if data.len() as i32 - num_outliers - 2 < 0 {
        return None;
    }
    let mut data = data.clone();
    data.sort();
    // comppute the gaps (moving from N to N-1 datapoints)
    for i in 0..(data.len()-1) {
        data[i] = data[i+1] - data[i];
    }
    data.pop(); // drop last element

    // drop the expected a number of outliers
    data.sort();
    for i in 0..num_outliers {
        data.pop();
    }

    if data.len() == 0 {
        return None;
    }

    let t = data.iter().sum::<i64>() as f64/data.len() as f64/1e6;
    let rate = 1.0/t as f64;

    Some(rate)
}