mod helpers;
use helpers::*;
use rustlog::info;

#[test]
fn file_line_toggle() {
    let _g = test_lock().lock().unwrap();
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    rustlog::set_show_file_line(true);
    info!("a");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        text.contains('<') && text.contains(':') && text.contains('>'),
        "expected <file:line>: {text}"
    );

    reset_runtime();
    rustlog::set_show_file_line(false);
    info!("b");
    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        !text.contains('<') || !text.contains(':') || !text.contains('>'),
        "file:line should be hidden: {text}"
    );
}
