mod helpers; use helpers::*;
use rustlog::{info_group, scope_time};

#[test]
fn group_and_scope_timer_emit_lines() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    info_group!("net", "retry {}", 3);
    scope_time!("startup", { std::thread::sleep(std::time::Duration::from_millis(2)); });

    let lines = lines_from(&buf);
    assert!(lines.iter().any(|l| l.contains("[net]") && l.ends_with("retry 3")));
    assert!(lines.iter().any(|l| l.contains("[startup]") && l.contains("took ")));
}
