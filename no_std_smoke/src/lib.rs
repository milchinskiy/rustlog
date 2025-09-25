#![no_std]

use rustlog::{
    Level, set_level,
    trace, debug, info, warn, error, fatal,
    scope_time, info_group,
};

pub fn smoke() {
    set_level(Level::Trace);
    trace!("t");
    debug!("d");
    info!("i");
    warn!("w");
    error!("e");
    fatal!("f");
    scope_time!("s", { /* no-op */ });
    info_group!("g", "x");
}
