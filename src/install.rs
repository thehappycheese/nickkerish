use std::{
    path::PathBuf,
    fs,
    io::Write
};
use serde::Serialize;
use anyhow::{Context, Result};

/// The way in which the client should try to to interrupt cell execution on this kernel,
#[derive(Debug, Serialize, Default)]
#[serde(rename_all="snake_case")]
enum InterruptMode{
    /// Use the operating system’s signalling facilities (e.g. SIGINT on POSIX systems)
    #[default]
    Signal,
    /// Send an interrupt_request message on the control channel (see Kernel interrupt).
    Message
}

#[derive(Debug, Serialize)]
struct KernelSpec{
    /// A list of command line arguments used to start the kernel. The text {connection_file} in any
    /// argument will be replaced with the path to the connection file.
    #[serde(rename="argv")]
    shell_kernel_run_command: Vec<String>,

    /// User visible name of the kernel "Rust", "Python"
    display_name: String,

    /// Generally a lower case version of the language name "rust" or "python"
    language: String,

    /// The way in which the client should try to to interrupt cell execution on this kernel,
    /// - `"signal"` (default) the operating system’s signalling facilities (e.g. SIGINT on POSIX
    ///   systems)
    /// - `"message"` sending an interrupt_request message on the control channel (see Kernel
    ///   interrupt).
    #[serde(skip_serializing_if = "Option::is_none")]
    interrupt_mode: Option<InterruptMode>
}

/// Installs the generates `./nickerish/kernel.json` and calls
/// 
/// ```bash
/// jupyter kernelspec install --user ./nickerish
/// ```
/// 
/// TODO: Add better error explaining to the user what to do if the `jupyter kernelspec` command fails
/// TODO: Add flag to make the `--user` flag optional
/// 
/// TODO: We can try manually install this according to the standard paths specified here
///       https://jupyter-client.readthedocs.io/en/latest/kernels.html#kernel-specs
pub fn kernel_spec() -> Result<()> {
    let current_executable_path = std::env::current_exe()
        .context("Failed to get current executable path")?;
    let current_executable_path_str = current_executable_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))?;

    // Define the content of kernel.json
    let kernel_spec = KernelSpec {
        shell_kernel_run_command: vec![
                format!("{current_executable_path_str}"),
                "run".to_owned(),
                "--connection-file".to_owned(),
                "{connection_file}".to_owned(),
            ],
        display_name: "NickkerishUiua".to_owned(),
        language: "uiua".to_owned(),

        // Lets try message, since vscode just flat out does not seem to play nice and I am just
        // trying random stuff now
        interrupt_mode: Some(InterruptMode::Message)
    };

    let kernel_folder = PathBuf::from("NickkerishUiua");
    fs::create_dir_all(&kernel_folder)
        .context("Failed to create kernel folder")?;

    let mut kernel_file = fs::File::create(kernel_folder.join("kernel.json"))
        .context("Failed to create kernel.json file")?;
    kernel_file.write_all(serde_json::to_string_pretty(&kernel_spec)?.as_bytes())
        .context("Failed to write to kernel.json file")?;

    let output = std::process::Command::new("jupyter")
        .arg("kernelspec")
        .arg("install")
        .arg("--user")
        .arg(&kernel_folder)
        .output()
        .context("Failed to execute jupyter kernelspec install command")?;

    if !output.status.success() {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Jupyter kernelspec installation failed");
    }

    println!("Kernel installed successfully");
    Ok(())
}