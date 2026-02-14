use core::fmt::Arguments;
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;

// Pull from crate root
use crate::EMIT_LOCK;
#[cfg(feature = "color")]
use crate::{color, level_color};
use crate::{ct_enabled, write_level, write_timestamp, ColorMode, HumanDuration, Level, Target};

/// Local logger
pub struct Logger {
    level: AtomicU8,
    show_tid: AtomicBool,
    show_time: AtomicBool,
    show_group: AtomicBool,
    show_file_line: AtomicBool,
    color_mode: AtomicU8,
    sink: StdMutex<Sink>,
}

struct Sink {
    target: Target,
    writer: Option<Arc<StdMutex<Box<dyn Write + Send>>>>,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            level: AtomicU8::new(Level::Info as u8),
            show_tid: AtomicBool::new(cfg!(feature = "thread-id")),
            show_time: AtomicBool::new(cfg!(feature = "timestamp")),
            show_group: AtomicBool::new(true),
            show_file_line: AtomicBool::new(cfg!(feature = "file-line")),
            color_mode: AtomicU8::new(ColorMode::Auto as u8),
            sink: StdMutex::new(Sink {
                target: Target::Stderr,
                writer: None,
            }),
        }
    }
}

impl Logger {
    #[inline]
    #[must_use]
    /// Create a new `LoggerBuilder`
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::default()
    }

    // configuration
    #[inline]
    /// Set the log level
    pub fn set_level(&self, l: Level) {
        self.level.store(l as u8, Ordering::Relaxed);
    }
    #[inline]
    /// Set whether to show thread ids
    pub fn set_show_thread_id(&self, on: bool) {
        self.show_tid.store(on, Ordering::Relaxed);
    }
    #[inline]
    /// Set whether to show timestamps
    pub fn set_show_time(&self, on: bool) {
        self.show_time.store(on, Ordering::Relaxed);
    }
    #[inline]
    /// Set whether to show group
    pub fn set_show_group(&self, on: bool) {
        self.show_group.store(on, Ordering::Relaxed);
    }
    #[inline]
    /// Set whether to show file and line
    pub fn set_show_file_line(&self, on: bool) {
        self.show_file_line.store(on, Ordering::Relaxed);
    }
    #[inline]
    /// Set the color mode
    pub fn set_color_mode(&self, m: ColorMode) {
        self.color_mode.store(m as u8, Ordering::Relaxed);
    }

    #[inline]
    /// Set the target
    /// # Panics
    /// This function will panic if locking the sink fails
    pub fn set_target(&self, t: Target) {
        self.sink.lock().unwrap().target = t;
    }
    /// Set the writer
    /// # Panics
    /// This function will panic if locking the sink fails
    pub fn set_writer(&self, w: Box<dyn Write + Send>) {
        let arc = Arc::new(StdMutex::new(w));
        let mut s = self.sink.lock().unwrap();
        s.writer = Some(arc);
        s.target = Target::Writer;
    }
    /// Set the output target to a file.
    /// # Errors
    /// This function will return an error if the file cannot be opened for writing.
    pub fn set_file(&self, path: impl AsRef<std::path::Path>) -> io::Result<()> {
        let f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        self.set_writer(Box::new(f));
        Ok(())
    }

    #[inline]
    fn enabled(&self, l: Level) -> bool {
        (l as u8) >= self.level.load(Ordering::Relaxed)
    }

    /// Emit a log message
    /// # Panics
    /// This function will panic if locking the sink fails
    pub fn emit_to(
        &self,
        l: Level,
        group: Option<&'static str>,
        file: &'static str,
        line_no: u32,
        args: Arguments,
    ) {
        if !self.enabled(l) || !ct_enabled(l) {
            return;
        }

        let (target, writer) = {
            let s = self.sink.lock().unwrap();
            (s.target, s.writer.clone())
        };

        let mut buf = Vec::<u8>::new();
        let use_color = self.use_color_for_target(target);

        if self.show_time.load(Ordering::Relaxed) {
            write_timestamp(&mut buf);
        }
        write_level(&mut buf, l, use_color);

        if self.show_tid.load(Ordering::Relaxed) {
            #[cfg(feature = "thread-id")]
            let _ = write!(&mut buf, " [{:?}]", std::thread::current().id());
        }
        if self.show_file_line.load(Ordering::Relaxed) {
            let _ = write!(&mut buf, " <{file}:{line_no}>");
        }

        if self.show_group.load(Ordering::Relaxed) {
            if let Some(g) = group {
                #[cfg(feature = "color")]
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
                #[cfg(not(feature = "color"))]
                {
                    let _ = write!(&mut buf, " [{g}]");
                }
            }
        }

        let _ = buf.write_all(b" ");
        let _ = buf.write_fmt(args);
        let _ = buf.write_all(b"\n");

        let _g = EMIT_LOCK.lock().unwrap();
        match target {
            Target::Stdout => {
                let _ = io::stdout().lock().write_all(&buf);
            }
            Target::Stderr => {
                let _ = io::stderr().lock().write_all(&buf);
            }
            Target::Writer => {
                if let Some(w) = writer {
                    let _ = w.lock().unwrap().write_all(&buf);
                }
            }
        }
    }

    #[inline]
    fn use_color_for_target(&self, target: Target) -> bool {
        #[cfg(not(feature = "color"))]
        {
            return false;
        }
        #[cfg(feature = "color")]
        match ColorMode::from(self.color_mode.load(Ordering::Relaxed)) {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => match target {
                Target::Stdout => io::stdout().is_terminal(),
                Target::Stderr => io::stderr().is_terminal(),
                Target::Writer => false,
            },
        }
    }
}

/// Timer guard
pub struct TimerGuard<'a> {
    logger: &'a Logger,
    label: &'static str,
    start: Instant,
    file: &'static str,
    line: u32,
}
impl<'a> TimerGuard<'a> {
    /// Create a new timer guard
    #[inline]
    #[must_use]
    pub fn new_at(logger: &'a Logger, label: &'static str, file: &'static str, line: u32) -> Self {
        Self {
            logger,
            label,
            start: Instant::now(),
            file,
            line,
        }
    }
}
impl Drop for TimerGuard<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.logger.emit_to(
            Level::Info,
            Some(self.label),
            self.file,
            self.line,
            format_args!("took {}", HumanDuration(elapsed)),
        );
    }
}
#[macro_export]
/// Macro for timing a scope
macro_rules! __rustlog_local_scope_time {
    ($lg:expr, $label:expr) => {
        let _rustlog_scope_time_guard =
            $crate::local::TimerGuard::new_at($lg, $label, file!(), line!());
    };
    ($lg:expr, $label:expr, $body:block) => {{
        let _rustlog_scope_time_guard =
            $crate::local::TimerGuard::new_at($lg, $label, file!(), line!());
        $body
    }};
}

// Helper conversions if you keep enums repr(u8)
impl From<u8> for ColorMode {
    fn from(x: u8) -> Self {
        match x {
            1 => Self::Always,
            2 => Self::Never,
            _ => Self::Auto,
        }
    }
}

/// Builder for `Logger`
pub struct LoggerBuilder {
    level: Level,
    show_tid: Option<bool>,
    show_time: Option<bool>,
    show_group: Option<bool>,
    show_file_line: Option<bool>,
    color_mode: Option<ColorMode>,
    target: Target,
    writer: Option<Arc<StdMutex<Box<dyn Write + Send>>>>,
    file_path: Option<std::path::PathBuf>,
}
impl Default for LoggerBuilder {
    fn default() -> Self {
        Self {
            level: Level::Info,
            show_tid: None,
            show_time: None,
            show_group: None,
            show_file_line: None,
            color_mode: None,
            target: Target::Stderr,
            writer: None,
            file_path: None,
        }
    }
}

impl LoggerBuilder {
    #[inline]
    #[must_use]
    /// Set the log level
    pub const fn set_level(mut self, l: Level) -> Self {
        self.level = l;
        self
    }
    #[inline]
    #[must_use]
    /// Show the thread id
    pub const fn set_show_thread_id(mut self, on: bool) -> Self {
        self.show_tid = Some(on);
        self
    }
    #[inline]
    #[must_use]
    /// Show the timestamp
    pub const fn set_show_time(mut self, on: bool) -> Self {
        self.show_time = Some(on);
        self
    }
    #[inline]
    #[must_use]
    /// Show the log group
    pub const fn set_show_group(mut self, on: bool) -> Self {
        self.show_group = Some(on);
        self
    }
    #[inline]
    #[must_use]
    /// Show the file and line number
    pub const fn set_show_file_line(mut self, on: bool) -> Self {
        self.show_file_line = Some(on);
        self
    }
    #[inline]
    #[must_use]
    /// Set the color mode
    pub const fn set_color_mode(mut self, m: ColorMode) -> Self {
        self.color_mode = Some(m);
        self
    }
    #[inline]
    #[must_use]
    /// Set the output target to stdout
    pub const fn stdout(mut self) -> Self {
        self.target = Target::Stdout;
        self
    }
    #[inline]
    #[must_use]
    /// Set the output target to stderr
    pub const fn stderr(mut self) -> Self {
        self.target = Target::Stderr;
        self
    }
    #[inline]
    #[must_use]
    /// Set the output target to a custom writer
    pub fn set_writer(mut self, w: Box<dyn Write + Send>) -> Self {
        self.target = Target::Writer;
        self.writer = Some(Arc::new(StdMutex::new(w)));
        self
    }
    #[inline]
    #[must_use]
    /// Set the output target to a file
    pub fn file(mut self, p: impl AsRef<std::path::Path>) -> Self {
        self.target = Target::Writer;
        self.file_path = Some(p.as_ref().to_owned());
        self
    }

    /// Build the logger
    /// # Errors
    /// This function will return an error if the file cannot be opened for writing
    pub fn build(self) -> io::Result<Logger> {
        let writer = match (self.target, self.file_path) {
            (Target::Writer, Some(p)) => {
                let f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)?;
                Some(Arc::new(
                    StdMutex::new(Box::new(f) as Box<dyn Write + Send>),
                ))
            }
            _ => self.writer,
        };
        let lg = Logger {
            sink: StdMutex::new(Sink {
                target: self.target,
                writer,
            }),
            ..Logger::default()
        };
        lg.set_level(self.level);
        if let Some(x) = self.show_tid {
            lg.set_show_thread_id(x);
        }
        if let Some(x) = self.show_time {
            lg.set_show_time(x);
        }
        if let Some(x) = self.show_group {
            lg.set_show_group(x);
        }
        if let Some(x) = self.show_file_line {
            lg.set_show_file_line(x);
        }
        if let Some(x) = self.color_mode {
            lg.set_color_mode(x);
        }
        Ok(lg)
    }

    /// Build the logger and leak it
    /// # Errors
    /// This function will return an error if the file cannot be opened for writing
    pub fn build_static(self) -> io::Result<&'static Logger> {
        Ok(Box::leak(Box::new(self.build()?)))
    }
}

// ===== Macros (require a logger argument) ====================================
// We purposely keep these inside the module and re-export them below. In most
// editors, `use rustlog::local::info; info!(lg, "...")` is ergonomic and avoids
// name collision with the root `rustlog::info!`.

#[macro_export]
/// Emit a log message
macro_rules! __rustlog_local_log {
    ($lg:expr, $lvl:expr, $grp:expr, $($t:tt)+) => {{
        let __lg = $lg; // evaluate once
        if $crate::ct_enabled($lvl) { __lg.emit_to($lvl, $grp, file!(), line!(), format_args!($($t)+)); }
    }}
}

#[macro_export]
/// Emit a trace log message
macro_rules! __rustlog_local_trace { ($lg:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Trace, None, $($t)+) } }
#[macro_export]
/// Emit a debug log message
macro_rules! __rustlog_local_debug { ($lg:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Debug, None, $($t)+) } }
#[macro_export]
/// Emit an info log message
macro_rules! __rustlog_local_info  { ($lg:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Info,  None, $($t)+) } }
#[macro_export]
/// Emit a warning log message
macro_rules! __rustlog_local_warn  { ($lg:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Warn,  None, $($t)+) } }
#[macro_export]
/// Emit an error log message
macro_rules! __rustlog_local_error { ($lg:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Error, None, $($t)+) } }
#[macro_export]
/// Emit a fatal log message
macro_rules! __rustlog_local_fatal { ($lg:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Fatal, None, $($t)+) } }

#[macro_export]
/// Emit a trace group
macro_rules! __rustlog_local_trace_group { ($lg:expr, $grp:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Trace, Some($grp), $($t)+) } }
#[macro_export]
/// Emit a debug group
macro_rules! __rustlog_local_debug_group { ($lg:expr, $grp:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Debug, Some($grp), $($t)+) } }
#[macro_export]
/// Emit an info group
macro_rules! __rustlog_local_info_group  { ($lg:expr, $grp:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Info,  Some($grp), $($t)+) } }
#[macro_export]
/// Emit a warning group
macro_rules! __rustlog_local_warn_group  { ($lg:expr, $grp:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Warn,  Some($grp), $($t)+) } }
#[macro_export]
/// Emit an error group
macro_rules! __rustlog_local_error_group { ($lg:expr, $grp:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Error, Some($grp), $($t)+) } }
#[macro_export]
/// Emit a fatal group
macro_rules! __rustlog_local_fatal_group { ($lg:expr, $grp:expr, $($t:tt)+) => { $crate::__rustlog_local_log!($lg, $crate::Level::Fatal, Some($grp), $($t)+) } }

// Re-export ergonomic names under `rustlog::local`.
// Import style: `use rustlog::local::info; info!(logger, "...");`
// (Note: macro re-export keeps them callable after `use`; absolute path calling
// as `rustlog::local::info!` may depend on toolchain; the import form is recommended.)
pub use crate::__rustlog_local_debug as debug;
pub use crate::__rustlog_local_error as error;
pub use crate::__rustlog_local_fatal as fatal;
pub use crate::__rustlog_local_info as info;
pub use crate::__rustlog_local_trace as trace;
pub use crate::__rustlog_local_warn as warn;

pub use crate::__rustlog_local_debug_group as debug_group;
pub use crate::__rustlog_local_error_group as error_group;
pub use crate::__rustlog_local_fatal_group as fatal_group;
pub use crate::__rustlog_local_info_group as info_group;
pub use crate::__rustlog_local_trace_group as trace_group;
pub use crate::__rustlog_local_warn_group as warn_group;

pub use crate::__rustlog_local_scope_time as scope_time;
