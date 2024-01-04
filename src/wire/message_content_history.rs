/// TODO: For future reference this is the response format that needs to be implemented
/// 
/// content = {
///   # 'ok' if the request succeeded or 'error', with error information as in all other replies.
///   'status' : 'ok',
///   # A list of 3 tuples, either:
///   # (session, line_number, input) or
///   # (session, line_number, (input, output)),
///   # depending on whether output was False or True, respectively.
///   'history' : list,
/// }


use serde::{Serialize, Deserialize};

/// Enumeration of the different types of history access.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HistoryAccessType {
    /// Access the 'tail' of the history, i.e., the last few entries.
    Tail,

    /// Access a range within the history.
    Range,

    /// Search the history based on a pattern.
    Search,
}

/// A request for accessing message content history.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct HistoryRequest {
    /// If True, also return output history in the resulting dict.
    output: bool,

    /// If True, return the raw input history, else the transformed input.
    raw: bool,

    /// The type of history access requested: can be 'range', 'tail', or 'search'.
    #[serde(rename="hist_access_type")]
    history_access_type: HistoryAccessType,

    /// Number of history items to access.
    #[serde(rename="n")]
    number_of_items: usize,

    /// If hist_access_type is 'range', this is the session number.
    /// Session is a number that increments each time the kernel starts; you can specify
    /// a positive session number, or a negative number to count back from the current session.
    #[serde(rename="session")]
    kernel_session: Option<i32>,

    /// Start line (cell) number within the session (only for 'range').
    #[serde(rename="start")]
    line_start: Option<i32>,

    /// Stop line (cell) number within the session (only for 'range').
    #[serde(rename="stop")]
    line_stop: Option<i32>,

    /// If hist_access_type is 'search', this is the glob pattern for matching cells.
    #[serde(rename="pattern")]
    search_pattern: Option<String>,

    /// If hist_access_type is 'search' and unique is true, do not include duplicated history.
    /// Default is false.
    unique: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_history_request(){
        let data = b"{\"raw\": true, \"output\": false, \"hist_access_type\": \"tail\", \"n\": 1000}";
        let history_request: HistoryRequest = serde_json::from_slice(data).unwrap();
        println!("{:?}", history_request);
    }
}