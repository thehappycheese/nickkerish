
/// Message spec 4.1 (IPython 2.0) added a messaging system for developers to add their own objects
/// with Frontend and Kernel-side components, and allow them to communicate with each other. To do
/// this, IPython adds a notion of a Comm, which exists on both sides, and can communicate in either
/// direction.
///
/// These messages are fully symmetrical - both the Kernel and the Frontend can send each message,
/// and no messages expect a reply. The Kernel listens for these messages on the Shell channel, and
/// the Frontend listens for them on the IOPub channel.
/// 
/// Since comm messages can execute arbitrary user code, handlers should set the parent header and
/// publish status busy / idle, just like an execute request.

use serde::{Serialize, Deserialize};


/// Every Comm has an ID and a target name. The code handling the message on the receiving side is
/// responsible for maintaining a mapping of target_name keys to constructors. After a comm_open
/// message has been sent, there should be a corresponding Comm instance on both sides. The data
/// key is always a dict and can be any extra JSON information used in initialization of the comm.
/// 
/// If the target_name key is not found on the receiving side, then it should immediately reply with
/// a [comm_close](CommClose) message to avoid an inconsistent state.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CommOpen {
    pub comm_id: String,
    pub target_name: String,
    pub data: serde_json::Value
}


/// Since comms live on both sides, when a comm is destroyed the other side must be notified.
/// This is done with a comm_close message.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CommClose {
    pub comm_id: String,
    pub data: serde_json::Map<String, serde_json::Value>,
}


/// Comm messages are one-way communications to update comm state, used for synchronizing widget
/// state, or simply requesting actions of a comm’s counterpart.
/// 
/// Essentially, each comm pair defines their own message specification implemented inside the data
/// dict.
/// 
/// There are no expected replies (of course, one side can send another comm_msg in reply).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CommMsg {
    pub comm_id: String,
    pub data: serde_json::Map<String, serde_json::Value>,
}
