#![cfg(feature = "color")]
use rustlog::info;

#[test]
fn stdout_auto_no_ansi_in_ci() {
    rustlog::set_target(rustlog::Target::Stdout);
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    info!("stdout-auto");
}
