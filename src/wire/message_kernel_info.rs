
use serde::{Serialize, Deserialize};

use super::JupyterReplyStatus;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JupyterKernelInfoLink {
    text: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JupyterKernelInfoLanguageInfo {
    /// Name of the programming language that the kernel implements.
    /// Kernel included in IPython returns 'python'.
    name: String,

    /// Language version number.
    /// It is Python version number (e.g., '2.7.3') for the kernel
    /// included in IPython.
    version: String,

    /// mimetype for script files in this language
    mimetype: String,

    /// Extension including the dot, e.g. '.py'
    file_extension: String,

    /// pygments lexer, for highlighting
    /// Only needed if it differs from the 'name' field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pygments_lexer: Option<String>,

    /// Codemirror mode, for highlighting in the notebook.
    /// Only needed if it differs from the 'name' field.
    /// TODO: apparently this should also accept a dict?
    // skip if none
    #[serde(skip_serializing_if = "Option::is_none")]
    codemirror_mode: Option<String>,

    /// nbconvert exporter, if notebooks written with this kernel should
    /// be exported with something other than the general 'script'
    /// exporter.
    #[serde(skip_serializing_if = "Option::is_none")]
    nbconvert_exporter: Option<String>,
}

impl Default for JupyterKernelInfoLanguageInfo {
    fn default() -> Self {
        Self {
            name: "nickkerish".to_owned(),
            version: "0.1.0".to_owned(),
            mimetype: "text/plain".to_owned(),
            file_extension: ".nk".to_owned(),
            pygments_lexer: Default::default(),
            codemirror_mode: Default::default(),
            nbconvert_exporter: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JupyterKernelInfoReply {
    /// 'ok' if the request succeeded or 'error',
    /// with error information as in all other replies.
    pub(crate) status: JupyterReplyStatus,
    /// Version of messaging protocol.
    /// The first integer indicates major version.  It is incremented when
    /// there is any backward incompatible change.
    /// The second integer indicates minor version.  It is incremented when
    /// there is any backward compatible change.
    pub(crate) protocol_version: String,
    /// The kernel implementation name
    /// (e.g. 'ipython' for the IPython kernel)
    pub(crate) implementation: String,
    /// Implementation version number.
    /// The version number of the kernel's implementation
    /// (e.g. IPython.__version__ for the IPython kernel)
    pub(crate) implementation_version: String,

    /// Information about the language of code for the kernel
    pub(crate) language_info: JupyterKernelInfoLanguageInfo,

    /// A boolean flag which tells if the kernel supports debugging in the notebook.
    /// Default is False
    pub(crate) banner: String,

    /// A boolean flag which tells if the kernel supports debugging in the notebook.
    /// Default is False
    pub(crate) debugger: bool,

    /// Optional: A list of dictionaries, each with keys 'text' and 'url'.
    /// These will be displayed in the help menu in the notebook UI.
    pub(crate) help_links: Vec<JupyterKernelInfoLink>,
}

impl Default for JupyterKernelInfoReply {
    fn default() -> Self {
        Self {
            status: JupyterReplyStatus::Ok,
            protocol_version: "5.4.0".to_owned(),
            implementation: "nickkerish_kernel".to_owned(),
            implementation_version: "0.1.0".to_owned(),
            language_info: Default::default(),
            banner: Default::default(),
            debugger: Default::default(),
            help_links: vec![JupyterKernelInfoLink {
                text: "Nickkerish Repo".to_owned(),
                url: "https://github.com/thehappycheese/nickkerish".to_owned(),
            }],
        }
    }
}