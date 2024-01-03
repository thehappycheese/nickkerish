use crate::{
    connection::ConnectionInformation,
    wire::{
        Message,
        JupyterMessageType,
        JupyterMessageContent,
        JupyterKernelInfoReply
    }
};
use anyhow::{Result, Context};
use zeromq::{SocketRecv, SocketSend};
use tracing::debug;
pub async fn serve(connection_information: ConnectionInformation) -> Result<()> {
    
    println!("Server Connecting...");
    debug!("Server Connecting...");
    // Create a ZMQ context

    // Create and connect the shell socket

    let mut shell_socket: zeromq::RouterSocket = connection_information.create_socket_shell    ().await?;
    // 
    let iopub_socket    : zeromq::PubSocket    = connection_information.create_socket_iopub    ().await?;
    // For kernel to request stdin from frontend. Wont be used
    let _stdin_socket    : zeromq::RouterSocket = connection_information.create_socket_stdin    ().await?; 
    // for shutdown restart and debug requests from client
    let control_socket  : zeromq::RouterSocket = connection_information.create_socket_control  ().await?;
    let heartbeat_socket: zeromq::RepSocket    = connection_information.create_socket_heartbeat().await?;

    println!("Successfully Created Sockets, waiting for message on Shell Socket");
    debug!("Successfully Created Sockets, waiting for message on Shell Socket");

    let shell_result = shell_socket.recv().await?;
    println!("Shell: {shell_result:?}");
    debug!("Shell: {shell_result:?}");
    let message_received: Message = shell_result.try_into().context("Deser JupyterMessage")?;

    
    println!("Shell: {message_received:?}");
    debug!("Shell: {message_received:?}");
    let response = Message {
        identities: message_received.identities.clone(),
        signature: "".into(), // TODO: When Message is serialized, this is not used but recomputed
        header: message_received.header.clone().map(|value| value.with_id_type_date(
            value.message_id.clone(),
            JupyterMessageType::KernelInfoReply,
            value.date.clone(), // TODO: get the current date
        )),
        parent_header: message_received.header.clone(),
        content: JupyterMessageContent::KernelInfoReply(
            JupyterKernelInfoReply::default().into()
        ).into(),
        ..Default::default()
    };
    println!("Sending KernelInfoReply {response:?}");
    debug!("Sending KernelInfoReply {response:?}");
    let response = response.to_zmq_message(connection_information.key)?;
    println!("Sending KernelInfoReply {response:?}");
    debug!("Sending KernelInfoReply {response:?}");
    shell_socket.send(response).await?;
    
    println!("Starting Heartbeat");
    let heartbeat_join_handel = tokio::spawn(async move {
        match handel_heartbeat(heartbeat_socket).await {
            Ok(_)=>(),
            Err(e)=>{
                println!("Heartbeat Error: {:?}", e);
                debug!("Heartbeat Error: {:?}", e);
                ()
            }
        }
    });
    println!("Waiting for threads");
    tokio::try_join!(heartbeat_join_handel)?;
    println!("Server Existing Without Error.");
    Ok(())
}

async fn handel_heartbeat(mut heartbeat_socket: zeromq::RepSocket) -> Result<()> {
    loop {
        let message = heartbeat_socket.recv().await?;
        println!("Received a heartbeat");
        debug!("Received a heartbeat: {message:?}");
        heartbeat_socket
            .send(zeromq::ZmqMessage::from(b"ping".to_vec()))
            .await?;
    }
}
