#[test]
fn hits_stderr_branch() {
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    rustlog::info!("stderr"); // executes Stderr arm
}
