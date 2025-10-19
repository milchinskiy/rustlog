use rustlog::local::info;
use rustlog::local::Logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::builder()
        .level(rustlog::Level::Debug)
        .show_thread_id(false)
        .show_file_line(false)
        .build()?;

    info!(logger, "Hello, local world!");
    rustlog::info!("Hello, global world!");

    Ok(())
}
