# rustlog

A small, dependency‑light logging crate with a pragmatic API, color (optional), groups, and a scope timer.

## Features at a glance

- **Macros:** `trace!`, `debug!`, `info!`, `warn!`, `error!`, `fatal!`
- **Extras:** `info_group!(group, ...)`, `scope_time!(label, { ... })`
- **Targets:** `Stdout`, `Stderr`, or a custom writer via `set_writer(...)` / `set_file(path)`
- **Runtime toggles:** show time, thread id, file\:line, group
- **Color (optional):** `Always` / `Never` / `Auto` (TTY detection for Stdout/Stderr)
- **Env config:** `RUST_LOG_LEVEL`, `RUST_LOG_COLOR`, `RUST_LOG_SHOW_TID`, `RUST_LOG_SHOW_TIME`
- **Compile‑time floor:** `debug` includes `trace`, `release` may strip `trace`/`debug`

> **MSRV:** Rust **1.70+** (uses `OnceLock` and `std::io::IsTerminal`).

---

## Install

```toml
[dependencies]
rustlog = "x.x"
```

### Feature flags

- `color` — ANSI colors; `Auto` uses TTY detection for Stdout/Stderr
- `timestamp` — prepend timestamp to each line
- `localtime` *(optional, only if you enable it)* — with `timestamp`, format local time instead of UTC
- `thread-id` — include thread id when enabled at runtime

> If you don’t enable `color`, output never contains ANSI escapes.

---

## Quick start

```rust
use rustlog::*;

fn main() {
    // Choose output early; first call wins (set‑once semantics).
    set_target(Target::Stderr); // default if unset
    // set_file("/var/log/app.log").unwrap(); // or write to a file

    // Configure runtime toggles
    set_show_time(true);        // requires `timestamp` feature
    set_show_thread_id(false);  // requires `thread-id` feature
    set_show_file_line(true);

    // Runtime level (compile‑time floor still applies)
    set_level(Level::Info);

    info!("hello {}", 42);
    warn!("heads up");
    info_group!("net", "retry #{}", 3);

    scope_time!("startup", {
        // work …
    }); // logs "took …" when the scope ends
}
```

Typical output (UTC timestamp shown when `timestamp` is enabled; colors elided):

```
2025-09-25 12:34:56.789Z INFO <main.rs:15> hello 42
2025-09-25 12:34:56.790Z WARN <main.rs:16> heads up
2025-09-25 12:34:56.791Z INFO <main.rs:19> [net] retry #3
2025-09-25 12:34:56.792Z INFO <main.rs:22> [startup] took 1.234 ms
```

---

## Targets

Targets are **set once** for the process (internally `OnceLock`). Set them at program start.

```rust
set_target(Target::Stdout);
set_target(Target::Stderr);            // default
set_file("app.log").unwrap();         // convenience: opens/creates + selects `Writer`

// Custom sink (useful in tests):
use std::io::Write;
struct Mem(Vec<u8>);
impl Write for Mem {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> { self.0.extend_from_slice(b); Ok(b.len()) }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}
set_writer(Box::new(Mem(Vec::new())));
set_target(Target::Writer);
```

> With `ColorMode::Auto`, `Writer` is treated as non‑TTY (no color). Force color with `ColorMode::Always` if you control the sink.

---

## Levels & filtering

- **Macros:** `trace!`, `debug!`, `info!`, `warn!`, `error!`, `fatal!`
- **Compile‑time floor:**
  - `debug` builds include `trace`/`debug` code paths.
  - `release` builds may compile out `trace`/`debug`; `info+` always remains.
- **Runtime filter:** `set_level(Level::Info)` etc.

A record is emitted if:

```
(level >= compile_time_min) && (level >= runtime_level)
```

---

## Groups & scope timer

```rust
info_group!("db", "query {}", "select 1");
scope_time!("init", { /* code */ }); // logs "took …" at drop
```

Duration formatting:

- `< 1_000 ns` → `NNN ns`
- `< 1_000_000 ns` → `NNN us`
- `< 1 s` → `M.us ms` (e.g. `1.234 ms`)
- `< 60 s` → `S.mmm s` (e.g. `1.234 s`)
- `< 3600 s` → `MmSS.mmm s` (e.g. `2m03.456s`)
- `< 24 h` → `HhMMmSS.mmm s` (e.g. `1h02m03.456s`)
- `≥ 24 h` → `Dd HHhMMmSS.mmm s`

---

## Colors (feature = `color`)

```rust
set_color_mode(ColorMode::Always); // force ANSI
set_color_mode(ColorMode::Never);  // disable
set_color_mode(ColorMode::Auto);   // Stdout/Stderr use TTY detect; Writer = no color
```

Env override (read by `init_from_env()`):

```bash
RUST_LOG_COLOR=always|never|auto
```

---

## Timestamps (feature = `timestamp`)

Enable at runtime:

```rust
set_show_time(true);
```

- **UTC** format (default): `YYYY-MM-DD HH:MM:SS.mmmZ`
- **Local time**: enable the `localtime` feature (if you turn it on in your build) to use the system local time.

> The UTC path uses a correct Gregorian conversion with no external deps.

---

## Thread id (feature = `thread-id`)

Enable at runtime:

```rust
set_show_thread_id(true);
```

---

## File\:line and group tag

```rust
set_show_file_line(true); // include `<file:line>`
// group tag is shown when you use info_group!(...) or scope_time!(label, ...)
```

---

## Application banner (app name & version)

Use the `banner!()` macro to print your app’s name and version as a single info‑level line.

```rust
use rustlog::*;

fn main() {
    set_target(Target::Stderr);
    set_level(Level::Info);

    banner!(); // -> "myapp v1.2.3"
}
```

### Customize name/version explicitly

If you don’t want to use Cargo metadata, pass strings directly:

```rust
banner!("myapp", "1.2.3");
```

`banner!()` is allocation‑free and safe to call early during startup.

## Environment variables

Call `init_from_env()` once at startup to read these:

| Variable             | Values                                        | Effect             |
| -------------------- | --------------------------------------------- | ------------------ |
| `RUST_LOG_LEVEL`     | `trace` `debug` `info` `warn` `error` `fatal` | Sets runtime level |
| `RUST_LOG_COLOR`     | `always` `never` `auto`                       | Sets color mode    |
| `RUST_LOG_SHOW_TID`  | `1` `true` *(case‑insensitive)*               | Show thread id     |
| `RUST_LOG_SHOW_TIME` | `1` `true` *(case‑insensitive)*               | Show timestamp     |

Example:

```bash
RUST_LOG_LEVEL=debug RUST_LOG_COLOR=auto RUST_LOG_SHOW_TIME=1 cargo run
```

---

## Testing tips

- To capture output in tests, install a memory writer and select `Target::Writer` **before** the first log in that test binary.
- Targets are set‑once. Place target selection at the top of `main()` or in a per‑test binary.
- Each log line is emitted with a single `write_all`, guarded by a mutex to avoid interleaving across threads.

---

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option.

```
SPDX-License-Identifier: MIT OR Apache-2.0
```

If you contribute, you agree to license your contributions under the same terms.
