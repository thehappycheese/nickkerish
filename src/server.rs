use std::collections::HashMap;

use crate::{
    connection::ConnectionInformation,
    wire::{
        Header, MessageContent, MessageType, ExecutionState, Message,
        KernelInfoReply, StatusPublication, KERNEL_MESSAGING_VERSION, IsCompleteReply, IsCompleteReplyStatus, ExecuteReply, ExecuteResultPublication,
    },
};

use anyhow::{Context, Result};
use bytes::Bytes;
use chrono::Utc;
use tracing::debug;
use uuid::Uuid;
use zeromq::{SocketRecv, SocketSend};

pub async fn serve(connection_information: ConnectionInformation) -> Result<()> {
    println_debug!("Server Connecting...");
    // Create a ZMQ context

    // Create and connect the shell socket
    let kernel_session_id: String = Uuid::new_v4().into(); // TODO: generate guid

    let mut shell_socket: zeromq::RouterSocket =
        connection_information.create_socket_shell().await?;
    //
    let mut iopub_socket: zeromq::PubSocket = connection_information.create_socket_iopub().await?;
    // For kernel to request stdin from frontend. Wont be used
    let _stdin_socket: zeromq::RouterSocket = connection_information.create_socket_stdin().await?;
    // for shutdown restart and debug requests from client
    let _control_socket: zeromq::RouterSocket =
        connection_information.create_socket_control().await?;
    let heartbeat_socket: zeromq::RepSocket =
        connection_information.create_socket_heartbeat().await?;

    println_debug!("Successfully Created Sockets");

    println_debug!("Starting Heartbeat");
    let heartbeat_join_handel = tokio::spawn(async move {
        handel_heartbeat(heartbeat_socket)
            .await
            .inspect_err(|err| println_debug!("Heartbeat Error: {:?}", err)).unwrap()
    });

    publish_kernel_status(
        &mut iopub_socket,
        &kernel_session_id,
        &connection_information.key,
        ExecutionState::Starting,
    ).await?;

    loop{
        let shell_result = shell_socket.recv().await?;
        println_debug!("Shell: {shell_result:?}");
        let message_received: Message = shell_result.try_into().context("Deser JupyterMessage")?;
        println_debug!("Shell: {message_received:?}");
        
        publish_kernel_status(
            &mut iopub_socket,
            &kernel_session_id,
            &connection_information.key,
            ExecutionState::Busy,
        ).await?;
        
        // TODO: Incoming messages should always have a header
        //       So should outgoing messages... when does a message not have a header?
        let message_header = message_received.header.unwrap();

        match message_header.message_type {
            MessageType::KernelInfoRequest=>{
                let response = Message {
                    identities: message_received.identities.clone(),
                    header: Header {
                        message_id: Uuid::new_v4().into(),
                        message_type: MessageType::KernelInfoReply,
                        date: Utc::now().to_rfc3339(),
                        session: kernel_session_id.clone().into(),
                        username: "Nickkerish Kernel".into(),
                        version: KERNEL_MESSAGING_VERSION.into(),
                    }.into(),
                    parent_header: message_header.clone().into(),
                    content: MessageContent::from(KernelInfoReply::default()).into(),
                    ..Default::default()
                };
                //println_debug!("Sending KernelInfoReply {response:?}");
                let response = response.to_zmq_message(&connection_information.key)?;
                println_debug!("Sending KernelInfoReply {response:?}");
                shell_socket.send(response).await?;
            },
            MessageType::ExecuteRequest=>{
                let mut code_to_execute = None;
                if let MessageContent::ExecuteRequest(execute_request) = message_received.content.unwrap(){
                    code_to_execute = Some(execute_request.code.clone());
                }
                println_debug!("Tried to execute {code_to_execute:?}");
                let response = Message {
                    identities: message_received.identities.clone(),
                    header: Header {
                        message_id: Uuid::new_v4().into(),
                        message_type: MessageType::ExecuteReply,
                        date: Utc::now().to_rfc3339(),
                        session: kernel_session_id.clone().into(),
                        username: "Nickkerish Kernel".into(),
                        version: KERNEL_MESSAGING_VERSION.into(),
                    }.into(),
                    parent_header: message_header.clone().into(),
                    content: MessageContent::from(ExecuteReply {
                        status: crate::wire::ExecuteReplyStatus::Ok,
                        execution_count: 1,
                        payload: None,
                        user_expressions: None,
                    }).into(),
                    ..Default::default()
                };
                
                let response = response.to_zmq_message(&connection_information.key)?;
                println_debug!("Sending ExecuteReply {response:?}");
                shell_socket.send(response).await?;
                publish_execution_result(
                    &mut iopub_socket,
                    &kernel_session_id,
                    &connection_information.key,
                    &format!("You tried to execute `{code_to_execute:?}`, but Nickkerish is a dummy kernel, and does not do what you want!")
                ).await?;
            },
            MessageType::IsCompleteRequest=>{
                let response = Message {
                    identities: message_received.identities.clone(),
                    header: Header {
                        message_id: Uuid::new_v4().into(),
                        message_type: MessageType::IsCompleteRequest,
                        date: Utc::now().to_rfc3339(),
                        session: kernel_session_id.clone().into(),
                        username: "Nickkerish Kernel".into(),
                        version: KERNEL_MESSAGING_VERSION.into(),
                    }.into(),
                    parent_header: message_header.clone().into(),
                    content: MessageContent::from(IsCompleteReply {
                        status: IsCompleteReplyStatus::Complete,
                        indent: None,
                    }).into(),
                    ..Default::default()
                };
                //println_debug!("Sending IsCompleteReply {response:?}");
                let response = response.to_zmq_message(&connection_information.key)?;
                println_debug!("Sending IsCompleteReply {response:?}");
                shell_socket.send(response).await?;
            },
            MessageType::HistoryRequest=>{
                println_debug!("HistoryRequest received... TODO: Respond");
            },
            

            // TODO: it is a bit dumb to have incoming and outgoing message types together maybe?
            //MessageType::IsCompleteReply=>unreachable!("This is an outgoing only message type"),
            MessageType::ExecuteResult=>unreachable!("This is an outgoing only message type"),
            MessageType::IsCompleteReply=>unreachable!("This is an outgoing only message type"),
            MessageType::KernelInfoReply=>unreachable!("This is an outgoing only message type"),
            MessageType::ExecuteReply =>unreachable!("This is an outgoing only message type"),
            MessageType::Status=>unreachable!("This is an outgoing only message type"),
        }
        publish_kernel_status(
            &mut iopub_socket,
            &kernel_session_id,
            &connection_information.key,
            ExecutionState::Idle,
        ).await?;
    }
    println_debug!("Waiting for threads");
    tokio::try_join!(heartbeat_join_handel)?;
    println_debug!("Server Existing Without Error.");
    Ok(())
}

async fn handel_heartbeat(mut heartbeat_socket: zeromq::RepSocket) -> Result<()> {
    loop {
        let message = heartbeat_socket.recv().await?;
        debug!("recieved heartbeat message {message:?}");
        heartbeat_socket
            .send(zeromq::ZmqMessage::from(b"ping".to_vec()))
            .await?;
    }
}

async fn publish_kernel_status(
    iopub_socket: &mut zeromq::PubSocket,
    kernel_session_id: &str,
    key: &str,
    status: ExecutionState,
) -> Result<()> {
    let message = Message {
        identities: vec![Bytes::from("kernel_status")], // topic
        content: MessageContent::from(StatusPublication {
            execution_state: status,
        })
        .into(),
        header: Header {
            message_id: Uuid::new_v4().into(),
            message_type: MessageType::Status,
            date: Utc::now().to_rfc3339(),
            session: kernel_session_id.into(),
            username: "Nickkerish Kernel".into(),
            version: KERNEL_MESSAGING_VERSION.into(),
        }
        .into(),
        ..Default::default()
    };
    println_debug!("PublishingKernel Status: {message:?}");
    iopub_socket.send(message.to_zmq_message(key)?).await?;
    Ok(())
}

async fn publish_execution_result(
    iopub_socket: &mut zeromq::PubSocket,
    kernel_session_id: &str,
    key: &str,
    execution_result:&str
)-> Result<()>{
    let message = Message {
        identities: vec![Bytes::from("execute_result")], // topic
        content: MessageContent::from(ExecuteResultPublication {
            execution_count: 1,
            data: HashMap::from([("text/plain".into(), execution_result.into())]),
            metadata: HashMap::new(),
        })
        .into(),
        header: Header {
            message_id: Uuid::new_v4().into(),
            message_type: MessageType::ExecuteResult,
            date: Utc::now().to_rfc3339(),
            session: kernel_session_id.into(),
            username: "Nickkerish Kernel".into(),
            version: KERNEL_MESSAGING_VERSION.into(),
        }
        .into(),
        ..Default::default()
    };
    println_debug!("Publishing Execution Result: {message:?}");
    iopub_socket.send(message.to_zmq_message(key)?).await?;
    Ok(())
}