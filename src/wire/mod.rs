mod header;
mod message_kernel_info;
mod message_type;
mod message;
mod reply_status;
mod message_content;

pub use reply_status::JupyterReplyStatus;
pub use message::Message;
pub use message_type::JupyterMessageType;
pub use header::Header;
pub use message_kernel_info::JupyterKernelInfoReply;
pub use message_content::JupyterMessageContent;
pub type HmacSha256 = hmac::Hmac<sha2::Sha256>;



const DELIMITER: &[u8] = b"<IDS|MSG>";