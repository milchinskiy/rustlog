#[test]
fn env_bool_zero_and_true_variants() {
    // Show time OFF via "0"
    std::env::set_var("RUST_LOG_SHOW_TIME", "0");
    // Show TID ON via mixed-case word
    std::env::set_var("RUST_LOG_SHOW_TID", "TrUe");

    rustlog::init_from_env();

    // Just ensure no panic; line content varies by features
    rustlog::info!("env-bools");

    std::env::remove_var("RUST_LOG_SHOW_TIME");
    std::env::remove_var("RUST_LOG_SHOW_TID");
}
