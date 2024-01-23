mod header;
mod message_type;
mod message;
mod message_reply_status;
mod message_content;

mod message_content_status;
mod message_content_kernel_info;
mod message_content_error;
mod message_content_history;
mod message_content_is_complete;
mod message_content_execute;
mod message_content_comm;

pub use message_reply_status::ReplyStatus;
pub use message::{MessageBytes, MessageParsed};
pub use message_type::MessageType;
pub use header::Header;
pub use message_content::MessageContent;
pub use message_content_status::{ExecutionState, StatusPublication};
pub use message_content_kernel_info::KernelInfoReply;
pub use message_content_error::ErrorReply;
pub use message_content_history::{HistoryAccessType, HistoryRequest};
pub use message_content_is_complete::{IsCompleteReply, IsCompleteRequest, IsCompleteReplyStatus};
pub use message_content_execute::{ExecuteReply, ExecuteRequest, ExecuteReplyStatus, ExecuteResultPublication, ExecuteInputPublication, StreamPublication};
pub use message_content_comm::{CommOpen, CommClose, CommMsg};

pub type HmacSha256 = hmac::Hmac<sha2::Sha256>;
pub const DELIMITER: &[u8] = b"<IDS|MSG>";
pub const KERNEL_MESSAGING_VERSION:&'static str = "5.3";