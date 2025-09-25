#![cfg(feature = "timestamp")]
mod helpers; use helpers::*;
use rustlog::{info};

#[test]
#[cfg(all(feature = "timestamp", not(feature = "localtime")))]
fn utc_timestamp_prefix() {
    // With `timestamp` (and without `localtime`), lines should include a trailing "Z " before the level
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    rustlog::set_show_time(true);
    info!("ts");

    let s = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let line = s.lines().next().unwrap_or("");
    assert!(line.contains("Z "), "expected UTC 'Z ' marker before level: {line}");
}

#[cfg(feature = "localtime")]
#[test]
fn localtime_timestamp_prefix() {
    // With `localtime`, we should NOT end timestamps with 'Z '
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    rustlog::set_show_time(true);
    info!("ts");

    let s = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let line = s.lines().next().unwrap_or("");
    assert!(!line.contains("Z "), "localtime should not include 'Z ': {line}");
}
