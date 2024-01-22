use anyhow::{Context, Result};
use tracing::Level;

/// Setup logging for the application.
///
/// This logging feature is intended to be turned off for production builds.
/// TODO: Figure out how to do that.
///
/// Logs are located based on the location of running exe so that we catch them nicely even if it
/// was vscode or jupyter started the exe.
/// 
/// TODO: Use a better mechanism to set the output location of logs
///
pub fn setup() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let mut current_executable_path = std::env::current_exe()
        .context("Failed to retrieve location of exe in logging::setup()")?;
    current_executable_path.push("../../../LOGS/");

    let file_appender = tracing_appender::rolling::never(current_executable_path, "log.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_max_level(Level::DEBUG)
        .init();
    Ok(guard)
}

macro_rules! println_debug {
    ($($arg:tt)*) => {{
        //println!($($arg)*);
        debug!($($arg)*);
    }};
}