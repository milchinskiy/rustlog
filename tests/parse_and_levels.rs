mod helpers;
use core::str::FromStr;
use helpers::*;
use rustlog::{level, set_level, ColorMode, Level};

#[test]
fn color_mode_from_str_variants() {
    assert_eq!(ColorMode::from_str("always").ok(), Some(ColorMode::Always));
    assert_eq!(ColorMode::from_str("NEVER").ok(), Some(ColorMode::Never));
    assert_eq!(ColorMode::from_str("Auto").ok(), Some(ColorMode::Auto));
    assert_eq!(ColorMode::from_str("").ok(), Some(ColorMode::Auto));
    assert!(ColorMode::from_str("garbage").is_err());
}

#[test]
fn level_set_and_get_all_variants() {
    let _g = test_lock().lock().unwrap();
    // no writer needed, just test the atoms
    for &lvl in &[
        Level::Trace,
        Level::Debug,
        Level::Info,
        Level::Warn,
        Level::Error,
        Level::Fatal,
    ] {
        set_level(lvl);
        assert_eq!(level(), lvl);
    }
}

#[test]
fn compile_time_floor_matches_profile() {
    // debug builds include Trace; release builds strip Trace/Debug at compile time
    #[cfg(debug_assertions)]
    {
        assert!(rustlog::ct_enabled(Level::Trace));
        assert!(rustlog::ct_enabled(Level::Debug));
    }
    #[cfg(not(debug_assertions))]
    {
        assert!(!rustlog::ct_enabled(Level::Trace));
        assert!(!rustlog::ct_enabled(Level::Debug));
        assert!(rustlog::ct_enabled(Level::Info));
    }
}
