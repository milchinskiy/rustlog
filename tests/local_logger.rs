use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use rustlog::local::debug as ldebug;
use rustlog::local::info as linfo;
use rustlog::local::Logger;
use rustlog::{Level, Target};

#[derive(Clone, Default)]
struct Mem(Arc<Mutex<Vec<u8>>>);
impl Write for Mem {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Mem {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> (Box<dyn Write + Send>, Arc<Mutex<Vec<u8>>>) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        (Box::new(Self(buf.clone())) as Box<dyn Write + Send>, buf)
    }
}

#[test]
fn default_and_local_coexist_and_use_own_settings() -> io::Result<()> {
    let (g_writer, g_buf) = Mem::new();
    let (l_writer, l_buf) = Mem::new();

    rustlog::set_writer(g_writer);
    rustlog::set_target(Target::Writer);
    rustlog::set_level(Level::Info);
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    rustlog::set_show_group(false);
    rustlog::set_show_file_line(false);

    let lg = Logger::builder()
        .writer(l_writer)
        .level(Level::Trace)
        .show_time(false)
        .show_thread_id(false)
        .show_group(false)
        .show_file_line(true)
        .build_static()?; // ergonomic for macros

    rustlog::info!("G: info");
    rustlog::debug!("G: debug (filtered)");

    linfo!(lg, "L: info");
    ldebug!(lg, "L: debug");

    let g = String::from_utf8(g_buf.lock().unwrap().clone()).unwrap();
    let l = String::from_utf8(l_buf.lock().unwrap().clone()).unwrap();

    // Global writer only got its own INFO (no DEBUG, no local lines)
    assert!(g.contains("G: info"), "global should include INFO");
    assert!(
        !g.contains("G: debug"),
        "global should filter DEBUG at INFO level"
    );
    assert!(
        !g.contains("L: info"),
        "global writer must not receive local logs"
    );
    assert!(
        !g.contains("L: debug"),
        "global writer must not receive local logs"
    );

    // Local writer got its own INFO and DEBUG
    assert!(l.contains("L: info"), "local should include INFO");
    assert!(
        l.contains("L: debug"),
        "local should include DEBUG at TRACE level"
    );
    assert!(
        !l.contains("G: info"),
        "local writer must not receive global logs"
    );

    // File:line only enabled for the local instance
    let fname = file!();
    assert!(!g.contains(fname), "global should hide file:line");
    assert!(l.contains(fname), "local should show file:line");

    Ok(())
}
