#[test]
fn hits_stdout_branch() {
    rustlog::set_target(rustlog::Target::Stdout);
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    rustlog::info!("stdout"); // executes Stdout arm
}
