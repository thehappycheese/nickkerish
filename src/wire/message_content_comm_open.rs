use serde::{Serialize, Deserialize};


/// A request for accessing message content history.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CommOpen {
    comm_id: String,
    target_name: String,
    data: serde_json::Value
}