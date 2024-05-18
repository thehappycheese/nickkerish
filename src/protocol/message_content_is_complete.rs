use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IsCompleteRequest {
    code:String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all="snake_case")]
pub enum IsCompleteReplyStatus {
    /// code is ready to be executed
    Complete,
    /// code should prompt for another line
    Incomplete,
    /// code will typically be sent for execution, so that the user sees the error soonest.
    Invalid,
    /// if the kernel is not able to determine this. The frontend should also handle the kernel not
    /// replying promptly. It may default to sending the code for execution, or it may implement
    /// simple fallback heuristics for whether to execute the code (e.g. execute after a blank
    /// line).
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IsCompleteReply {
    pub status:IsCompleteReplyStatus,
    
    /// If status is 'incomplete', indent should contain the characters to use
    /// to indent the next line. This is only a hint: front-ends may ignore it
    /// and use their own auto-indentation rules. For other statuses, this
    /// field does not exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent: Option<String>,
}