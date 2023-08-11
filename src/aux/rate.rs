use std::sync::Mutex;

static SHOW_OUTPUT: Mutex<bool> = Mutex::new(false);

#[allow(dead_code)]
pub fn set_show_rate_output(val: bool) {
    let mut guard = SHOW_OUTPUT.lock().unwrap();
    *guard = val
}

// returns an average and a median rate (after dropping the outliers)
pub fn calc_rate(data: &Vec<i64>, num_outliers: i32) -> Option<(f64, f64)> {
    assert!(num_outliers >= 0);
    if data.len() as i32 - num_outliers - 2 < 0 {
        return None;
    }
    let mut data = data.clone();
    data.sort_unstable();
    // comppute the gaps (moving from N to N-1 datapoints)
    for i in 0..(data.len() - 1) {
        data[i] = data[i + 1] - data[i];
    }
    data.pop(); // drop last element

    // drop the expected a number of outliers
    data.sort_unstable();

    if *SHOW_OUTPUT.lock().unwrap() {
        println!("Show the sorted data before skipping the outliers!!");
        data.iter()
            .enumerate()
            .for_each(|(idx, v)| println!("{idx}:  {v}    check:  {}", data[idx]));
    }

    for _i in 0..num_outliers {
        data.pop();
    }

    if data.is_empty() {
        return None;
    }

    let t_avg = data.iter().sum::<i64>() as f64 / data.len() as f64 / 1e6;
    let avg_rate = 1.0 / t_avg;

    let med_idx = data.len() / 2;
    let t_med = data[med_idx] as f64 / 1e6;
    let med_rate = 1.0 / t_med;

    if *SHOW_OUTPUT.lock().unwrap() {
        println!("  t_avg = {t_avg} ->  rate_avg={avg_rate} en t_med={t_med}  ->  med_rate {med_rate} voor  {} punten", data.len());
    }

    Some((avg_rate, med_rate))
}
