use chrono::Local;

use std::path::PathBuf;

use std::io::Write::std::io;

use std::net::IpAddr::std;

use std::fs;

pub(crate) fn setup_logging() -> Result<fs::File, std::io::Error> {
    let mut path = PathBuf::from(std::env::current_exe()?.parent().unwrap());
    path.push("../../debug_logs");

    std::fs::create_dir_all(&path)?;

    let datetime = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    path.push(format!("log_{}.txt", datetime));

    fs::OpenOptions::new().create_new(true).write(true).open(path)
}

pub(crate) fn log(file: &mut fs::File, message: &str) -> Result<(), std::io::Error> {
    writeln!(file, "{}", message)
}
