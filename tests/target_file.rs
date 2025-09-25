use std::{fs, path::PathBuf};

#[test]
fn writes_to_file_target() {
    let mut p: PathBuf = std::env::temp_dir();
    p.push(format!("rustlog_{}.log", std::process::id()));

    rustlog::set_file(&p).expect("set_file ok");
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    rustlog::set_show_file_line(true);

    rustlog::info_group!("file", "hello {}", 42);

    let s = fs::read_to_string(&p).expect("read file");
    assert!(s.contains("hello 42"), "file content was:\n{s}");
    let _ = fs::remove_file(&p);
}
