use std::net::IpAddr;

/// Represents the JSON connection file received from vscode or jupyter lab
#[derive(Debug, serde::Deserialize)]
pub(crate) struct Connection {
    pub(crate) ip: IpAddr,
    pub(crate) key: String,
    pub(crate) transport: String,
    pub(crate) signature_scheme: String,
    pub(crate) kernel_name: String,
    pub(crate) shell_port: u16,
    pub(crate) iopub_port: u16,
    pub(crate) stdin_port: u16,
    pub(crate) control_port: u16,
    pub(crate) hb_port: u16,
}
