//! This module provides utility functions that are configuration-related

use crate::args::Args;
use crate::threads::ThreadInfo;
use clap::Parser;
use crate::validator::Validator;

/// Determines the total number of threads to use for application execution. These threads
/// are used by the Crawler, the Deleter, and the Logger
///
/// Once the number of initial threads is determined by the threads module, one additional
/// thread is added to the total. This thread will be dedicated to the logger functionality.
///
/// # Arguments
///
/// * `args` - Command-line arguments
///
/// # Returns
///
/// * `Result<ThreadInfo, Box<dyn std::error::Error + Send + Sync>>` - Ok with ThreadInfo if successful.
pub fn define_threads(args: &Args) -> Result<ThreadInfo, Box<dyn std::error::Error + Send + Sync>> {
    // take number of threads returned by ThreadInfo and add one extra thread for the logger
    let mut info = ThreadInfo::compute_thread_count(args)?;
    info.total_thread_count += 1;
    Ok(info)
}

/// Parses command-line arguments and validates them.
///
/// # Returns
///
/// * `Result<Args, Box<dyn std::error::Error + Send + Sync>>` - Ok with parsed Args if successful.
pub fn initialize_arguments() -> Result<Args, Box<dyn std::error::Error + Send + Sync>> {
    // parse arguments
    let mut args = Args::parse();
    // validate paths, logfile, thread count, and number of buffers
    Validator::validate(&mut args)?;
    Ok(args)
}
