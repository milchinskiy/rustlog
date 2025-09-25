#![cfg(feature = "color")]
mod helpers; use helpers::*;
use rustlog::{info, set_color_mode, ColorMode};

fn has_ansi(s: &str) -> bool { s.contains("\u{1b}[") }

#[test]
fn color_mode_always_vs_never_vs_auto() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    // Always → expect ANSI
    set_color_mode(ColorMode::Always);
    info!("always");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(has_ansi(&text), "expected ANSI when ColorMode::Always, got: {text}");

    // Never → no ANSI
    reset_runtime();
    set_color_mode(ColorMode::Never);
    info!("never");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(!has_ansi(&text), "expected no ANSI when ColorMode::Never, got: {text}");

    // Auto with Writer → treat as non-TTY → no ANSI
    reset_runtime();
    set_color_mode(ColorMode::Auto);
    info!("auto");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(!has_ansi(&text), "expected no ANSI when Auto + Writer target, got: {text}");
}
