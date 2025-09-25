#![cfg(feature = "color")]

#[test]
fn stdout_auto_branch_executes_without_ansi() {
    rustlog::set_target(rustlog::Target::Stdout);
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);

    // No env override → Auto is default
    rustlog::info!("stdout-auto");
}
