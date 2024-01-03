use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all="snake_case")]
pub enum KernelExecutionState {
    Busy,
    Idle,
    #[default]
    Starting,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct PublishKernelStatus {
    pub execution_state: KernelExecutionState
}