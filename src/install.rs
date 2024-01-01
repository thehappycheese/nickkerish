use std::{path::PathBuf, fs};

use serde::Serialize;



/// 
#[derive(Debug, Serialize)]
struct KernelSpec{
    argv: Vec<String>,
    display_name: String,
    language: String,
}


struct JupyterKernelspecInstallError(str);
// Implement the Display trait. This is used for converting the error to a string.
impl fmt::Display for JupyterKernelspecInstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JupyterKernelspecInstallError: {}", self.0)
    }
}
// Implement the Error trait. This trait doesn't require you to implement any methods,
// but your type must also implement the Display (and Debug, which is automatically
// derived) trait.
impl std::error::Error for JupyterKernelspecInstallError {}

pub fn kernel_spec() -> Result<(),Box<dyn std::error::Error>>{
    let current_executable_path = std::env::current_exe()?;
    let current_executable_path_str = current_executable_path.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to convert path to string")
    })?;

    // Define the content of kernel.json
    let kernel_spec = KernelSpec {
        argv: vec![
                format!("{current_executable_path_str}"),
                "run".to_owned(),
                "--connection-file".to_owned(),
                "{connection_file}".to_owned(),
            ],
        display_name: "Nickerish".to_owned(),
        language: "nickerish".to_owned(),
    };

    // Create a new folder named "Nickerish" in the current working directory
    let kernel_folder = PathBuf::from("Nickerish");
    fs::create_dir_all(&kernel_folder)?;

    // Create kernel.json inside the "Nickerish" folder
    let mut kernel_file = fs::File::create(kernel_folder.join("kernel.json"))?;
    kernel_file.write_all(serde_json::to_string_pretty(&kernel_spec)?.as_bytes())?;

    //now we will try to execute the shell command `jupyter kernelspec install {kernel_folder}`
    let output = std::process::Command::new("jupyter")
        .arg("kernelspec")
        .arg("install")
        .arg("--user")
        .arg(kernel_folder)
        .output()?;
    if !output.status.success() {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stderr)
    }else{
        println!("Kernel installed successfully");
        Ok(())
    }
}