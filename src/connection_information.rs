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

/// Represents the JSON connection file created by the client (eg vscode or jupyter lab) and read by
/// the kernel
#[derive(Debug, Deserialize)]
pub struct ConnectionInformation {
    /// The IP address of the kernel
    #[serde(rename="ip")]
    pub ip_address: IpAddr,
    /// the only valid value for `signature_scheme` appears to be `"hmac-sha256"` or perhaps `""`
    /// is allowed if `key` is also `""`
    pub signature_scheme: SignatureScheme,
    /// Typically a UUID when signature scheme is specified.
    /// To disable message signing, set this to an empty string
    /// 
    /// TODO: Probably should be an option and the empty string should be a None variant
    pub key: String,
    /// Either `"tcp"`
    /// [Transmission Control Protocol](https://en.wikipedia.org/wiki/Transmission_Control_Protocol)
    /// (on unix `"ipc"`
    /// [Inter-Process Communication](https://en.wikipedia.org/wiki/Inter-process_communication) may
    /// also be an option, but this is not currently supported).
    pub transport: Transport,
    /// kernel name (Seems to match the name provided in `kernel.json` for the `language` property)
    pub kernel_name: String,

    /// The port used for bi-directional communication with the client. The kernel hosts a
    /// [zeromq::RouterSocket]. It allows multiple incoming connections from
    /// clients, and this is the socket where requests for code execution, object information,
    /// prompts, etc. are made to the kernel by any frontend. The communication on this socket is a
    /// sequence of request/reply actions from each frontend and the kernel.
    pub shell_port: u16,
    /// The kernel hosts a [zeromq::PubSocket] on this port that is used to broadcast state to all
    /// connected clients. All side effects (stdout, stderr, debugging events etc.) as well as the
    /// requests coming from any client over the shell socket and its own requests on the stdin
    /// socket. In a multi-client scenario, we want all clients to be able to know what each of the
    /// others other has sent to the kernel (this can be useful in collaborative scenarios, for
    /// example).
    pub iopub_port: u16,
    /// The kernel hosts a [zeromq::RouterSocket] on this port. The kernel uses this port to
    /// request terminal-style text input (`stdin` / Standard Input). The frontend that executed the
    /// code has a DEALER socket that acts as a ‘virtual keyboard’ for the kernel while this
    /// communication is happening.  In practice, frontends may display such kernel requests using a
    /// special input widget or otherwise indicating that the user is to type input for the kernel
    /// instead of normal commands in the frontend. 
    /// All messages are tagged with enough information (see ) for clients to know which
    /// messages come from their own interaction with the kernel and which ones are from other
    /// clients, so they can display each type appropriately.
    pub stdin_port: u16,
    /// The kernel hosts a [zeromq::RouterSocket] on this port used to receive shutdown messages (and
    /// other critical messages which should not be blocked by long-running execution requests) from
    /// the client. The control channel is also used for debugging messages.
    /// 
    /// This channel is identical to Shell, but operates on a separate socket to avoid queueing
    /// behind execution requests. 
    /// TODO: it is not clear if this means that any message which might be sent over the shell
    /// channel can alternatively be sent over the control channel and vice versa?
    pub control_port: u16,
    /// The kernel hosts a [zeromq::RepSocket] on this port which is used to echo back any heartbeat
    /// messages from clients to ensure they are still connected.
    /// The message content typically consists of a single frame `b"ping"` which the kernel must
    /// immediately return verbatim.
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
                self.ip_address,
                self.$port,
            );
            socket.bind(format!("{}://{}:{}", self.transport, self.ip_address, self.$port).as_str()).await?;
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
