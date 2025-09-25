#![warn(missing_docs, unsafe_code)]
//! A minimal logging crate.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod imp {
    use core::fmt::Arguments;
    use std::io::{self, IsTerminal, Write};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
    use std::sync::{Mutex as StdMutex, OnceLock};
    use std::time::Instant;

    // ===== Levels =====
    /// Log levels
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
    #[repr(u8)]
    pub enum Level {
        /// Trace
        Trace = 0,
        /// Debug
        Debug,
        /// Info
        Info,
        /// Warn
        Warn,
        /// Error
        Error,
        /// Fatal
        Fatal,
    }

    // ===== Compile-time minimum level (simplified) =====
    // In debug builds, include all levels (Trace+).
    // In release builds, compile out TRACE/DEBUG entirely for zero overhead.
    #[cfg(debug_assertions)]
    const CT_MIN: Level = Level::Trace;
    #[cfg(not(debug_assertions))]
    const CT_MIN: Level = Level::Info;
    static RUNTIME_LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);
    static SHOW_TID: AtomicBool = AtomicBool::new(cfg!(feature = "thread-id"));
    static SHOW_TIME: AtomicBool = AtomicBool::new(cfg!(feature = "timestamp"));
    static SHOW_GROUP: AtomicBool = AtomicBool::new(true);
    static SHOW_FILE_LINE: AtomicBool = AtomicBool::new(cfg!(feature = "file-line"));

    /// Color mode
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    #[repr(u8)]
    pub enum ColorMode {
        /// Auto
        Auto,
        /// Always
        Always,
        /// Never
        Never,
    }
    static COLOR_MODE: AtomicU8 = AtomicU8::new(ColorMode::Auto as u8);
    #[inline]
    const fn level_from_u8(x: u8) -> Level {
        match x {
            0 => Level::Trace,
            1 => Level::Debug,
            3 => Level::Warn,
            4 => Level::Error,
            5 => Level::Fatal,
            _ => Level::Info, // sane default
        }
    }
    #[inline]
    const fn color_mode_from_u8(x: u8) -> ColorMode {
        match x {
            1 => ColorMode::Always,
            2 => ColorMode::Never,
            _ => ColorMode::Auto,
        }
    }
    #[inline]
    fn color_mode() -> ColorMode {
        color_mode_from_u8(COLOR_MODE.load(Ordering::Relaxed))
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ParseColorModeError;

    impl core::str::FromStr for ColorMode {
        type Err = ParseColorModeError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.eq_ignore_ascii_case("always") {
                Ok(Self::Always)
            } else if s.eq_ignore_ascii_case("never") {
                Ok(Self::Never)
            } else if s.is_empty() || s.eq_ignore_ascii_case("auto") {
                Ok(Self::Auto)
            } else {
                Err(ParseColorModeError)
            }
        }
    }

    impl core::convert::TryFrom<&str> for ColorMode {
        type Error = ParseColorModeError;
        fn try_from(s: &str) -> Result<Self, Self::Error> {
            s.parse()
        }
    }

    // ===== Target sink =====
    /// Output target
    #[derive(Copy, Clone)]
    pub enum Target {
        /// stdout
        Stdout,
        /// stderr
        Stderr,
        /// custom
        Writer,
    }
    static TARGET: OnceLock<Target> = OnceLock::new();
    static WRITER: OnceLock<StdMutex<Box<dyn Write + Send>>> = OnceLock::new();
    /// Sets the output target once. Subsequent calls are ignored.
    /// Call this early (e.g., at program start) if you need `Stdout`.
    pub fn set_target(t: Target) {
        let _ = TARGET.set(t);
    }
    /// Sets the output target to a custom writer.
    pub fn set_writer(w: Box<dyn Write + Send>) {
        let _ = WRITER.set(StdMutex::new(w));
    }
    /// Sets the output target to a file.
    /// # Errors
    /// This function will return an error if the file cannot be opened for
    pub fn set_file(path: impl AsRef<Path>) -> io::Result<()> {
        let f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        set_writer(Box::new(f));
        set_target(Target::Writer);
        Ok(())
    }
    #[inline]
    fn target() -> Target {
        *TARGET.get_or_init(|| Target::Stderr)
    }

    static EMIT_LOCK: StdMutex<()> = StdMutex::new(());

    /// Returns `true` if the logger is enabled for the given level
    #[inline]
    #[must_use]
    pub const fn ct_enabled(l: Level) -> bool {
        (l as u8) >= (CT_MIN as u8)
    }
    #[inline]
    fn rt_enabled(l: Level) -> bool {
        (l as u8) >= RUNTIME_LEVEL.load(Ordering::Relaxed)
    }

    #[cfg(feature = "color")]
    mod color {
        pub const RST: &str = "\x1b[0m";
        pub const BOLD: &str = "\x1b[1m";
        pub const TRACE: &str = "\x1b[90m"; // bright black
        pub const DEBUG: &str = "\x1b[36m"; // cyan
        pub const INFO: &str = "\x1b[32m"; // green
        pub const WARN: &str = "\x1b[33m"; // yellow
        pub const ERROR: &str = "\x1b[31m"; // red
        pub const FATAL: &str = "\x1b[35m"; // magenta
    }
    /// Returns the color code for the given level
    #[cfg(feature = "color")]
    #[inline]
    const fn level_color(l: Level) -> &'static str {
        use color::{DEBUG, ERROR, FATAL, INFO, TRACE, WARN};
        match l {
            Level::Trace => TRACE,
            Level::Debug => DEBUG,
            Level::Info => INFO,
            Level::Warn => WARN,
            Level::Error => ERROR,
            Level::Fatal => FATAL,
        }
    }

    fn use_color() -> bool {
        #[cfg(not(feature = "color"))]
        {
            false
        }
        #[cfg(feature = "color")]
        {
            match color_mode() {
                ColorMode::Always => true,
                ColorMode::Never => false,
                ColorMode::Auto => match target() {
                    Target::Stdout => io::stdout().is_terminal(),
                    Target::Stderr => io::stderr().is_terminal(),
                    Target::Writer => false, // unknown sink => assume no TTY
                },
            }
        }
    }

    /// Returns the current logging level
    #[inline]
    pub fn level() -> Level {
        level_from_u8(RUNTIME_LEVEL.load(Ordering::Relaxed))
    }
    /// Sets the current logging level
    pub fn set_level(l: Level) {
        RUNTIME_LEVEL.store(l as u8, Ordering::Relaxed);
    }
    /// Show thread ids
    pub fn set_show_thread_id(on: bool) {
        SHOW_TID.store(on, Ordering::Relaxed);
    }
    /// Show timestamps
    pub fn set_show_time(on: bool) {
        SHOW_TIME.store(on, Ordering::Relaxed);
    }
    /// Show file and line
    pub fn set_show_file_line(on: bool) {
        SHOW_FILE_LINE.store(on, Ordering::Relaxed);
    }
    /// Show group
    pub fn set_show_group(on: bool) {
        SHOW_GROUP.store(on, Ordering::Relaxed);
    }
    /// Sets the color mode
    pub fn set_color_mode(mode: ColorMode) {
        COLOR_MODE.store(mode as u8, Ordering::Relaxed);
    }
    /// Initialize the logger from environment variables
    pub fn init_from_env() {
        if let Ok(s) = std::env::var("RUST_LOG_LEVEL") {
            let l = match s.to_lowercase().as_str() {
                "trace" => Level::Trace,
                "debug" => Level::Debug,
                "info" => Level::Info,
                "warn" => Level::Warn,
                "error" => Level::Error,
                "fatal" => Level::Fatal,
                _ => level(),
            };
            set_level(l);
        }
        if let Ok(s) = std::env::var("RUST_LOG_COLOR") {
            set_color_mode(s.parse().unwrap_or(ColorMode::Auto));
        }
        if let Ok(s) = std::env::var("RUST_LOG_SHOW_TID") {
            set_show_thread_id(s == "1" || s.eq_ignore_ascii_case("true"));
        }
        if let Ok(s) = std::env::var("RUST_LOG_SHOW_TIME") {
            set_show_time(s == "1" || s.eq_ignore_ascii_case("true"));
        }
    }

    /// Correct Gregorian Y-M-D from days since 1970-01-01 (no deps).
    #[inline]
    #[allow(dead_code)]
    const fn civil_from_days_utc(days_since_unix_epoch: i64) -> (i32, u32, u32) {
        // Howard Hinnant’s algorithm
        let z = days_since_unix_epoch + 719_468; // days since 0000-03-01
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097; // [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0,399]
        let yd = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
        let mp = (5 * yd + 2) / 153; // [0, 11]
        let d = yd - (153 * mp + 2) / 5 + 1; // [1, 31]
        let m = mp + 3 - 12 * (mp / 10); // [1, 12]
        let y = 100 * era + yoe + (m <= 2) as i64; // year
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        (y as i32, m as u32, d as u32)
    }
    #[inline]
    fn write_timestamp(mut w: impl Write) {
        #[cfg(all(feature = "timestamp", not(feature = "localtime")))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let secs = now.as_secs() as i64;
            let ms = now.subsec_millis();

            let days = secs.div_euclid(86_400);
            let sod = secs.rem_euclid(86_400);
            let h = (sod / 3_600) as i64;
            let m = (sod % 3_600 / 60) as i64;
            let s = (sod % 60) as i64;

            let (year, month, day) = civil_from_days_utc(days);
            let _ = write!(
                w,
                "{year:04}-{month:02}-{day:02} {h:02}:{m:02}:{s:02}.{ms:03}Z "
            );
        }
        #[cfg(all(feature = "timestamp", feature = "localtime"))]
        {
            // Local time via `time` crate if you enable the `localtime` feature
            let now = std::time::SystemTime::now();
            let now: time::OffsetDateTime = now.into();
            let now = now
                .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC));
            let _ = write!(
                w,
                "{} ",
                now.format(
                    &time::format_description::parse(
                        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
                    )
                    .unwrap()
                )
                .unwrap()
            );
        }
    }

    #[inline]
    fn write_tid(mut w: impl Write) {
        if SHOW_TID.load(Ordering::Relaxed) {
            #[cfg(feature = "thread-id")]
            let _ = write!(w, " [{:?}]", std::thread::current().id());
        }
    }

    #[inline]
    fn write_level(mut w: impl Write, l: Level, use_color: bool) {
        #[cfg(feature = "color")]
        if use_color {
            let _ = write!(
                w,
                "{}{:<5}{}",
                level_color(l),
                format!("{l:?}").to_uppercase(),
                color::RST
            );
            return;
        }
        let _ = write!(w, "{:<5}", format!("{l:?}").to_uppercase());
    }

    fn emit_raw_bytes(bytes: &[u8]) {
        let _g = EMIT_LOCK.lock().unwrap();
        match target() {
            Target::Stdout => {
                let _ = io::stdout().lock().write_all(bytes);
            }
            Target::Stderr => {
                let _ = io::stderr().lock().write_all(bytes);
            }
            Target::Writer => {
                if let Some(m) = WRITER.get() {
                    let mut w = m.lock().unwrap();
                    let _ = w.write_all(bytes);
                }
            }
        }
    }

    /// Emit a log message
    #[inline]
    pub fn emit(
        l: Level,
        group: Option<&'static str>,
        file: &'static str,
        line_no: u32,
        args: Arguments,
    ) {
        if !rt_enabled(l) {
            return;
        }
        let use_color = use_color();
        let mut buf = Vec::<u8>::new();

        if SHOW_TIME.load(Ordering::Relaxed) {
            write_timestamp(&mut buf);
        }
        write_level(&mut buf, l, use_color);
        write_tid(&mut buf);
        if SHOW_FILE_LINE.load(Ordering::Relaxed) {
            let _ = write!(&mut buf, " <{file}:{line_no}>");
        }
        if SHOW_GROUP.load(Ordering::Relaxed) {
            if let Some(g) = group {
                if use_color {
                    let _ = write!(
                        &mut buf,
                        " [{}{}{}{}]",
                        color::BOLD,
                        level_color(l),
                        g,
                        color::RST
                    );
                } else {
                    let _ = write!(&mut buf, " [{g}]");
                }
            }
        }
        let _ = buf.write_all(b" ");
        let _ = buf.write_fmt(args);
        let _ = buf.write_all(b"\n");
        emit_raw_bytes(&buf);
    }

    /// Emit a log message
    #[macro_export]
    macro_rules! __rustlog_log { ($lvl:expr, $grp:expr, $($t:tt)+) => {{ if $crate::ct_enabled($lvl) { $crate::emit($lvl, $grp, file!(), line!(), format_args!($($t)+)) } }} }
    /// trace
    #[macro_export]
    macro_rules! trace { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Trace, None, $($t)+) } }
    /// debug
    #[macro_export]
    macro_rules! debug { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Debug, None, $($t)+) } }
    /// info
    #[macro_export]
    macro_rules! info  { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Info,  None, $($t)+) } }
    /// warning
    #[macro_export]
    macro_rules! warn  { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Warn,  None, $($t)+) } }
    /// error
    #[macro_export]
    macro_rules! error { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Error, None, $($t)+) } }
    /// fatal
    #[macro_export]
    macro_rules! fatal { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Fatal, None, $($t)+) } }
    /// trace group
    #[macro_export]
    macro_rules! trace_group { ($grp:expr, $($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Trace, Some($grp), $($t)+) } }
    /// debug group
    #[macro_export]
    macro_rules! debug_group { ($grp:expr, $($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Debug, Some($grp), $($t)+) } }
    /// info group
    #[macro_export]
    macro_rules! info_group  { ($grp:expr, $($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Info,  Some($grp), $($t)+) } }
    /// warning group
    #[macro_export]
    macro_rules! warn_group  { ($grp:expr, $($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Warn,  Some($grp), $($t)+) } }
    /// error group
    #[macro_export]
    macro_rules! error_group { ($grp:expr, $($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Error, Some($grp), $($t)+) } }
    /// fatal group
    #[macro_export]
    macro_rules! fatal_group { ($grp:expr, $($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Fatal, Some($grp), $($t)+) } }
    /// Time a block
    #[macro_export]
    macro_rules! scope_time {
        ($label:expr, $body:block) => {{
            let _guard = $crate::TimerGuard::new_at($label, file!(), line!());
            $body
        }};
    }
    pub struct HumanDuration(std::time::Duration);
    impl core::fmt::Display for HumanDuration {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let d = self.0;
            let secs = d.as_secs();
            let nanos = d.subsec_nanos();
            if secs == 0 {
                if nanos < 1_000 {
                    write!(formatter, "{nanos} ns")
                } else if nanos < 1_000_000 {
                    write!(formatter, "{} us", nanos / 1_000)
                } else {
                    let ms = nanos / 1_000_000;
                    let us = (nanos / 1_000) % 1_000;
                    write!(formatter, "{ms}.{us:03} ms")
                }
            } else if secs < 60 {
                let ms = nanos / 1_000_000;
                write!(formatter, "{secs}.{ms:03} s")
            } else if secs < 3_600 {
                let m = secs / 60;
                let s = secs % 60;
                let ms = nanos / 1_000_000;
                write!(formatter, "{m}m{s:02}.{ms:03}s")
            } else if secs < 86_400 {
                let h = secs / 3_600;
                let m = (secs % 3_600) / 60;
                let s = secs % 60;
                let ms = nanos / 1_000_000;
                write!(formatter, "{h}h{m:02}m{s:02}.{ms:03}s")
            } else {
                let days = secs / 86_400;
                let rem = secs % 86_400;
                let h = rem / 3_600;
                let m = (rem % 3_600) / 60;
                let s = rem % 60;
                let ms = nanos / 1_000_000;
                write!(formatter, "{days}d {h:02}h{m:02}m{s:02}.{ms:03}s")
            }
        }
    }

    /// Timer guard
    pub struct TimerGuard {
        label: &'static str,
        start: Instant,
        file: &'static str,
        line: u32,
    }
    impl TimerGuard {
        /// Create a new timer guard
        #[inline]
        #[must_use]
        pub fn new_at(label: &'static str, file: &'static str, line: u32) -> Self {
            Self {
                label,
                start: Instant::now(),
                file,
                line,
            }
        }
    }
    impl Drop for TimerGuard {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed();
            emit(
                Level::Info,
                Some(self.label),
                self.file,
                self.line,
                format_args!("took {}", HumanDuration(elapsed)),
            );
        }
    }

    /// Emit a banner
    pub fn banner() {
        let name = env!("CARGO_PKG_NAME");
        let ver = env!("CARGO_PKG_VERSION");
        let mode = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let line = format!("{name} {ver} ({mode})\n");
        emit_raw_bytes(line.as_bytes());
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use core::time::Duration as StdDuration;

        #[test]
        fn human_duration_formats_all_ranges() {
            assert_eq!(
                format!("{}", HumanDuration(StdDuration::from_nanos(500))),
                "500 ns"
            );
            assert_eq!(
                format!("{}", HumanDuration(StdDuration::from_nanos(1_500))),
                "1 us"
            );
            assert_eq!(
                format!("{}", HumanDuration(StdDuration::from_nanos(1_234_000))),
                "1.234 ms"
            );
            assert_eq!(
                format!("{}", HumanDuration(StdDuration::from_millis(1_234))),
                "1.234 s"
            );
            assert_eq!(
                format!("{}", HumanDuration(StdDuration::from_secs(65))),
                "1m05.000s"
            );
            assert_eq!(
                format!(
                    "{}",
                    HumanDuration(StdDuration::from_secs(3 * 3600 + 7 * 60 + 5))
                ),
                "3h07m05.000s"
            );
            assert_eq!(
                format!("{}", HumanDuration(StdDuration::from_secs(2 * 86_400 + 5))),
                "2d 00h00m05.000s"
            );
        }
    }
}

#[cfg(not(feature = "std"))]
mod imp {
    //! Minimal `no_std` surface. All I/O is a no-op; the API matches the `std` path
    //! so `#![no_std]` crates can depend on this library without pulling `std`.

    use core::fmt::Arguments;
    use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

    // ===== Levels =====
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
    #[repr(u8)]
    pub enum Level {
        Trace = 0,
        Debug,
        Info,
        Warn,
        Error,
        Fatal,
    }

    // ===== Compile-time minimum (profile-based) =====
    #[cfg(debug_assertions)]
    const CT_MIN: Level = Level::Trace;
    #[cfg(not(debug_assertions))]
    const CT_MIN: Level = Level::Info;

    // ===== Runtime state (no_std: toggles exist but are cosmetic) =====
    static RUNTIME_LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);
    static SHOW_TID: AtomicBool = AtomicBool::new(false);
    static SHOW_TIME: AtomicBool = AtomicBool::new(false);
    static SHOW_FILE_LINE: AtomicBool = AtomicBool::new(false);

    // ===== Color mode (kept for API parity) =====
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    #[repr(u8)]
    pub enum ColorMode {
        Auto = 0,
        Always = 1,
        Never = 2,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ParseColorModeError;

    impl core::str::FromStr for ColorMode {
        type Err = ParseColorModeError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.eq_ignore_ascii_case("always") {
                Ok(ColorMode::Always)
            } else if s.eq_ignore_ascii_case("never") {
                Ok(ColorMode::Never)
            } else if s.is_empty() || s.eq_ignore_ascii_case("auto") {
                Ok(ColorMode::Auto)
            } else {
                Err(ParseColorModeError)
            }
        }
    }

    // ===== Helpers: safe u8 <-> enum mapping =====
    #[inline]
    const fn level_from_u8(x: u8) -> Level {
        match x {
            0 => Level::Trace,
            1 => Level::Debug,
            2 => Level::Info,
            3 => Level::Warn,
            4 => Level::Error,
            5 => Level::Fatal,
            _ => Level::Info,
        }
    }

    // ===== Public controls (no_std-safe) =====
    #[inline]
    pub const fn ct_enabled(l: Level) -> bool {
        (l as u8) >= (CT_MIN as u8)
    }

    #[inline]
    pub fn set_level(l: Level) {
        RUNTIME_LEVEL.store(l as u8, Ordering::Relaxed);
    }

    #[inline]
    pub fn level() -> Level {
        level_from_u8(RUNTIME_LEVEL.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn set_show_thread_id(on: bool) {
        let _ = on;
        SHOW_TID.store(on, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_show_time(on: bool) {
        let _ = on;
        SHOW_TIME.store(on, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_show_file_line(on: bool) {
        let _ = on;
        SHOW_FILE_LINE.store(on, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_color_mode(_: ColorMode) { /* no effect in no_std */
    }

    #[inline]
    pub fn init_from_env() { /* no env in no_std; keep API parity */
    }

    // ===== Emission =====
    #[inline]
    pub fn emit(_: Level, _: Option<&'static str>, _: &'static str, _: u32, _: Arguments) {
        // no_std: no I/O; define your own sink behind a feature if needed
    }

    // ===== Macros =====
    #[macro_export]
    macro_rules! __rustlog_log {
        ($lvl:expr, $grp:expr, $($t:tt)+) => {{
            if $crate::ct_enabled($lvl) {
                $crate::emit($lvl, $grp, file!(), line!(), format_args!($($t)+))
            }
        }}
    }

    #[macro_export]
    macro_rules! trace  { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Trace, None, $($t)+) } }
    #[macro_export]
    macro_rules! debug  { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Debug, None, $($t)+) } }
    #[macro_export]
    macro_rules! info   { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Info,  None, $($t)+) } }
    #[macro_export]
    macro_rules! warn   { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Warn,  None, $($t)+) } }
    #[macro_export]
    macro_rules! error  { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Error, None, $($t)+) } }
    #[macro_export]
    macro_rules! fatal  { ($($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Fatal, None, $($t)+) } }

    #[macro_export]
    macro_rules! info_group { ($grp:expr, $($t:tt)+) => { $crate::__rustlog_log!($crate::Level::Info, Some($grp), $($t)+) } }

    // In no_std we do not measure time; just run the block.
    #[macro_export]
    macro_rules! scope_time {
        ($label:expr, $body:block) => {{
            let _ = $label;
            $body
        }};
    }
}

// Re-exports for crate users
#[cfg(feature = "std")]
pub use imp::{
    banner, ct_enabled, emit, init_from_env, level, set_color_mode, set_file, set_level,
    set_show_file_line, set_show_group, set_show_thread_id, set_show_time, set_target, set_writer,
    ColorMode, Level, Target, TimerGuard,
};

#[cfg(not(feature = "std"))]
pub use imp::{
    ct_enabled, emit, init_from_env, level, set_color_mode, set_level, set_show_file_line,
    set_show_thread_id, set_show_time, ColorMode, Level,
};
