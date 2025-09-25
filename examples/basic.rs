use rustlog::{
    init_from_env, set_level, set_target, Level, Target, banner,
    set_show_file_line, set_show_thread_id, set_show_time,
    trace, info, warn, error, fatal,
    scope_time, debug, info_group
};

fn main() {
    // Initialize from environment (optional):
    //   RUST_LOG_LEVEL=debug RUST_LOG_COLOR=always RUST_LOG_SHOW_TID=1 RUST_LOG_SHOW_TIME=1
    set_level(Level::Trace); // runtime threshold
    set_target(Target::Stderr); // default
    set_show_file_line(true);
    set_show_thread_id(true);
    set_show_time(true);
    init_from_env();
    
    banner();

    trace!("hello {}", "world");
    debug!("hello {}", "world");
    info!("hello {}", "world");
    warn!("disk almost full: {}%", 92);
    error!("disk almost full: {}%", 92);
    fatal!("disk almost full: {}%", 92);

    scope_time!("startup", {
        debug!("only visible at DEBUG+");
        std::thread::sleep(std::time::Duration::from_millis(10));
    });

    info_group!("net", "retry in {} ms", 200);
}
