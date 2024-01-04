use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all="snake_case")]
pub enum MessageType {
    ExecuteRequest,
    ExecuteReply,
    KernelInfoRequest,
    KernelInfoReply,
    IsCompleteRequest,
    HistoryRequest,
    IsCompleteReply,
    ExecuteResult,
    // InterruptReply,

    // IO Pub
    Status
}
