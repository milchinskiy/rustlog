#![cfg(feature = "color")]
mod helpers;
use helpers::*;
use rustlog::info;

fn has_ansi(s: &str) -> bool {
    s.contains("\u{1b}[")
}

#[test]
fn env_color_always_enables_ansi() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    std::env::set_var("RUST_LOG_COLOR", "always");
    rustlog::init_from_env();
    info!("colored");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        has_ansi(&text),
        "expected ANSI when RUST_LOG_COLOR=always: {text}"
    );

    std::env::remove_var("RUST_LOG_COLOR");
}
