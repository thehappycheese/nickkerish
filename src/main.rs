use clap::{Parser, Args, Subcommand, ValueEnum};
use clio::Input;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::net::IpAddr;
use chrono::Local;

mod connection;
mod install;
mod logging;

use logging::{log, setup_logging};

// use clap to create a command line argument --connection-file to receive the path
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
enum Cli {
    /// run the server
    #[command()]
    Run {
        /// Path to the connection file supplied by jupyter or vscode.
        /// This is a json file that contains the ip address, ports and other connection metadata.
        #[arg(long)]
        connection_file: Input,
    },
    /// create a new kernel.json and install it by running `jupyter kernelspec install --user [...]`
    #[command()]
    InstallKernelSpec
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_file = setup_logging()?;
    macro_rules! logg {
        (x:$expr) => {
            log(&mut log_file, $x)?
        };
    };
    logg!("Starting Nickerish Kernel");

    let mut args = Cli::parse();
    if let Cli::InstallKernelSpec = args    {
        install::kernel_spec();
        logg!("Kernel installed successfully")?;
    } else if let Cli::Run { mut connection_file } = args {

        let connection_info: connection::Connection =
            serde_json::from_reader(&mut connection_file).unwrap();
        
        logg!(format!("Trying to connect using conneciton file:\n    {connection_info:?}").as_str())?;

        // Create a ZMQ context
        let ctx = zmq::Context::new();

        // Create and connect the shell socket
        let shell_socket = ctx.socket(zmq::SocketType::REQ)?;
        let shell_endpoint = format!("tcp://{}:{}", connection_info.ip, connection_info.shell_port);
        shell_socket.connect(&shell_endpoint)?;

        // IOPub socket setup
        let mut iopub_socket = ctx.socket(zmq::SocketType::SUB)?;
        let iopub_endpoint = format!("tcp://{}:{}", connection_info.ip, connection_info.iopub_port);
        iopub_socket.connect(&iopub_endpoint)?;
        

        // Stdin socket setup
        let mut stdin_socket = ctx.socket(zmq::SocketType::REQ)?;
        let stdin_endpoint = format!("tcp://{}:{}", connection_info.ip, connection_info.stdin_port);
        stdin_socket.connect(&stdin_endpoint)?;

        // Control socket setup
        let mut control_socket = ctx.socket(zmq::SocketType::REQ)?;
        let control_endpoint = format!("tcp://{}:{}", connection_info.ip, connection_info.control_port);
        control_socket.connect(&control_endpoint)?;

        // Heartbeat (HB) socket setup
        let mut hb_socket = ctx.socket(zmq::SocketType::REQ)?;
        let hb_endpoint = format!("tcp://{}:{}", connection_info.ip, connection_info.hb_port);
        hb_socket.connect(&hb_endpoint)?;


        println!("Success");
        logg!("Successfully Started Sockets");

        // Block and wait for a message
        println!("Wait for something to happen on iopub");
        iopub_socket.set_subscribe(b"")?; // Subscribe to all messages
        let message = iopub_socket.recv_msg(0)?;
        println!("Received iopub message: {:?}", message);

        // Block and wait for a message
        println!("Wait for something to happen on iopub");
        iopub_socket.set_subscribe(b"")?; // Subscribe to all messages
        let message = iopub_socket.recv_msg(0)?;
        println!("Received iopub message: {:?}", message);
        
    }

    

    Ok(())
}
