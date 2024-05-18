use serde::{Serialize, Deserialize};
use super::message_type::MessageType;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Header{
    #[serde(rename="msg_id")]
    pub message_id:String,
    #[serde(rename="msg_type")]
    pub message_type:MessageType,
    pub username:String,
    pub session:String,
    pub date:String,
    pub version:String
}