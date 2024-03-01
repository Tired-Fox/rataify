use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use lazy_static::lazy_static;
use reqwest::StatusCode;

lazy_static! {
    static ref REQLogFile: Mutex<File> = Mutex::new(OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("spotify.log")
        .unwrap());
}

pub struct ResponseLogger;

impl ResponseLogger {
    pub async fn log_error(status: StatusCode, body: &str) {
        let log_file = &mut REQLogFile.lock().unwrap();
        let _ = log_file.write(format!("[{status}] {}\n", body.replace("\n", "")).as_bytes());
    }
}
