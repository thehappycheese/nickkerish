use std::{collections::HashMap, default};

use crate::{
    connection::ConnectionInformation,
    wire::{
        KERNEL_MESSAGING_VERSION,

        MessageBytes,
        MessageParsed,
        Header,
            MessageType,
        MessageContent,
            ExecutionState,

        KernelInfoReply,
        StatusPublication,
        IsCompleteReply,
        IsCompleteReplyStatus,
        ExecuteReply,
        ExecuteResultPublication, CommClose, CommOpen, ExecuteInputPublication, StreamPublication,
    },
    util::{iso_8601_Z_now, zmq_message_pretty_print, EmptyObjectOr},
};

use anyhow::Result;
use bytes::Bytes;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;
use zeromq::{SocketRecv, SocketSend};



pub async fn serve(connection_information: ConnectionInformation) -> Result<()> {
    println_debug!("Server Connecting...");
    
    // Define global constants
    let kernel_session_id: String = Uuid::new_v4().into();
    let kernel_username: String = "kernel".to_string(); // TODO: the spec isn't clear if the kernel replies should actually contain this field or not, or what the value should be when responding

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
        Default::default(),
        &connection_information.key,
        &kernel_username,
        ExecutionState::Starting,
    ).await?;

    publish_kernel_status(
        &mut iopub_socket,
        &kernel_session_id,
        Default::default(),
        &connection_information.key,
        &kernel_username,
        ExecutionState::Idle,
    ).await?;

    loop{
        let shell_result = shell_socket.recv().await?;
        let message_received: MessageBytes = match shell_result.clone().try_into(){
            Ok(message_received) => message_received,
            Err(err) => {
                println_debug!("RECV SHELL: {:}", zmq_message_pretty_print(shell_result));
                println_debug!("Unable to unpack received message: {err:?}");
                continue;
            }
        };
        let message_received = match message_received.decode(&connection_information.key){
            Ok(message_received)=>message_received,
            Err(err)=>{
                println_debug!("Unable to decode received message: {err:?}");
                continue;
            }
        };
        println_debug!("RECV SHELL:: {message_received}");
        
        publish_kernel_status(
            &mut iopub_socket,
            &kernel_session_id,
            message_received.header.clone(),
            &connection_information.key,
            &kernel_username,
            ExecutionState::Busy,
        ).await?;
        
        // TODO: Incoming messages should always have a header
        //       So should outgoing messages... when does a message not have a header?
        // TODO: this nesting sucks
        if let EmptyObjectOr::Object(message_header) = &message_received.header {

            match message_header.message_type {
                MessageType::KernelInfoRequest=>{
                    let response = message_received.reply(
                        Header {
                            message_id: Uuid::new_v4().into(),
                            message_type: MessageType::KernelInfoReply,
                            date: iso_8601_Z_now(),
                            session: kernel_session_id.clone().into(),
                            username: kernel_username.clone().into(),
                            version: KERNEL_MESSAGING_VERSION.into(),
                        }.into(),
                        MessageContent::from(KernelInfoReply::default()).into(),
                        Default::default(),
                        Default::default()
                    );
                    println_debug!("Sending KernelInfoReply {response:}");
                    //let response = response.to_zmq_message(&connection_information.key)?;
                    //println_debug!("Sending KernelInfoReply {response:?}");
                    shell_socket.send(response.encode()?.into()).await?;
                },
                MessageType::ExecuteRequest=>{
                    let mut code_to_execute = None;
                    if let EmptyObjectOr::Object(MessageContent::ExecuteRequest(execute_request)) = &message_received.content{
                        code_to_execute = Some(execute_request.code.clone());
                    }
                    let code_to_execute = match code_to_execute {
                        Some(x)=>x,
                        None=>{
                            // TODO: fix this, it may legitimately happen.
                            panic!("Tried to execute NONE... currently this is a panic and die situation")
                        }
                    };
                    println_debug!("Tried to execute {code_to_execute:?}");
                    let response = message_received.reply(
                        Header{
                            message_id: Uuid::new_v4().into(),
                            message_type: MessageType::ExecuteInput,
                            date: iso_8601_Z_now(),
                            session: kernel_session_id.clone().into(),
                            username: kernel_username.clone().into(),
                            version: KERNEL_MESSAGING_VERSION.into(),
                        },
                        MessageContent::ExecuteInputPublication(ExecuteInputPublication{
                            code: code_to_execute.clone(),
                            execution_count: 1,
                        }).into(),
                        Default::default(),
                        Default::default()
                        
                    );
                    println_debug!("Sending ExecuteInput {response:}");
                    shell_socket.send(response.encode()?.into()).await?;
                    publish_execution_result(
                        &mut iopub_socket,
                        &kernel_session_id,
                        message_received.header.clone(),
                        &connection_information.key,
                        &kernel_username,
                        &format!("You tried to execute `{code_to_execute:?}`, but Nickkerish is a dummy kernel, and does not do what you want!")
                    ).await?;
                    let response = message_received.reply(
                        Header {
                            message_id: Uuid::new_v4().into(),
                            message_type: MessageType::ExecuteReply,
                            date: iso_8601_Z_now(),
                            session: kernel_session_id.clone().into(),
                            username: kernel_username.clone().into(),
                            version: KERNEL_MESSAGING_VERSION.into(),
                        },
                        MessageContent::from(ExecuteReply {
                            status: crate::wire::ExecuteReplyStatus::Ok,
                            execution_count: 1,
                            payload: None,
                            user_expressions: None,
                        }).into(),
                        Default::default(),
                        Default::default()
                        
                    );
                    println_debug!("Sending ExecuteReply {response:}");
                    shell_socket.send(response.encode()?.into()).await?;
                    
                },
                MessageType::IsCompleteRequest=>{
                    let response = message_received.reply(
                        Header {
                            message_id: Uuid::new_v4().into(),
                            message_type: MessageType::IsCompleteRequest,
                            date: iso_8601_Z_now(),
                            session: kernel_session_id.clone().into(),
                            username: kernel_username.clone().into(),
                            version: KERNEL_MESSAGING_VERSION.into(),
                        },
                        MessageContent::from(IsCompleteReply {
                            status: IsCompleteReplyStatus::Complete,
                            indent: None,
                        }).into(),
                        Default::default(),
                        Default::default()
                    );
                    //println_debug!("Sending IsCompleteReply {response:?}");
                    println_debug!("Sending IsCompleteReply {response}");
                    shell_socket.send(response.encode()?.into()).await?;
                },
                MessageType::HistoryRequest=>{
                    println_debug!("HistoryRequest received... TODO: Respond");
                },
                MessageType::CommOpen=>{
                    // we don't do comms, shut it down instantly
                    if let EmptyObjectOr::Object(MessageContent::CommOpen(comm_open)) = &message_received.content {
                        let content = MessageContent::CommClose(CommClose{
                            comm_id:comm_open.comm_id.clone(),
                            data:Default::default(),
                        }).into();
                        let response = message_received.reply(
                            Header {
                                message_id: Uuid::new_v4().into(),
                                message_type: MessageType::CommClose,
                                username: kernel_username.clone().into(),
                                session: kernel_session_id.clone().into(),
                                date: iso_8601_Z_now(),
                                version: KERNEL_MESSAGING_VERSION.into(),
                            },
                            content,
                            Default::default(),
                            Default::default(),
                        );
                        println_debug!("Sending CommClose {response:?}");
                        shell_socket.send(response.encode()?.into()).await?;
                    }else{
                        println_debug!("CommMsg received... but could not unpack content");
                        panic!("CommMsg received... but could not unpack content")
                    }
                },
                MessageType::CommMsg=>{
                    println_debug!("CommMsg received... TODO: Respond");
                },
                MessageType::CommClose=>{
                    println_debug!("CommClose received... TODO: Respond");
                },
                // TODO: it is a bit dumb to have incoming and outgoing message types together maybe?
                //MessageType::IsCompleteReply=>unreachable!("This is an outgoing only message type"),
                MessageType::ExecuteInput=>unreachable!("This is an outgoing only message type"),
                MessageType::Stream=>unreachable!("This is an outgoing only message type"),
                MessageType::ExecuteResult=>unreachable!("This is an outgoing only message type"),
                MessageType::IsCompleteReply=>unreachable!("This is an outgoing only message type"),
                MessageType::KernelInfoReply=>unreachable!("This is an outgoing only message type"),
                MessageType::ExecuteReply =>unreachable!("This is an outgoing only message type"),
                MessageType::Status=>unreachable!("This is an outgoing only message type"),
            }
        }
        publish_kernel_status(
            &mut iopub_socket,
            &kernel_session_id,
            message_received.parent_header.clone(),
            &connection_information.key,
            &kernel_username,
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
        //debug!("Received heartbeat message {message:?}");
        heartbeat_socket.send(message).await?;
    }
}

async fn publish_kernel_status(
    iopub_socket: &mut zeromq::PubSocket,
    kernel_session_id: &str,
    parent_header: EmptyObjectOr<Header>,
    key: &str,
    username: &str,
    status: ExecutionState,
) -> Result<()> {
    let message = MessageParsed {
        key:key.into(),
        identities: Vec::new(),//vec![Bytes::from("kernel_status")], // TODO: is topic needed? excvr doesn't use one
        content: MessageContent::from(StatusPublication {
            execution_state: status,
        })
        .into(),
        header: Header {
            message_id: Uuid::new_v4().into(),
            message_type: MessageType::Status,
            date: iso_8601_Z_now(),
            session: kernel_session_id.into(),
            username: username.into(),
            version: KERNEL_MESSAGING_VERSION.into(),
        }
        .into(),
        parent_header:parent_header.clone(),
        metadata: Default::default(),
        extra_buffers:Default::default(),
    };
    println_debug!("PublishingKernel Status: {message}");
    iopub_socket.send(message.encode()?.into()).await?;
    Ok(())
}

async fn publish_execution_result(
    iopub_socket: &mut zeromq::PubSocket,
    kernel_session_id: &str,
    parent_header: EmptyObjectOr<Header>,
    key: &str,
    username: &str,
    execution_result:&str
)-> Result<()>{
    let message = MessageParsed{
        key:key.into(),
        identities: vec![Bytes::from("stream")], // topic
        content: MessageContent::from(StreamPublication {
            name: "stdout".into(),
            text: execution_result.into(),
        }).into(),
        header: Header {
            message_id: Uuid::new_v4().into(),
            message_type: MessageType::Stream,
            date: iso_8601_Z_now(),
            session: kernel_session_id.into(),
            username: username.into(),
            version: KERNEL_MESSAGING_VERSION.into(),
        }.into(),
        parent_header:parent_header.clone(),
        ..Default::default()
    };
    println_debug!("Publishing Stream: {message}");
    iopub_socket.send(message.encode()?.into()).await?;

    let message = MessageParsed {
        key:key.into(),
        identities: vec![Bytes::from("execute_result")], // topic
        content: MessageContent::from(ExecuteResultPublication {
            execution_count: 1,
            data: json!({"text/plain":execution_result.to_owned()}),
            metadata: Default::default(),
        })
        .into(),
        header: Header {
            message_id: Uuid::new_v4().into(),
            message_type: MessageType::ExecuteResult,
            date: iso_8601_Z_now(),
            session: kernel_session_id.into(),
            username: username.into(),
            version: KERNEL_MESSAGING_VERSION.into(),
        }.into(),
        parent_header,
        ..Default::default()
    };
    println_debug!("Publishing Execution Result: {message}");
    iopub_socket.send(message.encode()?.into()).await?;
    Ok(())
}
