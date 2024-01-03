use super::{JupyterKernelInfoReply, PublishKernelStatus};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JupyterMessageContent {
    KernelInfoReply(JupyterKernelInfoReply),
    PublishKernelStatus(PublishKernelStatus),
}

impl From<JupyterKernelInfoReply> for JupyterMessageContent {
    fn from(reply: JupyterKernelInfoReply) -> Self {
        Self::KernelInfoReply(reply)
    }
}
impl From<PublishKernelStatus> for JupyterMessageContent {
    fn from(status: PublishKernelStatus) -> Self {
        Self::PublishKernelStatus(status)
    }
}