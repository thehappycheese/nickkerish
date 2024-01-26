#![feature(result_option_inspect)]

#[macro_use]
mod logging;

mod command_line_interface;
mod connection;
mod install;
mod server;
mod util;
mod wire;

mod execute;

use command_line_interface::CommandLineInterface;
use server::serve;

use anyhow::Result;
use clap::Parser;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    let _logging_worker_guard = logging::setup()?;
    println_debug!("Logging setup complete");
    match CommandLineInterface::parse() {
        CommandLineInterface::InstallKernelSpec => {
            println_debug!("Installing Nickkerish Kernel...");
            install::kernel_spec()
                .inspect_err(|err| println_debug!("Failed to install kernelspec {err}"))?;
            println_debug!("Kernel installed successfully");
        }
        CommandLineInterface::Run {
            mut connection_file,
        } => {
            println_debug!("Starting the Nickkerish Kernel...");
            let connection_information = serde_json::from_reader(&mut connection_file)
                .inspect_err(|err| println_debug!("Failed to read connection file: {err}"))?;
            serve(connection_information)
                .await
                .inspect_err(|err| println_debug!("Server Failed: {err}"))?;
        }
    }
    println_debug!("Exiting Main");
    Ok(())
}
