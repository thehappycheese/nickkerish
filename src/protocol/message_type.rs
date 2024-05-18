use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all="snake_case")]
pub enum MessageType {
    // Shell
    ExecuteRequest,
    ExecuteReply,
    KernelInfoRequest,
    KernelInfoReply,
    IsCompleteRequest,
    IsCompleteReply,
    HistoryRequest,

    CommOpen,
    CommClose,
    CommMsg,
    // IO Pub
    Stream,
    ExecuteResult,
    ExecuteInput,
    Status,
}