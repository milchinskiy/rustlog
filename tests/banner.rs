mod helpers; use helpers::*;

#[test]
fn banner_prints_one_line() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    rustlog::banner();
    let lines = lines_from(&buf);
    assert!(!lines.is_empty());
    // Expect: "<name> <ver> (<mode>)"
    assert!(lines[0].contains(env!("CARGO_PKG_NAME")));
    assert!(lines[0].contains(env!("CARGO_PKG_VERSION")));
}
