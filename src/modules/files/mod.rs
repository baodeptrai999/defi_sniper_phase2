use chrono::Local;
use std::fs::{File, create_dir_all, OpenOptions};
use std::io::{BufWriter, Write};
use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::*;

lazy_static! {
  static ref LOG_WRITER: Mutex<BufWriter<File>> = {
    create_dir_all(LOG_DIR).expect("Failed to create log directory.");

    let file_name = format!("{}/log_{}", LOG_DIR, Local::now().format("%Y-%m-%d_%H"));
    let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(file_name)
    .expect("Failed to open log file");
  Mutex::new(BufWriter::new(file))
  };
}

pub fn log_to_file(message: &str){
  let mut writer = LOG_WRITER.lock().expect("Failed to lock log writer mutex.");
  writeln!(writer, "{}", message).expect("Failed to write log to file.");
  writer.flush().expect("Failed to flush log writer");
}