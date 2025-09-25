#![allow(dead_code)]

use std::io::{self, Write};
use std::sync::{Arc, Mutex, OnceLock};

// Global lock to serialize tests (logger has global state)
pub fn test_lock() -> &'static Mutex<()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
}

// Shared buffer for all tests (since writer/target are set once)
fn shared_buf() -> &'static Arc<Mutex<Vec<u8>>> {
    static BUF: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();
    BUF.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

#[derive(Clone)]
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

struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {}
}
pub fn attach_mem_sink() -> (Arc<Mutex<Vec<u8>>>, impl Drop) {
    static INIT: OnceLock<()> = OnceLock::new();
    let buf = shared_buf().clone();

    // Install writer+target only once (first test that runs)
    INIT.get_or_init(|| {
        rustlog::set_writer(Box::new(Mem(buf.clone())));
        rustlog::set_target(rustlog::Target::Writer);
    });

    (buf, Guard)
}

/// Reset runtime state
/// # Panics
pub fn reset_runtime() {
    // Deterministic output (no time/tid)
    rustlog::set_show_time(false);
    rustlog::set_show_thread_id(false);
    rustlog::set_level(rustlog::Level::Trace);

    // Clear buffer between tests
    let buf = shared_buf();
    buf.lock().unwrap().clear();
}

/// Extract lines from buffer
/// # Panics
pub fn lines_from(buf: &Arc<Mutex<Vec<u8>>>) -> Vec<String> {
    let bytes = buf.lock().unwrap().clone();
    String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
