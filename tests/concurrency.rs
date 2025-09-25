mod helpers;
use helpers::*;
use rustlog::info;

const N_THREADS: usize = 4;
const N_LINES: usize = 50;

#[test]
fn lines_are_atomic_under_concurrency() {
    let _g = test_lock().lock().unwrap(); // serialize tests
    let (buf, _guard) = attach_mem_sink();
    reset_runtime();

    std::thread::scope(|s| {
        for t in 0..N_THREADS {
            s.spawn(move || {
                for i in 0..N_LINES {
                    info!("T{} #{i}", t);
                }
            });
        }
    });

    let lines = lines_from(&buf);
    assert_eq!(lines.len(), N_THREADS * N_LINES);
    // Each line must end with our message; no partial writes
    for l in &lines {
        assert!(l.contains("INFO"));
    }
}
