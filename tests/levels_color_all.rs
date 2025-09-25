#![cfg(feature = "color")]
mod helpers; use helpers::*;
use rustlog::{set_color_mode, ColorMode};
use rustlog::{trace, debug, info, warn, error, fatal};

fn has_ansi(s: &str) -> bool { s.contains("\u{1b}[") }

#[test]
fn color_per_level_all_arms() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    // Force color to ensure we hit the color path & mapping for each level
    set_color_mode(ColorMode::Always);

    trace!("t");
    debug!("d");
    info!("i");
    warn!("w");
    error!("e");
    fatal!("f");

    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(has_ansi(&text), "expected ANSI in output: {text}");

    assert!(text.contains("INFO"));
    assert!(text.contains("WARN"));
    assert!(text.contains("ERROR"));
    assert!(text.contains("FATAL"));

    #[cfg(debug_assertions)]
    {
        assert!(text.contains("TRACE"));
        assert!(text.contains("DEBUG"));
    }
}
