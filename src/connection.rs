use std::{net::IpAddr, fmt::Display};
use serde::Deserialize;
use anyhow::Result;
use zeromq::Socket;
use tracing::debug;
#[derive(Debug, Deserialize)]
pub enum Transport {
    #[serde(alias="tcp",alias="TCP", rename(serialize = "tcp"))]
    Tcp,
    // #[serde(alias="ipc",alias="IPC", rename(serialize = "ipc"))]
    // Ipc // unix only
}

impl Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transport::Tcp => write!(f, "tcp"),
            // Transport::Ipc => write!(f, "icp"),
        }
    }
}
#[derive(Debug, Deserialize)]
pub enum SignatureScheme{
    #[serde(rename="hmac-sha256")]
    HmacSha256
}

/// Represents the JSON connection file received from vscode or jupyter lab
#[derive(Debug, Deserialize)]
pub struct ConnectionInformation {
    pub ip: IpAddr,
    pub signature_scheme: SignatureScheme,
    /// typically a UUID when signature scheme is specified
    pub key: String,
    /// Either TCP or IPC
    pub transport: Transport,
    /// kernel name (Seems to match the name provided in `kernel.json` for the `language` property)
    pub kernel_name: String,

    pub shell_port: u16,
    pub iopub_port: u16,
    pub stdin_port: u16,
    pub control_port: u16,
    /// heartbeat port
    #[serde(rename="hb_port")]
    pub heartbeat_port: u16,
}

macro_rules! create_socket {
    ($fname:ident, $socket_type:ty, $port:ident) => {
        pub async fn $fname(&self) -> Result<$socket_type> {
            let mut socket = <$socket_type>::new();
            println_debug!(
                "binding {} for {} {}://{}:{}",
                stringify!($socket_type),
                stringify!($fname),
                self.transport,
                self.ip,
                self.$port,
            );
            socket.bind(format!("{}://{}:{}", self.transport, self.ip, self.$port).as_str()).await?;
            Ok(socket)
        }
    };
}

impl ConnectionInformation {
    create_socket!(create_socket_shell    , zeromq::RouterSocket, shell_port    );
    create_socket!(create_socket_iopub    , zeromq::PubSocket   , iopub_port    );
    create_socket!(create_socket_stdin    , zeromq::RouterSocket, stdin_port    );
    create_socket!(create_socket_control  , zeromq::RouterSocket, control_port  );
    create_socket!(create_socket_heartbeat, zeromq::RepSocket   , heartbeat_port);
}
