use std::{
    path::PathBuf,
    fs,
    io::Write
};
use serde::Serialize;
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all="snake_case")]
enum InterruptMode{
    #[default]
    Signal,
    Message
}

#[derive(Debug, Serialize)]
struct KernelSpec{
    argv: Vec<String>,
    display_name: String,
    language: String,
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
pub fn kernel_spec() -> Result<()> {
    let current_executable_path = std::env::current_exe()
        .context("Failed to get current executable path")?;
    let current_executable_path_str = current_executable_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))?;

    // Define the content of kernel.json
    let kernel_spec = KernelSpec {
        argv: vec![
                format!("{current_executable_path_str}"),
                "run".to_owned(),
                "--connection-file".to_owned(),
                "{connection_file}".to_owned(),
            ],
        display_name: "Nickkerish".to_owned(),
        language: "nickkerish".to_owned(),
        interrupt_mode: None // Default is signal
    };

    let kernel_folder = PathBuf::from("Nickkerish");
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