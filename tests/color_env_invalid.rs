#![cfg(feature = "color")]
mod helpers;
use helpers::*;
use rustlog::info;

fn has_ansi(s: &str) -> bool {
    s.contains("\u{1b}[")
}

#[test]
fn invalid_env_value_means_auto_no_ansi_on_writer() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    std::env::set_var("RUST_LOG_COLOR", "nonesuch");
    rustlog::init_from_env();

    info!("x");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        !has_ansi(&text),
        "Writer + Auto should produce no ANSI: {text}"
    );

    std::env::remove_var("RUST_LOG_COLOR");
}
