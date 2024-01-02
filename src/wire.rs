use std::io::Cursor;

use hmac::{Hmac, Mac};
use zeromq::ZmqMessage;
use serde::{Deserialize, Serialize};
use bytes::Bytes;
use anyhow::Result;

pub type HmacSha256 = Hmac<sha2::Sha256>;


const DELIMITER: &[u8] = b"<IDS|MSG>";

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all="snake_case")]
pub enum JupyterMessageType {
    ExecuteRequest,
    ExecuteReply,
    KernelInfoRequest,
    KernelInfoReply,

    // IO Pub
    StatusRequest,
    StatusReply,

    #[default]
    Null
}


#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all="snake_case")]
enum KernelExecutionState {
    Busy,
    Idle,
    /// Just once at startup
    #[default]
    Starting
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JupyterKernelInfoLink{
    text:String,
    url:String
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JupyterKernelInfoLanguageInfo{
    /// Name of the programming language that the kernel implements.
    /// Kernel included in IPython returns 'python'.
    name: String,

    /// Language version number.
    /// It is Python version number (e.g., '2.7.3') for the kernel
    /// included in IPython.
    version: String,

    /// mimetype for script files in this language
    mimetype: String,

    /// Extension including the dot, e.g. '.py'
    file_extension: String,

    /// pygments lexer, for highlighting
    /// Only needed if it differs from the 'name' field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pygments_lexer: Option<String>,

    /// Codemirror mode, for highlighting in the notebook.
    /// Only needed if it differs from the 'name' field.
    /// TODO: apparently this should also accept a dict?
    // skip if none
    #[serde(skip_serializing_if = "Option::is_none")]
    codemirror_mode: Option<String>,

    /// nbconvert exporter, if notebooks written with this kernel should
    /// be exported with something other than the general 'script'
    /// exporter.
    #[serde(skip_serializing_if = "Option::is_none")]
    nbconvert_exporter: Option<String>,
}

impl Default for JupyterKernelInfoLanguageInfo{
    fn default() -> Self {
        Self {
            name: "nickkerish".to_owned(),
            version: "0.1.0".to_owned(),
            mimetype: "text/plain".to_owned(),
            file_extension: ".nk".to_owned(),
            pygments_lexer: Default::default(),
            codemirror_mode: Default::default(),
            nbconvert_exporter: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize,PartialEq)]
#[serde(rename_all="lowercase")]
enum JupyterReplyStatus{
    Ok,
    Error
}

#[derive(Debug, Serialize, Deserialize,PartialEq)]
pub struct JupyterKernelInfoReply {
    /// 'ok' if the request succeeded or 'error',
    /// with error information as in all other replies.
    status:JupyterReplyStatus,
    /// Version of messaging protocol.
    /// The first integer indicates major version.  It is incremented when
    /// there is any backward incompatible change.
    /// The second integer indicates minor version.  It is incremented when
    /// there is any backward compatible change.
    protocol_version:String,
    /// The kernel implementation name
    /// (e.g. 'ipython' for the IPython kernel)
    implementation:String,
    /// Implementation version number.
    /// The version number of the kernel's implementation
    /// (e.g. IPython.__version__ for the IPython kernel)
    implementation_version:String,

    /// Information about the language of code for the kernel
    language_info:JupyterKernelInfoLanguageInfo,

    /// A boolean flag which tells if the kernel supports debugging in the notebook.
    /// Default is False
    banner:String,

    /// A boolean flag which tells if the kernel supports debugging in the notebook.
    /// Default is False
    debugger:bool,

    /// Optional: A list of dictionaries, each with keys 'text' and 'url'.
    /// These will be displayed in the help menu in the notebook UI.
    help_links:Vec<JupyterKernelInfoLink>,
}

impl Default for JupyterKernelInfoReply{
    fn default() -> Self {
        Self {
            status: JupyterReplyStatus::Ok,
            protocol_version: "5.4.0".to_owned(),
            implementation: "nickkerish_kernel".to_owned(),
            implementation_version: "0.1.0".to_owned(),
            language_info: Default::default(),
            banner: Default::default(),
            debugger: Default::default(),
            help_links: vec![
                JupyterKernelInfoLink{
                    text:"Nickkerish Repo".to_owned(),
                    url:"https://github.com/thehappycheese/nickkerish".to_owned()
                }
            ]
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct JupyterHeader {
    #[serde(rename="msg_id")]
    pub message_id:String,
    #[serde(rename="msg_type")]
    pub message_type:JupyterMessageType,
    pub username:String,
    pub session:String,
    pub date:String,
    pub version:String
}

impl JupyterHeader{
    pub fn with_id_type_date(&self, message_id:String, message_type:JupyterMessageType, date:String) -> Self {
        Self{
            message_id,
            message_type,
            username: self.username.clone(),
            session: self.session.clone(),
            date,
            version: self.version.clone()
        }
    }
}

impl Into<Bytes> for JupyterHeader{
    fn into(self) -> Bytes {
        Bytes::from(serde_json::to_string(&self).unwrap())
    }
}
impl TryFrom<Bytes> for JupyterHeader{
    type Error = anyhow::Error;
    fn try_from(value: Bytes) -> Result<Self> {
        Ok(serde_json::from_slice(&value)?)
    }
}


#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EmptyObject{}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JupyterMessageContent{
    /// parses "{}""
    EmptyObject{},
    /// defers parsing to JupyterKernelInfoReply
    KernelInfoReply(JupyterKernelInfoReply)
}
impl Default for JupyterMessageContent{
    fn default() -> Self {
        Self::EmptyObject{}
    }
}

#[derive(Debug, Default)]
pub struct JupyterMessage{
    pub identities    : Vec<Bytes>,
    pub signature     : Bytes, // hex string
    pub header        : JupyterHeader, // JSON Dict
    pub parent_header : Bytes, // JSON Dict
    pub metadata      : Bytes, // JSON Dict // TODO: default should be {}
    pub content       : JupyterMessageContent, // JSON Dict
    pub extra_buffers : Vec<Bytes>
}

impl TryFrom<ZmqMessage> for JupyterMessage{
    type Error = anyhow::Error;
    fn try_from(message:ZmqMessage) -> Result<Self> {
        // find the index of the delimiter
        let frames = message.into_vec();
        let delimiter_index = frames.iter().position(|frame| frame == &DELIMITER).unwrap();
        let content:JupyterMessageContent = match serde_json::from_reader(
            Cursor::new(&frames[delimiter_index + 5])
        ) {
            Ok(item)=>item,
            Err(e)=>{
                println!("Error parsing content: {e}");

                return Err(e.into())
            }
        };
        Ok(JupyterMessage{
            identities    : frames[0..delimiter_index].into(),
            signature     : frames[delimiter_index + 1].clone(),
            header        : frames[delimiter_index + 2].clone().try_into()?,
            parent_header : frames[delimiter_index + 3].clone(),
            metadata      : frames[delimiter_index + 4].clone(),
            content       ,
            extra_buffers : frames[delimiter_index + 6..].into(),
        })
    }

}

impl JupyterMessage{
    pub fn to_zmq_message(&self, key:String) -> Result<ZmqMessage>{
        // compute signature
        let header_bytes:Bytes = self.header.clone().into();
        let content = serde_json::to_string(&self.content).unwrap();
        let mut signature = HmacSha256::new_from_slice(
            key.as_bytes()
        )?;

        signature.update(&header_bytes);
        signature.update(&self.parent_header);
        signature.update(&self.metadata);
        signature.update(content.as_bytes());
        let signature = hex::encode(signature.finalize().into_bytes());

        let mut frames:Vec<Bytes> = vec![];
        frames.extend(self.identities.clone());
        frames.push(DELIMITER.into());
        frames.push(signature.into());
        frames.push(header_bytes);
        frames.push(self.parent_header.clone());
        frames.push(self.metadata.clone());
        frames.push(Bytes::from(serde_json::to_string(&self.content).unwrap()));
        frames.extend(self.extra_buffers.clone());
        frames.try_into().map_err(|e|anyhow::anyhow!(format!("{e}")))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_empty_object(){
        let content = b"{}";
        let content_parsed:EmptyObject = serde_json::from_reader(
            Cursor::new(content)
        ).unwrap();
        assert_eq!(content_parsed,EmptyObject{});
    }

    #[test]
    fn test_parse_empty_empty_object(){
        let content = b"{}";
        let content_parsed:JupyterMessageContent = serde_json::from_reader(
            Cursor::new(content)
        ).unwrap();
        assert_eq!(content_parsed,JupyterMessageContent::EmptyObject{});
    }

    #[test]
    fn default_kernel_info_reply(){
        let content = JupyterKernelInfoReply::default();
        let message = JupyterMessage{
            content:JupyterMessageContent::KernelInfoReply(content),
            ..Default::default()
        };
        let message:ZmqMessage = message.to_zmq_message("key".to_owned()).unwrap();
        println!("Default kernel reply message: {:?}", message);
    }
}