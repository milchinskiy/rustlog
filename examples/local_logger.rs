use rustlog::local::{info, info_group, scope_time};
use rustlog::local::Logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::builder()
        .set_level(rustlog::Level::Debug)
        .set_show_thread_id(false)
        .set_show_file_line(false)
        .build()?;

    info!(&logger, "Hello, local world!");
    rustlog::info!("Hello, global world!");

    info_group!(&logger, "group-name", "Hello, local world!");
    rustlog::info_group!("group-name", "Hello, global world!");

    scope_time!(&logger, "startup-local", {
        info!(&logger, "only visible at DEBUG+");
        std::thread::sleep(std::time::Duration::from_millis(10));
    });
    rustlog::scope_time!("startup-global", {
        rustlog::info!("only visible at DEBUG+");
        std::thread::sleep(std::time::Duration::from_millis(10));
    });

    Ok(())
}
