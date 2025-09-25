#[test]
fn set_file_returns_error_on_dir() {
    let dir = std::env::temp_dir();
    // set_file on a directory should error
    assert!(rustlog::set_file(&dir).is_err());
}
