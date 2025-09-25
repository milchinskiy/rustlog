#![cfg(feature = "thread-id")]
mod helpers;
use helpers::*;
use rustlog::info;

#[test]
fn thread_id_toggle() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    rustlog::set_show_thread_id(true);
    info!("tid-on");
    let s = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        s.contains('['),
        "expected TID bracket when enabled, got: {s}"
    );

    reset_runtime();
    rustlog::set_show_thread_id(false);
    info!("tid-off");
    let s = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(!s.contains("[tid-on]"), "false positive on message token");
}
