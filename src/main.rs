use clap::Parser;
use clio::Input;

mod connection;
mod install;
use zeromq::{SocketSend, SocketRecv};

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    println!("Starting Nickerish Kernel...");

    let args = Cli::parse();

    if let Cli::InstallKernelSpec = args    {
        install::kernel_spec()?;
        println!("Kernel installed successfully");
    } else if let Cli::Run { mut connection_file } = args {

        let connection_info: connection::Connection =
            serde_json::from_reader(&mut connection_file).unwrap();
        
            println!("Trying to connect using connection file:\n    {connection_info:?}");

        // Create a ZMQ context

        // Create and connect the shell socket
        let mut shell_socket:zeromq::RouterSocket     = connection_info.create_socket_shell    ().await?;
        // let iopub_socket:zeromq::PubSocket     = connection_info.create_socket_iopub    ().await?;
        // let stdin_socket:zeromq::RouterSocket     = connection_info.create_socket_stdin    ().await?;
        // let control_socket:zeromq::RouterSocket   = connection_info.create_socket_control  ().await?;
        let heartbeat_socket:zeromq::RepSocket = connection_info.create_socket_heartbeat().await?;
        
        
        println!("Successfully Created Sockets");

        let shell_result = shell_socket.recv().await?;
        println!("Received message from shell: {:?}", shell_result);

        // Block and wait for a message
        tokio::spawn(async move {
            handel_heartbeat(heartbeat_socket).await
        });
        
    }
    Ok(())
}

async fn handel_heartbeat(mut heartbeat_socket:zeromq::RepSocket) -> anyhow::Result<()>{
    loop{
        heartbeat_socket.recv().await?;
        println!("Received a heartbeat");
        heartbeat_socket.send(zeromq::ZmqMessage::from(b"ping".to_vec())).await?;
    }
}
