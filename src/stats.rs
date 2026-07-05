pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    values.iter().sum::<f64>() / values.len() as f64
}
pub fn variance(values: &[f64]) -> f64 {
    if values.len() < 2 { return 0.0; }
    let m = mean(values);
    values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / values.len() as f64
}
pub fn stddev(values: &[f64]) -> f64 { variance(values).sqrt() }
pub fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() { return 0.0; }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();
    if n % 2 == 1 { values[n / 2] } else { (values[n / 2 - 1] + values[n / 2]) / 2.0 }
}
pub fn min(values: &[f64]) -> f64 { values.iter().cloned().fold(f64::INFINITY, f64::min) }
pub fn max(values: &[f64]) -> f64 { values.iter().cloned().fold(f64::NEG_INFINITY, f64::max) }
pub fn sum(values: &[f64]) -> f64 { values.iter().sum() }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_mean() { assert!((mean(&[1.0, 2.0, 3.0, 4.0]) - 2.5).abs() < 1e-9); }
    #[test] fn test_variance() { assert!((variance(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]) - 4.0).abs() < 1e-9; }
    #[test] fn test_stddev() { assert!(stddev(&[1.0, 2.0, 3.0, 4.0, 5.0]) > 1.4 && stddev(&[1.0, 2.0, 3.0, 4.0, 5.0]) < 1.6); }
    #[test] fn test_median_odd() { assert_eq!(median(vec![3.0, 1.0, 2.0]), 2.0); }
    #[test] fn test_median_even() { assert_eq!(median(vec![1.0, 2.0, 3.0, 4.0]), 2.5); }
    #[test] fn test_min_max() { assert_eq!(min(&[3.0, 1.0, 4.0, 1.0, 5.0]), 1.0); assert_eq!(max(&[3.0, 1.0, 4.0, 1.0, 5.0]), 5.0); }
    #[test] fn test_empty() { assert_eq!(mean(&[]), 0.0); assert_eq!(sum(&[]), 0.0); }
}
