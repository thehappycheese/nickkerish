use serde::{Serialize, Deserialize};
use super::message_type::JupyterMessageType;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Header{
    #[serde(rename="msg_id")]
    pub message_id:String,
    #[serde(rename="msg_type")]
    pub message_type:Option<JupyterMessageType>, // TODO: why is this optional?
    pub username:String,
    pub session:String,
    pub date:String,
    pub version:String
}
impl Header{
    pub fn with_id_type_date(&self, message_id:String, message_type:JupyterMessageType, date:String) -> Self {
        Self{
            message_id,
            message_type:Some(message_type),
            username: self.username.clone(),
            session: self.session.clone(),
            date,
            version: self.version.clone()
        }
    }
}