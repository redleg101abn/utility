//! This module defines the command-line arguments for the application.

use clap::Parser;
use chrono::Local;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "A multi-threaded tool for the massive deletion of files and directories.",
    long_about = "Nukem is a command-line tool designed to help you delete massive amounts of files and \
                  directories efficiently. It supports various options such as dry-run mode, specifying \
                  number of threads and buffers, and verbose logging to the console or to a logfile. \
                  You can provide multiple paths, and it will process each according to the specified options.",
    help_template = "\
--------------------------
{bin} {version}
--------------------------
{about}
--------------------------

Usage: {usage}

{all-args}"
)]

pub struct Args {
    /// Full path(s) to the file(s) or directory(s) that will be deleted. This is the only required
    /// field.
    #[clap(required = true)]
    pub paths: Vec<PathBuf>,

    /// Full path of the directory for the logfile
    #[clap(short = 'l', long = "logfile_path")]
    pub logfile_path: Option<PathBuf>,

    /// Number of threads to use for file and directory deletion. It cannot be zero or greater than 64.
    #[clap(short = 't', long = "threads")]
    pub threads: Option<usize>,

    /// Enable verbose logging
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Number of buffers to use for file and directory deletion. Allowable values are between 100 and 2000.
    #[clap(short = 'b', long = "buffer", default_value = "100")]
    pub buffer_size: usize,

    /// Perform a dry run without deleting any files or directories
    #[clap(short = 'd', long = "dry-run")]
    pub dry_run: bool,
}

impl Args {
    /// Ensure that the given path to the logfile location exists, then create the name for the logfile
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The formatted logfile name.
    pub fn resolve_logfile_name(&self) -> Option<String> {
        // if the user specified '-l', generate date/time stamped logfile name
        if let Some(ref path) = self.logfile_path {
            let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
            let log_filepath = path.to_str().unwrap_or("").to_string();
            Some(format!("{}/nukem_{}.log", log_filepath, timestamp))
        } else {
            None
        }
    }
}
