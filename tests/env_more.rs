mod helpers; use helpers::*;
use rustlog::Level;

#[test]
fn invalid_level_keeps_previous() {
    let _g = test_lock().lock().unwrap();
    reset_runtime();

    rustlog::set_level(Level::Warn);
    std::env::set_var("RUST_LOG_LEVEL", "not-a-level");
    rustlog::init_from_env();
    assert_eq!(rustlog::level(), Level::Warn);

    std::env::remove_var("RUST_LOG_LEVEL");
}

#[test]
fn boolean_env_variants_are_accepted() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    std::env::set_var("RUST_LOG_SHOW_TID", "TRUE");
    std::env::set_var("RUST_LOG_SHOW_TIME", "1");
    rustlog::init_from_env();

    rustlog::info!("env-variants");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(text.contains("env-variants"));

    std::env::remove_var("RUST_LOG_SHOW_TID");
    std::env::remove_var("RUST_LOG_SHOW_TIME");
}
