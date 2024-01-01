use std::io::Cursor;

use zeromq::ZmqMessage;
use serde::{Deserialize, Serialize};
use bytes::Bytes;
use anyhow::Result;


///```text
/// ZmqMessage {
///     frames: [
///         b"7\xddt`C\x11KR\xae$!\xd3\x07\xe5\xd9\xbf",
///         b"<IDS|MSG>",
///         b"7e72471b9215b42f5c39ce33cf44801cb88ebb90b87dcc4515bb35da79cef80f",
///         b"{\"msg_id\: \"74264fb5-099e-4e26-973b-45880be96742_25128_42\", \"msg_type\": \"kernel_info_request\", \"uername\": \"username\", \"session\": \"74264fb5-099e-4e26-973b-45880be96742\", \"date\": \"202401-01T15:31:37.839684Z\", \"version\": \"5.3\"}",
///         b"{}",
///         b"{}",
///         b"{}"
///     ]
/// }
/// ```
/// 

const DELIMITER: &[u8] = b"<IDS|MSG>";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum JupyterMessageType{
    ExecuteRequest,
    ExecuteReply,
    KernelInfoRequest,
    KernelInfoReply,
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

    /// Pygments lexer, for highlighting
    /// Only needed if it differs from the 'name' field.
    pygments_lexer: Option<String>,

    /// Codemirror mode, for highlighting in the notebook.
    /// Only needed if it differs from the 'name' field.
    /// TODO: apparently this should also accept a dict?
    codemirror_mode: Option<String>,

    /// Nbconvert exporter, if notebooks written with this kernel should
    /// be exported with something other than the general 'script'
    /// exporter.
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
            implementation: "nickerish_kernel".to_owned(),
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

#[derive(Debug, Deserialize)]
pub struct JupyterHeader {
    message_id:String,
    message_type:JupyterMessageType,
    username:String,
    session:String,
    date:String,
    version:String
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
    pub header        : Bytes, // JSON Dict
    pub parent_header : Bytes, // JSON Dict
    pub metadata      : Bytes, // JSON Dict
    pub content       : JupyterMessageContent, // JSON Dict
    pub extra_buffers : Vec<Bytes>
}

impl TryFrom<ZmqMessage> for JupyterMessage{
    type Error = anyhow::Error;
    fn try_from(message:ZmqMessage) -> Result<Self> {
        let f = b"12";
        // find the index of the delimiter
        let frames = message.into_vec();
        let delimiter_index = frames.iter().position(|frame| frame == &DELIMITER).unwrap();
        println!("Found delimiter_index; {delimiter_index}");
        let content_bytes = frames[delimiter_index + 4].clone();
        println!("Try to parse content: {content_bytes:?}");
        let content:JupyterMessageContent = match serde_json::from_reader(
            Cursor::new(&frames[delimiter_index + 5])
        ) {
            Ok(item)=>item,
            Err(e)=>{
                println!("Error parsing content: {e}");
                JupyterMessageContent::default()
            }
        };
        Ok(JupyterMessage{
            identities    : frames[0..delimiter_index].into(),
            signature     : frames[delimiter_index + 1].clone(),
            header        : frames[delimiter_index + 2].clone(),
            parent_header : frames[delimiter_index + 3].clone(),
            metadata      : frames[delimiter_index + 4].clone(),
            content       ,
            extra_buffers : frames[delimiter_index + 6..].into(),
        })
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
}