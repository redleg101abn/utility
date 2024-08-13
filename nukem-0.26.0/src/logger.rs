//! This module provides functionality for logging messages to the console and a logfile,
//! if the user specified one.
//!
//! There are two commandline options that determine how the logging system operates:
//! '-v' : If the verbose option was specified, all messages will be printed. If '-v' was not
//!        specified, then only the reports are printed
//! '-l' : If this was specified, a logfile is created and all events will be written to it, in
//!        addition to the console.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Sender};
use chrono::Local;

/// The `Logger` structure is responsible for logging messages.
pub struct Logger {
    sender: Sender<String>,
    logfile: Option<Arc<Mutex<std::fs::File>>>,
    verbose: bool,
}

impl Logger {
    /// Creates a new `Logger` instance.
    ///
    /// # Arguments
    ///
    /// * `logfile_path` - An optional path to the logfile.
    /// * `verbose` - A boolean indicating whether verbosity is enabled.
    ///
    /// # Returns
    ///
    /// * `Arc<Self>` - A pointer to the `Logger` instance.
    ///
    /// # Panics
    ///
    /// This function will panic if it fails to open the log file.
    pub fn new(logfile_path: Option<String>, verbose: bool, buffer_size: usize) -> Arc<Self> {
        // set send and receive mpsc channel buffer size
        let (tx, mut rx) = mpsc::channel(buffer_size);
        let logfile = logfile_path.map(|path| {
            Arc::new(Mutex::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .expect("Failed to open log file"),
            ))
        });

        // create logger instance
        let logger = Arc::new(Logger { sender: tx, logfile, verbose });

        let logger_clone = Arc::clone(&logger);
        // spawn a task that listens for messages on the receiving end ('rx')
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                // print the message to the console
                println!("{}", msg);
                // if there is a logfile...
                if let Some(ref file) = logger_clone.logfile {
                    // lock the logfile for safe, exclusive access
                    let mut file = file.lock().unwrap();
                    // write message to logfile
                    if let Err(e) = writeln!(file, "{}", msg) {
                        eprintln!("Failed to write to log file: {:?}", e);
                    }
                }
            }
        });

        logger
    }

    /// Logs a message with an optional timestamp and error flag.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to log.
    /// * `is_error` - A boolean indicating whether the message is an error.
    /// * `timestamp` - A boolean indicating whether to include a timestamp.
    /// * `verbose` - A boolean indicating whether to log only if verbosity is enabled.
    pub async fn log(&self, message: &str, is_error: bool, timestamp: bool, verbose: bool) {
        // prevent messages from being logged unless the logger itself is in verbose mode
        if verbose && !self.verbose {
            return;
        }
        // if message has a timestamp or if 'verbose' has been selected
        let formatted_message = if timestamp || verbose {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            if is_error {
                format!("[ERROR][{}] {}", ts, message)
            } else if verbose {
                format!("[INFO][{}] {}", ts, message)
            } else {
                format!("[{}] {}", ts, message)
            }
        } else if is_error {
            format!("[ERROR] {}", message)
        } else if verbose {
            format!("[INFO] {}", message)
        } else {
            message.to_string()
        };

        if is_error {
            eprintln!("{}", formatted_message);
        }

        self.send_message(formatted_message).await;
    }

    /// Sends a log message through the mpsc channel.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send.
    async fn send_message(&self, message: String) {
        if self.sender.send(message).await.is_err() {
            eprintln!("Failed to send message to logger");
        }
    }
}
