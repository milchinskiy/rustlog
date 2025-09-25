mod helpers; use helpers::*;

#[test]
fn banner_has_trailing_newline() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    rustlog::banner();

    let bytes = buf.lock().unwrap().clone();
    assert!(bytes.ends_with(b"\n"), "banner must end with a newline");
}
