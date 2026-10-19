// Suppress the console window in release builds (this is a GUI app).
// Kept visible in debug builds so log/println! output is easy to see.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod platform;
mod player;
mod ui;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("Starting Fluent Player...");

    ui::run()?;

    Ok(())
}
