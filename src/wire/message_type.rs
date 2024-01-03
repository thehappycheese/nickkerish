use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all="snake_case")]
pub enum JupyterMessageType {
    ExecuteRequest,
    ExecuteReply,
    KernelInfoRequest,
    KernelInfoReply,
    // IO Pub
    Status
}
