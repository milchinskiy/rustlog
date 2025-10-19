use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rustlog::{Level, Target};
use rustlog::local::{Logger};

#[derive(Clone, Default)]
struct Mem(Arc<Mutex<Vec<u8>>>);
impl Write for Mem {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
impl Mem {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> (Box<dyn Write + Send>, Arc<Mutex<Vec<u8>>>) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        (Box::new(Self(buf.clone())) as Box<dyn Write + Send>, buf)
    }
}

#[test]
fn scope_time_local_block_and_raii() -> io::Result<()> {
    // Local logger only: independent writer
    let (l_writer, l_buf) = Mem::new();
    let lg = Logger::builder()
        .set_writer(l_writer)
        .set_level(Level::Trace)
        .set_show_time(false)
        .set_show_group(true)      // show label as group
        .set_show_file_line(true)  // show <file:line>
        .build_static()?;

    // Block form: logs after the block
    rustlog::local::scope_time!(lg, "local-block", {
        std::thread::sleep(Duration::from_millis(5));
    });

    let after_block = String::from_utf8(l_buf.lock().unwrap().clone()).unwrap();
    assert!(after_block.contains("local-block"), "local block label should appear (group)");
    assert!(after_block.contains("took "), "local block should contain 'took {{HumanDuration}}'");
    assert!(after_block.contains(file!()), "local block should contain file:line");

    // RAII form: log only when the scope ends
    let before_len = l_buf.lock().unwrap().len();
    {
        rustlog::local::scope_time!(lg, "local-raii");
        std::thread::sleep(Duration::from_millis(5));
        let during_len = l_buf.lock().unwrap().len();
        assert_eq!(during_len, before_len, "local RAII should not log before scope ends");
    }

    let after_raii = String::from_utf8(l_buf.lock().unwrap().clone()).unwrap();
    assert!(after_raii.contains("local-raii"), "local RAII label should appear (group)");
    assert!(after_raii.contains("took "), "local RAII should contain 'took {{HumanDuration}}'");
    assert!(after_raii.contains(file!()), "local RAII should contain file:line");

    Ok(())
}

use std::sync::OnceLock as StdOnceLock;
static TEST_LOCK: StdOnceLock<Mutex<()>> = StdOnceLock::new();

#[test]
fn scope_time_global_block_and_raii() {
    let _guard = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

    // Global/default logger → g_buf
    let (g_writer, g_buf) = Mem::new();
    rustlog::set_writer(g_writer);
    rustlog::set_target(Target::Writer);

    // Ensure the runtime level allows whatever level TimerGuard uses (Info by default)
    rustlog::set_level(Level::Info);

    // Enable flags so assertions are stable
    rustlog::set_show_time(false);
    rustlog::set_show_group(true);
    rustlog::set_show_file_line(true);

    // Block form: logs after the block
    rustlog::scope_time!("global-block", {
        std::thread::sleep(Duration::from_millis(5));
    });

    let after_block = String::from_utf8(g_buf.lock().unwrap().clone()).unwrap();
    assert!(after_block.contains("global-block"), "global block label should appear (group)");
    assert!(after_block.contains("took "), "global block should contain 'took {{HumanDuration}}'");
    assert!(after_block.contains(file!()), "global block should contain file:line");

    // RAII form: log only when the scope ends
    let before_len = g_buf.lock().unwrap().len();
    {
        rustlog::scope_time!("global-raii");
        std::thread::sleep(Duration::from_millis(5));
        let during_len = g_buf.lock().unwrap().len();
        assert_eq!(during_len, before_len, "global RAII should not log before scope ends");
    }

    let after_raii = String::from_utf8(g_buf.lock().unwrap().clone()).unwrap();
    assert!(after_raii.contains("global-raii"), "global RAII label should appear (group)");
    assert!(after_raii.contains("took "), "global RAII should contain 'took {{HumanDuration}}'");
    assert!(after_raii.contains(file!()), "global RAII should contain file:line");
}
