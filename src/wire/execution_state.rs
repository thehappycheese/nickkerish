#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum KernelExecutionState {
    Busy,
    Idle,
    #[default]
    Starting,
}
