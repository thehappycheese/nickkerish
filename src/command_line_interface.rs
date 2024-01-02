use clio::Input;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) enum CommandLineInterface {
    /// run the server
    #[command()]
    Run {
        /// Path to the connection file supplied by jupyter or vscode.
        /// This is a json file that contains the ip address, ports and other connection metadata.
        #[arg(long)]
        connection_file: Input,
    },
    /// create a new kernel.json and install it by running `jupyter kernelspec install --user [...]`
    #[command()]
    InstallKernelSpec
}
