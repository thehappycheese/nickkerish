mod header;
mod message_kernel_info;
mod message_type;
mod message;
mod reply_status;
mod message_content;
mod message_kernel_state;

pub use reply_status::JupyterReplyStatus;
pub use message::Message;
pub use message_type::JupyterMessageType;
pub use header::Header;
pub use message_kernel_info::JupyterKernelInfoReply;
pub use message_content::JupyterMessageContent;
pub use message_kernel_state::{KernelExecutionState, PublishKernelStatus};
pub type HmacSha256 = hmac::Hmac<sha2::Sha256>;



pub const DELIMITER: &[u8] = b"<IDS|MSG>";
pub const KERNEL_MESSAGING_VERSION:&'static str = "5.3";