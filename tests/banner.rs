mod helpers; use helpers::*;

#[test]
fn banner_prints_one_line() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    rustlog::banner!();
    let lines = lines_from(&buf);
    assert!(!lines.is_empty());
    // Expect: "<name> <ver> (<mode>)"
    assert!(lines[0].contains(env!("CARGO_PKG_NAME")));
    assert!(lines[0].contains(format!("v{}", env!("CARGO_PKG_VERSION")).as_str()));
}

#[test]
fn banner_prints_own_meta() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    let app_name = "test-app-name";
    let app_version = "1.2.3";

    rustlog::banner!(app_name, app_version);
    let lines = lines_from(&buf);
    assert!(!lines.is_empty());
    assert!(lines[0].contains(app_name));
    assert!(lines[0].contains(format!("v{app_version}").as_str()));
}
