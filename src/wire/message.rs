use super::{
    HmacSha256,
    Header,
    MessageContent,
    DELIMITER
};
use crate::util::EmptyObjectOr;
use crate::util::TryFromJsonBytesString;
use crate::util::TryToJsonBytesString;
use crate::util::abbreviate_string;

use anyhow::Context;
use anyhow::Result;
use bytes::Bytes;
use hmac::Mac;
use zeromq::ZmqMessage;


/// A deserialized ZMQ Jupyter Message
#[derive(Debug, Default)]
pub struct Message {
    /// When this message is to be a response to a received message, then just copy the identities
    /// from the received message. For iopub messages this is just a single value
    /// and is the "topic" by convention is the message type.
    /// 
    /// > In most cases, the IOPub topics are irrelevant and completely ignored, because front-ends
    /// > just subscribe to all topics. The convention used in the IPython kernel is to use the
    /// > `msg_type` as the topic, and possibly extra information about the message, e.g.
    /// > kernel.{u-u-i-d}.execute_result or stream.stdout
    /// 
    /// See <https://jupyter-client.readthedocs.io/en/latest/messaging.html#the-wire-protocol>
    pub identities: Vec<Bytes>,

    /// The signature is the HMAC hex digest of the concatenation of:
    /// - A shared key (typically the key field of a connection file)
    /// - The serialized header dict
    /// - The serialized parent header dict
    /// - The serialized metadata dict
    /// - The serialized content dict
    pub signature: Bytes,
    pub header: EmptyObjectOr<Header>,
    pub parent_header: EmptyObjectOr<Header>,
    pub metadata: serde_json::Map<String, serde_json::Value>,
    pub content: EmptyObjectOr<MessageContent>,
    /// Raw data buffers, which can be used by message types that support binary data such as comms
    /// and extensions to the protocol.
    pub extra_buffers: Vec<Bytes>,
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Message]{{\n\tidentities: [{:}],\n\tsignature: {:}\n\theader: {:}\n\tparent_header: {:}\n\tmetadata: {:}\n\tcontent: {:}\n\textra_buffers: [{:}]}}",
            self.identities.len(),
            abbreviate_string(&format!("{:?}", &self.signature)),
            serde_json::to_string(&self.header).unwrap(),
            serde_json::to_string(&self.parent_header).unwrap(),
            serde_json::to_string(&self.metadata).unwrap(),
            serde_json::to_string(&self.content).unwrap(),
            self.extra_buffers.len()
        )
    }
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
            header        : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 2]).context("Failed to decode .header in TryFrom<ZmqMessage> for Message")?,
            parent_header : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 3]).context("Failed to decode .parent_header in TryFrom<ZmqMessage> for Message")?,
            metadata      : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 4]).context("Failed to decode .metadata in TryFrom<ZmqMessage> for Message")?,
            content       : TryFromJsonBytesString::try_from_json_bytes(&frames[delimiter_index + 5]).context(format!("Failed to decode .content in TryFrom<ZmqMessage> for Message;\n {:?}", &frames[delimiter_index + 5]))?,
            extra_buffers : frames[delimiter_index + 6..].into(),
        })
    }
}

impl Message {
    /// Could not use `TryInto` trait because key must be provided
    pub fn to_zmq_message(&self, key: &str) -> Result<ZmqMessage> {
        let mut frames: Vec<Bytes> = vec![];
        frames.extend(self.identities.clone());
        frames.push(DELIMITER.into());
        frames.push(self.compute_signature(key)?.into());
        frames.push(self.header.try_to_json_bytes()?);
        frames.push(self.parent_header.try_to_json_bytes()?);
        frames.push(serde_json::to_string(&self.metadata)?.into());
        frames.push(serde_json::to_string(&self.content)?.into());
        frames.extend(self.extra_buffers.clone());
        ZmqMessage::try_from(frames).map_err(|e| anyhow::anyhow!(e))
    }

    fn compute_signature(&self, key: &str) -> Result<String> {
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
    use crate::wire::KernelInfoReply;

    use super::*;
    #[test]
    fn test_default_message(){
        let result = Message::default();
        println!("{result:?}");
        let result = result.to_zmq_message("test_dummy_key").unwrap();
        println!("{result:?}");
    }
    #[test]
    fn default_kernel_info_reply() {
        let content = KernelInfoReply::default();
        let message = Message {
            content: MessageContent::KernelInfoReply(content).into(),
            ..Default::default()
        };
        let message: ZmqMessage = message.to_zmq_message("test_dummy_key").unwrap();
        println!("Default kernel reply message: {:?}", message);
    }
    #[test]
    fn default_display_message() {
        let content = KernelInfoReply::default();
        let message = Message {
            content: MessageContent::KernelInfoReply(content).into(),
            ..Default::default()
        };
        println!("{message}");
    }
}
