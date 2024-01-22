use super::{
    HmacSha256,
    Header,
    MessageContent,
    DELIMITER
};
use crate::util::EmptyObjectOr;
use crate::util::TryFromJsonBytesString;
use crate::util::TryToJsonBytesString;

use anyhow::Result;
use bytes::Bytes;
use hmac::Mac;
use zeromq::ZmqMessage;

/// Compute the signature for the message
/// 
/// TODO: The 
fn compute_signature(
    key           : &str, 
    header        : &Bytes, 
    parent_header : &Bytes, 
    metadata      : &Bytes, 
    content       : &Bytes, 
    extra_buffers : &Vec<Bytes>
) -> Result<Bytes> {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
    mac.update(header);
    mac.update(parent_header);
    mac.update(metadata);
    mac.update(content);
    for buffer in extra_buffers {
        mac.update(buffer);
    }
    let mac:Vec<u8> = mac.finalize().into_bytes().to_vec();
    Ok(mac.into())
}

#[derive(Debug)]
pub struct MessageBytes {
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
    identities: Vec<Bytes>,
    /// The signature must be the HMAC hex digest of the concatenation of:
    /// - A shared key (typically the key field of a connection file)
    /// - The serialized header dict
    /// - The serialized parent header dict
    /// - The serialized metadata dict
    /// - The serialized content dict
    /// 
    /// If authentication is disabled, then signature is expected to be an empty string.
    /// (See [ConnectionInformation](crate::connection::ConnectionInformation::key))
    signature: Bytes,

    /// the header for this message
    header: Bytes,
    /// the header copied from the message that caused this message (or an empty dict)
    parent_header: Bytes,
    metadata: Bytes,
    content: Bytes,
    extra_buffers: Vec<Bytes>,
}

impl MessageBytes{
    fn validate_signature(&self, key: &str) -> Result<()> {
        let signature = compute_signature(
            &key,
            &self.header,
            &self.parent_header,
            &self.metadata,
            &self.content,
            &self.extra_buffers,
        )?;
        let signature = hex::encode(signature);
        if signature == self.signature {
            Ok(())
        } else {
            Err(anyhow::anyhow!(format!("Signature validation failed {:?} != {:?}",signature, self.signature)))
        }
    }
    pub fn decode(self, key:&str) -> Result<MessageParsed> {
        self.validate_signature(key)?;
        Ok(MessageParsed{
            key           : key.to_owned(),
            identities    : self.identities,
            header        : TryFromJsonBytesString::try_from_json_bytes(&self.header)?,
            parent_header : TryFromJsonBytesString::try_from_json_bytes(&self.parent_header)?,
            metadata      : TryFromJsonBytesString::try_from_json_bytes(&self.metadata)?,
            content       : TryFromJsonBytesString::try_from_json_bytes(&self.content)?,
            extra_buffers : self.extra_buffers,
        })
    }
}

impl std::fmt::Display for MessageBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            //"[Message]{{\n\tidentities: [{:}],\n\tsignature: {:}\n\theader: {:}\n\tparent_header: {:}\n\tmetadata: {:}\n\tcontent: {:}\n\textra_buffers: [{:}]}}",
            "[MessageBytes]{{\n\tidentities: [{:?}],\n\tsignature: {:?},\n\theader: {:?}\n\tparent_header: {:?}\n\tmetadata: {:?}\n\tcontent: {:?}\n\textra_buffers: [{:?}]}}",
            self.identities.len(),
            self.header,
            self.signature,
            self.parent_header,
            self.metadata,
            self.content,
            self.extra_buffers.len()
        )
    }
}

impl From<ZmqMessage> for MessageBytes {
    fn from(message: ZmqMessage) -> Self {
        let message = message.into_vec();
        let delimiter_index = message.iter().position(|frame| frame == &DELIMITER).unwrap();
        MessageBytes {
            identities    : message[0..delimiter_index].into(),
            signature     : message[delimiter_index + 1].clone(),
            header        : message[delimiter_index + 2].clone(),
            parent_header : message[delimiter_index + 3].clone(),
            metadata      : message[delimiter_index + 4].clone(),
            content       : message[delimiter_index + 5].clone(),
            extra_buffers : message[delimiter_index + 6..].into(),
        }
    }
}

impl Into<ZmqMessage> for MessageBytes {
    fn into(self) -> ZmqMessage {
        let mut frames = Vec::new();
        frames.extend(self.identities);
        frames.push(DELIMITER.into());
        frames.push(self.signature);
        frames.push(self.header);
        frames.push(self.parent_header);
        frames.push(self.metadata);
        frames.push(self.content);
        frames.extend(self.extra_buffers);
        // NOTE: Empty Message Error is not possible since `frames.len()>0`
        ZmqMessage::try_from(frames).unwrap()
    }
}

/// A deserialized ZMQ Jupyter Message
#[derive(Debug, Default)]
pub struct MessageParsed {
    
    /// The key which will/was used to to sign the message
    pub key: String,

    /// Identities are part of the ZMQ protocol and are used for routing.
    /// We don't know why multiple identities might be needed? The whole delimiter business is very annoying.
    /// 
    pub identities: Vec<Bytes>,
    
    /// the header for this message
    pub header: EmptyObjectOr<Header>,

    /// A copy of the header from the message that 'caused' this message
    pub parent_header: EmptyObjectOr<Header>,

    /// Any valid JSON inside an object, or an empty object {}
    pub metadata: serde_json::Map<String, serde_json::Value>,

    /// Any valid JSON message content inside an object, or an empty object {}
    pub content: EmptyObjectOr<MessageContent>,

    /// Raw data buffers, which can be used by message types that support binary data such as comms
    /// and extensions to the protocol.
    pub extra_buffers: Vec<Bytes>,
}

impl std::fmt::Display for MessageParsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            //"[Message]{{\n\tidentities: [{:}],\n\tsignature: {:}\n\theader: {:}\n\tparent_header: {:}\n\tmetadata: {:}\n\tcontent: {:}\n\textra_buffers: [{:}]}}",
            "[Message]{{\n\tidentities: [{:}],\n\theader: {:}\n\tparent_header: {:}\n\tmetadata: {:}\n\tcontent: {:}\n\textra_buffers: [{:}]}}",
            self.identities.len(),
            serde_json::to_string(&self.header).unwrap(),
            serde_json::to_string(&self.parent_header).unwrap(),
            serde_json::to_string(&self.metadata).unwrap(),
            serde_json::to_string(&self.content).unwrap(),
            self.extra_buffers.len()
        )
    }
}

impl MessageParsed {
    pub fn new(
        key: String,
        identities: Vec<Bytes>,
        header: EmptyObjectOr<Header>,
        parent_header: EmptyObjectOr<Header>,
        metadata: serde_json::Map<String, serde_json::Value>,
        content: EmptyObjectOr<MessageContent>,
        extra_buffers: Vec<Bytes>,
    ) -> Self {
        MessageParsed {
            key           : key,
            identities    : identities,
            header        : header,
            parent_header : parent_header,
            metadata      : metadata,
            content       : content,
            extra_buffers : extra_buffers,
        }
    }

    pub fn reply(
        &self,
        header: Header,
        content: EmptyObjectOr<MessageContent>,
        metadata: serde_json::Map<String, serde_json::Value>,
        extra_buffers: Vec<Bytes>,
    ) -> MessageParsed {
        MessageParsed{
            key: self.key.clone(),
            identities: self.identities.clone(),
            header: EmptyObjectOr::Object(header),
            parent_header: self.header.clone(),
            metadata: metadata,
            content: content,
            extra_buffers: extra_buffers,
        }
    }

    pub fn encode(self) -> Result<MessageBytes> {
        let header        = (&self.header       ).try_to_json_bytes()?;
        let parent_header = (&self.parent_header).try_to_json_bytes()?;
        let metadata      = (&self.metadata     ).try_to_json_bytes()?;
        let content       = (&self.content      ).try_to_json_bytes()?;
        let signature = Bytes::from(hex::encode(compute_signature(
            &self.key,
            &header,
            &parent_header,
            &metadata,
            &content,
            &self.extra_buffers
        )?));
        Ok(MessageBytes{
            identities    : self.identities.clone(),
            signature     ,
            header        ,
            parent_header ,
            metadata      ,
            content       ,
            extra_buffers : self.extra_buffers,
        })
    }
    
}