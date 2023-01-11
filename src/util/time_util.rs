/*
    Utility functions related to time
*/

use chrono::offset::Local;
use std::thread::sleep;

use std::time::{Duration, SystemTime};

pub fn time_since(t: SystemTime) -> Duration {
    // Note: this function may panic in case of clock drift
    t.elapsed().unwrap()
}
pub fn div_durations(d1: Duration, d2: Duration) -> u128 {
    ((d1.as_nanos() as f64) / (d2.as_nanos() as f64)) as u128
}
pub fn nanos_timestamp(t: SystemTime) -> u128 {
    // Note: this function may panic in case of clock drift
    t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()
}
pub fn current_datetime_str() -> String {
    let out = Local::now().format("%Y-%m-%d-%H%M%S").to_string();
    println!("Current Datetime: {:?}", out);
    out
}
pub fn sleep_for_secs(s: u64) {
    sleep(Duration::from_secs(s));
}
