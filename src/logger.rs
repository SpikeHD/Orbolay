use std::fmt::Display;

use chrono::Local;
use colored::Colorize;

pub enum LogKind {
  Info,
  Success,
  Warn,
  Error,
}

pub fn log(s: impl AsRef<str> + Display, kind: Option<LogKind>) {
  let status = match kind {
    Some(LogKind::Info) => "INFO".blue(),
    Some(LogKind::Success) => "DONE".green(),
    Some(LogKind::Warn) => "WARN".yellow(),
    Some(LogKind::Error) => "FAIL".red(),
    None => "INFO".blue(),
  };

  println!(
    "[{}] [{}] {}",
    Local::now().format("%Y-%m-%d %H:%M:%S"),
    status,
    s
  );
}

#[macro_export]
macro_rules! log {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Info))
  };
}

#[macro_export]
macro_rules! success {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Success))
  };
}

#[macro_export]
macro_rules! warn {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Warn))
  };
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Error))
  };
}