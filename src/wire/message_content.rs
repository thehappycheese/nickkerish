use super::JupyterKernelInfoReply;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JupyterMessageContent {
    KernelInfoReply(JupyterKernelInfoReply),
    PublishKernelStatus, // TODO:
}

impl From<JupyterKernelInfoReply> for JupyterMessageContent {
    fn from(reply: JupyterKernelInfoReply) -> Self {
        Self::KernelInfoReply(reply)
    }
}