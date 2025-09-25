mod helpers; use helpers::*;
use rustlog::{trace, debug, info, warn, error, fatal, Level};

fn has_msg(lines: &[String], token: &str, msg: &str) -> bool {
    lines.iter().any(|l| l.contains(token) && l.trim_end().ends_with(msg))
}

#[test]
fn filters_by_runtime_level() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    // runtime: INFO
    rustlog::set_level(Level::Info);
    trace!("t1"); debug!("d1"); info!("i1"); warn!("w1"); error!("e1"); fatal!("f1");
    let lines = lines_from(&buf);

    assert!(has_msg(&lines, "INFO",  "i1"));
    assert!(has_msg(&lines, "WARN",  "w1"));
    assert!(has_msg(&lines, "ERROR", "e1"));
    assert!(has_msg(&lines, "FATAL", "f1"));
    assert!(!has_msg(&lines, "TRACE", "t1"));
    assert!(!has_msg(&lines, "DEBUG", "d1"));

    // runtime: TRACE (TRACE/DEBUG may be compiled out in release)
    reset_runtime();
    trace!("t2"); debug!("d2"); info!("i2"); warn!("w2"); error!("e2"); fatal!("f2");
    let lines = lines_from(&buf);

    assert!(has_msg(&lines, "INFO",  "i2"));
    assert!(has_msg(&lines, "WARN",  "w2"));
    assert!(has_msg(&lines, "ERROR", "e2"));
    assert!(has_msg(&lines, "FATAL", "f2"));

    let trace_ok = !lines.iter().any(|l| l.contains("TRACE")) || has_msg(&lines, "TRACE", "t2");
    let debug_ok = !lines.iter().any(|l| l.contains("DEBUG")) || has_msg(&lines, "DEBUG", "d2");
    assert!(trace_ok, "TRACE token present but no matching message: {lines:#?}");
    assert!(debug_ok, "DEBUG token present but no matching message: {lines:#?}");
}
