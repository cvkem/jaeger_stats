


pub fn calculate_frequency(data: &Vec<i64>, num_outliers: i32) -> Option<f64> {
    assert!(num_outliers > 0);
    println!("TMP  data-len {} and outliers={num_outliers}", data.len());
    if data.len() as i32 - num_outliers - 2 < 0 {
        return None;
    }
    let mut data = data.clone();
    data.sort();
    for i in 0..(data.len()-1) {
        data[i] = data[i+1] - data[i];
    }
    data.pop(); // drop last element
    data.sort();

    let idx: usize = data.len() - num_outliers as usize - 1;
    let T: i64 = data[idx];
    if T <= 0 {
        return None;
    }
    let rate = 1_000_000.0/T as f64;

    println!("TMP  T={T}  and rate={rate}");
    
    Some(rate)
}