use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ExecuteRequest {
    /// Source code to be executed by the kernel, one or more lines.
    pub code: String,
    /// # A boolean flag which, if True, signals the kernel to execute
    /// this code as quietly as possible.
    /// silent=True forces store_history to be False,
    /// and will *not*:
    ///   - broadcast output on the IOPUB channel
    ///   - have an execute_result
    /// The default is False.
    pub silent: bool,
    /// A boolean flag which, if True, signals the kernel to populate history
    /// The default is True if silent is False.  If silent is True, store_history
    /// is forced to be False.
    pub store_history: bool,

    /// A dict mapping names to expressions to be evaluated in the
    /// user's dict. The rich display-data representation of each will be evaluated after execution.
    /// See the display_data content for the structure of the representation data.
    pub user_expressions: HashMap<String, String>,

    /// Some front-ends do not support stdin requests.
    /// If this is true, code running in the kernel can prompt the user for input
    /// with an input_request message (see below). If it is false, the kernel
    /// should not send these messages.
    pub allow_stdin: bool,

    /// A boolean flag, which, if True, aborts the execution queue if an exception is encountered.
    /// If False, queued execute_requests will execute even if this request generates an exception.
    pub stop_on_error: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecuteReplyStatus {
    Ok,
    Error,
    Aborted,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ExecuteReply {
    pub status: ExecuteReplyStatus,
    /// The global kernel counter that increases by one with each request that
    /// stores history.  This will typically be used by clients to display
    /// prompt numbers to the user.  If the request did not store history, this will
    /// be the current value of the counter in the kernel.
    /// 
    /// The kernel should have a single, monotonically increasing counter of all execution requests
    /// that are made with store_history=True. This counter is used to populate the In[n] and Out[n]
    /// prompts. The value of this counter will be returned as the execution_count field of all
    /// execute_reply and execute_input messages.
    pub execution_count: i32,

    /// present when status is Ok
    /// 
    /// 'payload' will be a list of payload dicts, and is optional.
    /// payloads are considered deprecated.
    /// The only requirement of each payload dict is that it have a 'source' key,
    /// which is a string classifying the payload (e.g. 'page').
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Vec<HashMap<String, String>>>,

    /// present when status is Ok
    /// 
    /// Results for the user_expressions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_expressions: Option<HashMap<String, String>>,
}


/// Results can have multiple simultaneous formats depending on its configuration. A plain text
/// representation should always be provided in the text/plain mime-type. Frontends are free to
/// display any or all of these according to its capabilities. Frontends should ignore mime-types
/// they do not understand. The data itself is any JSON object and depends on the format. It is
/// often, but not always a string.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ExecuteResultPublication{
    /// The counter for this execution is also provided so that clients can
    /// display it, since IPython automatically creates variables called _N
    /// (for prompt N).
    pub execution_count : usize,

    /// The data dict contains key/value pairs, where the keys are MIME
    /// types and the values are the raw data of the representation in that
    /// format.
    /// 
    /// the object being displayed is that passed to the display hook,
    /// i.e. the *result* of the execution.
    // TODO: Should the map be to a Json Value?
    pub data : serde_json::Value,

    /// Any metadata that describes the data
    pub metadata : serde_json::Map<String, serde_json::Value>,
}

/// To let all frontends know what code is being executed at any given time, these messages contain
/// a re-broadcast of the code portion of an execute_request, along with the execution_count.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ExecuteInputPublication{
    /// Source code to be executed, one or more lines
    pub code:String,
    ///  The counter for this execution is also provided so that clients can display it, since
    /// IPython automatically creates variables called _iN (for input prompt In[N]).
    pub execution_count:usize,
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct StreamPublication{
    /// The name of the stream is one of 'stdout', 'stderr'
    pub name:String,
    /// The text is an arbitrary string to be written to that stream
    pub text:String,
}