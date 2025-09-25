use rustlog::Level;

fn set_and_check(name: &str, val: &str, expect: Level) {
    std::env::set_var(name, val);
    rustlog::init_from_env();
    assert_eq!(rustlog::level(), expect, "{name}={val}");
    std::env::remove_var(name);
}

#[test]
fn env_levels_cover_all_variants() {
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);

    set_and_check("RUST_LOG_LEVEL", "trace", rustlog::Level::Trace);
    set_and_check("RUST_LOG_LEVEL", "debug", rustlog::Level::Debug);
    set_and_check("RUST_LOG_LEVEL", "info", rustlog::Level::Info);
    set_and_check("RUST_LOG_LEVEL", "warn", rustlog::Level::Warn);
    set_and_check("RUST_LOG_LEVEL", "error", rustlog::Level::Error);
    set_and_check("RUST_LOG_LEVEL", "fatal", rustlog::Level::Fatal);
}
