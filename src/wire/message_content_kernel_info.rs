
use serde::{Serialize, Deserialize};

use super::{ReplyStatus, KERNEL_MESSAGING_VERSION};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InfoLink {
    text: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LanguageInfo {
    /// Name of the programming language that the kernel implements.
    /// Kernel included in IPython returns 'python'.
    name: String,

    /// Language version number.
    /// It is Python version number (e.g., '2.7.3') for the kernel
    /// included in IPython.
    version: String,

    /// mimetype for script files in this language (probably just text/plain ?) 
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

impl Default for LanguageInfo {
    fn default() -> Self {
        Self {
            name               : "uiua".to_owned(),
            version            : "v0.1.0".to_owned(),
            mimetype           : "text/x-uiua".to_owned(),
            file_extension     : ".ua".to_owned(),
            pygments_lexer     : Default::default(),
            codemirror_mode    : Default::default(),
            nbconvert_exporter : Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct KernelInfoReply {
    /// 'ok' if the request succeeded or 'error',
    /// with error information as in all other replies.
    pub status: ReplyStatus,
    /// Version of messaging protocol.
    /// The first integer indicates major version.  It is incremented when
    /// there is any backward incompatible change.
    /// The second integer indicates minor version.  It is incremented when
    /// there is any backward compatible change.
    /// 
    /// > NOTE: in practice this appears to accept `X.Y` but the spec calls for `X.Y.Z`
    pub protocol_version: String,
    /// The implementation name of the kernel.
    /// 
    /// The [default](Default::default()) value is taken from `Cargo.toml::package.name` using
    /// `env!("CARGO_PKG_NAME")`
    /// 
    /// e.g. 'ipython'
    pub implementation: String,
    /// Implementation version of the kernel using three part `X.Y.Z`
    /// [Semantic Versioning](https://semver.org/)
    /// 
    /// The [default](Default::default()) value is taken from `Cargo.toml::package.version` using
    /// `env!("CARGO_PKG_VERSION")`
    /// 
    /// > NOTE: Should be a three part semantic version number like `X.Y.Z`
    pub implementation_version: String,

    /// Information about the language of code the kernel is designed to execute
    pub language_info: LanguageInfo,

    /// A banner of information about the kernel,
    /// which may be displayed in console environments.
    /// 
    /// The [default](Default::default()) value is taken from `Cargo.toml::package.description`
    /// using `env!("CARGO_PKG_DESCRIPTION")`
    /// 
    /// e.g. `Python 3.10.10 | packaged by conda-forge | (main, Mar 24 2023, 20:00:38)
    /// [MSC v.1934 64 bit (AMD64)] Type 'copyright', 'credits' or 'license' for more
    /// information IPython 8.11.0 -- An enhanced Interactive Python. Type '?' for help.`
    pub banner: String,

    /// A boolean flag which tells if the kernel supports debugging in the notebook.
    /// Default is False
    pub debugger: bool,

    /// Optional: A list of dictionaries, each with keys 'text' and 'url'.
    /// These will be displayed in the help menu in the notebook UI.
    /// 
    /// e.g. ` [{"text": "Python Reference", "url": "https://docs.python.org/3.10"},
    /// {"text": "IPython Reference", "url": "https://ipython.org/documentation.html"},
    /// {"text": "NumPy Reference", "url": "https://docs.scipy.org/doc/numpy/reference/"}, ...`
    pub help_links: Vec<InfoLink>,
}

impl Default for KernelInfoReply {
    fn default() -> Self {
        Self {
            status                 : ReplyStatus::Ok,
            protocol_version       : KERNEL_MESSAGING_VERSION.into(),
            implementation         : env!("CARGO_PKG_NAME").into(),
            implementation_version : env!("CARGO_PKG_VERSION").into(),
            language_info          : Default::default(),
            banner                 : env!("CARGO_PKG_DESCRIPTION").into(),
            debugger               : false,
            help_links             : vec![InfoLink {
                text: "Nickkerish Repo".to_owned(),
                url: "https://github.com/thehappycheese/nickkerish".to_owned(),
            }],
        }
    }
}