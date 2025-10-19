use rustlog::local::Logger;
use rustlog::{Level, Target};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
struct Mem(Arc<Mutex<Vec<u8>>>);
impl Write for Mem {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
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
fn local_scope_time_block_only_writes_to_local() -> io::Result<()> {
    let (g_writer, g_buf) = Mem::new();
    rustlog::set_writer(g_writer);
    rustlog::set_target(Target::Writer);
    rustlog::set_level(Level::Info);
    rustlog::set_show_time(false);
    rustlog::set_show_group(false);
    rustlog::set_show_file_line(false);

    let (l_writer, l_buf) = Mem::new();
    let lg = Logger::builder()
        .set_writer(l_writer)
        .set_level(Level::Trace)
        .set_show_time(false)
        .set_show_group(true)
        .set_show_file_line(true)
        .build_static()?;

    rustlog::local::scope_time!(lg, "block-run", {
        std::thread::sleep(Duration::from_millis(5));
    });

    let g = String::from_utf8(g_buf.lock().unwrap().clone()).unwrap();
    let l = String::from_utf8(l_buf.lock().unwrap().clone()).unwrap();

    assert!(
        l.contains("[block-run] took"),
        "label + 'took' should appear in local"
    );
    assert!(
        l.contains(file!()),
        "file:line should be present for local logger"
    );
    assert!(g.is_empty(), "global must remain empty");
    Ok(())
}
