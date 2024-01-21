use serde::{Deserialize, Serialize};


/// See docs for [StatusPublication]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all="snake_case")]
pub enum ExecutionState {
    Busy,
    Idle,
    #[default]
    Starting,
}

/// Published by the kernel at startup and before and after each request to indicate the 
/// execution state of the kernel.
/// 
/// This message type is used by front-ends to monitor the status of the kernel.
/// 
/// When a kernel receives a request and begins processing it, the kernel shall immediately publish
/// a status message with execution_state: 'busy'. When that kernel has completed processing the
/// request and has finished publishing associated IOPub messages, if any, it shall publish a status
/// message with execution_state: 'idle'. Thus, the outputs associated with a given execution shall
/// generally arrive between the busy and idle status messages associated with a given request.
/// 
/// The `starting` status is supposed to be sent just once at startup, but I have not seen it used
/// by ipython or evcxr
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct StatusPublication {
    /// When the kernel starts to handle a message, it will enter the 'busy'
    /// state and when it finishes, it will enter the 'idle' state.
    /// The kernel will publish state 'starting' exactly once at process startup.
    pub execution_state: ExecutionState
}

impl From<ExecutionState> for StatusPublication {
    fn from(value: ExecutionState) -> Self {
        StatusPublication {
            execution_state: value
        }
    }
}