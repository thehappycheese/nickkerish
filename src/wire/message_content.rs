use super::{
    HistoryRequest,
    IsCompleteReply,
    IsCompleteRequest,
    KernelInfoReply,
    StatusPublication,
    ExecuteRequest,
    ExecuteReply,
    ExecuteResultPublication, CommOpen,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    KernelInfoReply(KernelInfoReply),
    HistoryRequest(HistoryRequest),
    
    ExecuteRequest(ExecuteRequest),
    ExecuteReply(ExecuteReply),
    ExecuteResultPublication(ExecuteResultPublication),
    KernelStatusPublication(StatusPublication),

    // TODO: due to the serde untagged the order of variants matters :( These must come below
    //       execution requests for now:
    IsCompleteRequest(IsCompleteRequest),
    IsCompleteReply(IsCompleteReply),

    CommOpen(CommOpen),
}

macro_rules! impl_from_message_content {
    // The macro will accept a series of pairs (VariantType, VariantName).
    ($($variant_type:ty => $variant_name:ident),+) => {
        $(
            impl From<$variant_type> for MessageContent {
                fn from(item: $variant_type) -> Self {
                    MessageContent::$variant_name(item)
                }
            }
        )+
    };
}

// Use the macro for each variant of the MessageContent enum.
impl_from_message_content! {
    KernelInfoReply => KernelInfoReply,
    StatusPublication => KernelStatusPublication,
    
    HistoryRequest => HistoryRequest,
    
    ExecuteReply => ExecuteReply,
    ExecuteRequest => ExecuteRequest,
    ExecuteResultPublication => ExecuteResultPublication,

    IsCompleteRequest => IsCompleteRequest,
    IsCompleteReply => IsCompleteReply
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_history_request() {
        let data =
            b"{\"raw\": true, \"output\": false, \"hist_access_type\": \"tail\", \"n\": 1000}";
        let history_request: MessageContent = serde_json::from_slice(data).unwrap();
        println!("{:?}", history_request);
    }
}
