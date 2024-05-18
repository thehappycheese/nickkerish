use serde::{Serialize, Deserialize};

use super::ReplyStatus;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ErrorReply{
    pub status:ReplyStatus,
    #[serde(rename = "ename")]
    pub error_name:String,
    #[serde(rename = "evalue")]
    pub error_message:String,
    #[serde(rename = "traceback")]
    pub stack_trace:Vec<String>,
}