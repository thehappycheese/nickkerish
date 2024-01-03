use super::{
    HmacSha256,
    Header,
    JupyterMessageContent,
    DELIMITER
};
use crate::util::EmptyObjectOr;
use crate::util::TryFromJsonBytesString;
use crate::util::TryToJsonBytesString;

use anyhow::Context;
use anyhow::Result;
use bytes::Bytes;
use hmac::Mac;
use serde::Deserialize;
use serde::Serialize;
use zeromq::ZmqMessage;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata{}

/// A deserialized ZMQ Jupyter Message
#[derive(Debug, Default)]
pub struct Message {
    pub identities: Vec<Bytes>,
    pub signature: Bytes,
    pub header: EmptyObjectOr<Header>,
    pub parent_header: EmptyObjectOr<Header>,
    pub metadata: Metadata,
    pub content: EmptyObjectOr<JupyterMessageContent>,
    pub extra_buffers: Vec<Bytes>,
}

impl TryFrom<ZmqMessage> for Message {
    type Error = anyhow::Error;
    fn try_from(message: ZmqMessage) -> Result<Self> {
        // find the index of the delimiter
        let frames = message.into_vec();
        let delimiter_index = frames.iter().position(|frame| frame == &DELIMITER).unwrap();
        Ok(Message {
            identities    : frames[0..delimiter_index].into(),
            signature     : frames[delimiter_index + 1].clone(),
            header        : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 2]).context("Failed to decode header in TryFrom<ZmqMessage> for Message")?,
            parent_header : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 3]).context("Failed to decode parent_header in TryFrom<ZmqMessage> for Message")?,
            metadata      : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 4]).context("Failed to decode metadata in TryFrom<ZmqMessage> for Message")?,
            content       : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 5]).context("Failed to decode content in TryFrom<ZmqMessage> for Message")?,
            extra_buffers : frames[delimiter_index + 6..].into(),
        })
    }
}

impl Message {
    pub fn to_zmq_message(&self, key: String) -> Result<ZmqMessage> {
        // compute signature
        let mut frames: Vec<Bytes> = vec![];
        frames.extend(self.identities.clone());
        frames.push(DELIMITER.into());
        frames.push(self.compute_signature(key)?.into());
        frames.push(self.header.try_to_json_bytes()?);
        frames.push(self.parent_header.try_to_json_bytes()?);
        frames.push(serde_json::to_string(&self.metadata)?.into());
        frames.push(serde_json::to_string(&self.content)?.into());
        frames.extend(self.extra_buffers.clone());
        frames
            .try_into()
            .map_err(|e| anyhow::anyhow!(format!("{e}")))
    }

    fn compute_signature(&self, key: String) -> Result<String> {
        let mut signature = HmacSha256::new_from_slice(key.as_bytes())?;
        
        signature.update(self.header.try_to_json_string()?.as_bytes());
        signature.update(self.parent_header.try_to_json_string()?.as_bytes());
        signature.update(serde_json::to_string(&self.metadata)?.as_bytes());
        signature.update(self.content.try_to_json_string()?.as_bytes());

        let signature = hex::encode(signature.finalize().into_bytes());
        Ok(signature)
    }
}


#[cfg(test)]
mod tests {
    use zeromq::ZmqMessage;
    use crate::wire::JupyterKernelInfoReply;

    use super::*;
    #[test]
    fn test_default_message(){
        let result = Message::default();
        println!("{result:?}");
        let result = result.to_zmq_message("key".to_string()).unwrap();
        println!("{result:?}");
    }
    #[test]
    fn default_kernel_info_reply() {
        let content = JupyterKernelInfoReply::default();
        let message = Message {
            content: JupyterMessageContent::KernelInfoReply(content).into(),
            ..Default::default()
        };
        let message: ZmqMessage = message.to_zmq_message("key".to_owned()).unwrap();
        println!("Default kernel reply message: {:?}", message);
    }
}
