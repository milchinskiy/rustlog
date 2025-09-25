#[test]
fn writer_target_without_sink_does_not_panic() {
    rustlog::set_target(rustlog::Target::Writer);
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    rustlog::info!("no-sink");
}
