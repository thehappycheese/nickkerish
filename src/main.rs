mod connection;
mod install;
mod wire;
mod server;
use clap::Parser;

mod command_line_interface;
use command_line_interface::CommandLineInterface;
use anyhow::Result;

use server::serve;

#[tokio::main]
async fn main() -> Result<()> {
    let _logging_worker_guard = match logging_setup(){
        Ok(guard)=>guard,
        Err(e)=>{
            debug!("Logging Setup Failed: {}", e);
            return Err(e);
        }
    };
    println!("Nickkerish Logging Setup Complete");
    debug!("Nickkerish Logging Setup Complete");
    match CommandLineInterface::parse() {
        CommandLineInterface::InstallKernelSpec => {
            println!("Installing Nickkerish Kernel...");
            debug!("Nickkerish Logging Setup Complete");
            match install::kernel_spec(){
                Ok(_)=>{},
                Err(e)=>{
                    debug!("Installation Failed: {}", e);
                    return Err(e)
                }
            };
            println!("Kernel installed successfully");
            debug!("Kernel installed successfully");
        },
        CommandLineInterface::Run {
            mut connection_file,
        } => {
            println!("Starting the Nickkerish Kernel...");
            debug!("Starting the Nickkerish Kernel...");
            let connection_information = match serde_json::from_reader(&mut connection_file) {
                Ok(connection_information) => connection_information,
                Err(e) => {
                    debug!("Failed to read connection file: {}", e);
                    return Err(e.into());
                }
            };
            match serve(connection_information).await {
                Ok(_)=>{
                    println!("Nickkerish Kernel Exited Successfully.");
                    debug!("Nickkerish Kernel Exited Successfully.");
                },
                Err(e)=>{
                    debug!("Server Failed: {}", e);
                    return Err(e)
                }
            
            };
            
        }
    }
    Ok(())
}


use tracing::{Level, debug};
use tracing_appender::non_blocking::WorkerGuard;

fn logging_setup() -> Result<WorkerGuard>{
    //let current_executable_path = std::env::current_exe()?;
 
    let file_appender = tracing_appender::rolling::never(r"E:\GitProjects\rusty-jupyter-kernel\JUNK\LOGS", "my_log.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_max_level(Level::DEBUG)
        .init();

    debug!("Logging Started");
    Ok(guard)
}
