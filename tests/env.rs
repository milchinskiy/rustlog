mod helpers; use helpers::*;
use rustlog::Level;

#[test]
fn init_from_env_sets_level_and_toggles() {
    let _g = test_lock().lock().unwrap();
    let (_buf, _guard) = attach_mem_sink();
    reset_runtime();

    std::env::set_var("RUST_LOG_LEVEL", "debug");
    std::env::set_var("RUST_LOG_SHOW_TIME", "0");
    std::env::set_var("RUST_LOG_SHOW_TID", "0");
    rustlog::init_from_env();
    assert_eq!(rustlog::level(), Level::Debug);

    std::env::set_var("RUST_LOG_LEVEL", "fatal");
    rustlog::init_from_env();
    assert_eq!(rustlog::level(), Level::Fatal);

    std::env::remove_var("RUST_LOG_LEVEL");
    std::env::remove_var("RUST_LOG_SHOW_TIME");
    std::env::remove_var("RUST_LOG_SHOW_TID");
}
