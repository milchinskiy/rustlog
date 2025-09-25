mod helpers; use helpers::*;
use rustlog::{info_group};

#[test]
fn group_visibility_toggle() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    // Default: group shown
    info_group!("g1", "a");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(text.contains("[g1]"), "expected group tag when enabled: {text}");

    // Hide group
    reset_runtime();
    rustlog::set_show_group(false);
    info_group!("g2", "b");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(!text.contains("[g2]"), "group tag should be suppressed when set_show_group(false): {text}");
}
