use super::{
    HistoryRequest,
    IsCompleteReply,
    IsCompleteRequest,
    KernelInfoReply,
    StatusPublication,
    ExecuteRequest,
    ExecuteReply,
    ExecuteResultPublication,
    CommOpen,
    CommClose,
    CommMsg, 
    ExecuteInputPublication,
    StreamPublication,
};
use serde::{Deserialize, Serialize};

macro_rules! define_message_content_and_impl_from {
    ($($type:tt),*) => {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(untagged)]
        pub enum MessageContent {
            $(
                $type($type),
            )*
        }

        $(
            impl From<$type> for MessageContent {
                fn from(item: $type) -> Self {
                    MessageContent::$type(item)
                }
            }
        )*
    }
}
define_message_content_and_impl_from!(
    KernelInfoReply,
    HistoryRequest,
    ExecuteRequest,
    ExecuteReply,
    ExecuteInputPublication,
    ExecuteResultPublication,
    StatusPublication,
    StreamPublication,
    IsCompleteRequest,
    IsCompleteReply,
    CommOpen,
    CommClose,
    CommMsg
);



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
