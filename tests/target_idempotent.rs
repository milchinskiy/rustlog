use std::{fs, path::PathBuf};

#[test]
fn later_set_target_is_ignored() {
    let mut p: PathBuf = std::env::temp_dir();
    p.push(format!("rustlog_idem_{}_{}.log",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));

    rustlog::set_file(&p).expect("set_file ok");
    rustlog::set_target(rustlog::Target::Stdout);

    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    rustlog::info!("sticky");

    let s = fs::read_to_string(&p).expect("read file");
    assert!(s.contains("sticky"), "second set_target must not override Writer; got:\n{s}");
    let _ = fs::remove_file(&p);
}
