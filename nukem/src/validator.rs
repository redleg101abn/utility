//! This module provides functions to validate data in various parts of the application

use std::path::PathBuf;
use glob::glob;
use crate::args::Args;

/// General purpose validation module. If it needs to be validated, it happens here.
pub struct Validator {}

impl Validator {
    /// Validate all the user-provided arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - command-line arguments.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if validation passes, Error if validation fails.
    pub fn validate(args: &mut Args) -> Result<(), String> {
        Self::validate_paths(&args.paths)?;
        Self::validate_logfile_path(&args.logfile_path)?;
        Self::validate_buffer_size(args.buffer_size)?;
        Self::validate_thread_count(args.threads)?;
        Ok(())
    }

    /// Validate the user-provided path(s) to content that will be deleted.
    ///
    /// # Arguments
    ///
    /// * `paths` - A reference to a list of file paths to validate.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if all paths are valid, Err with a message if any path is invalid.
    fn validate_paths(paths: &[PathBuf]) -> Result<(), String> {
        for path in paths {
            // User supplied wildcard
            if path.to_string_lossy().contains("*") || path.to_string_lossy().contains("?") {
                // If the glob pattern resolves to a path that exists, continue. If the path does not exist, error
                let mut matched = false;
                for entry in glob(&*path.to_string_lossy()).map_err(|err| format!("Failed to read glob pattern: {}", err))? {
                    match entry {
                        Ok(path) => {
                            if !path.exists() {
                                return Err(format!("Path '{}' does not exist", path.display()));
                            }
                            matched = true;
                            break;
                        },
                        Err(e) => return Err(format!("Failed to read glob pattern: {}", e)),
                    }
                }
                if !matched {
                    return Err(format!("No paths matched the provided glob pattern: {}", path.display()));
                }
            } else {
                // Path does not have wildcards. If the path exists, return OK. Otherwise, return error
                if !path.exists() {
                    return Err(format!("Path '{}' does not exist", path.display()));
                }
            }
        }
        Ok(())
    }

    /// Validate the user-specified logfile path. The user should just specify the path to a
    /// directory, not a filename for the logfile. The filename is generated automatically, complete
    /// with date/time stamp.
    ///
    /// # Arguments
    ///
    /// * `logfile_path` - A reference to the logfile path to validate.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if the logfile path is valid or not provided, Error if the logfile path is invalid.
    fn validate_logfile_path(logfile_path: &Option<PathBuf>) -> Result<(), String> {
        if let Some(log_path) = logfile_path {
            // Ensure the provided path is a directory
            if !log_path.is_dir() {
                return Err(format!(
                    "Logfile path '{}' is not a directory or does not exist. Please provide a directory path.",
                    log_path.display()
                ));
            }
        }
        Ok(())
    }

    /// Validate the user-specified buffer size. This is used by the sender and receiver channels
    /// in the 'crawler' and 'deleter' processes. The range that we specify (between 100 and 2000) is
    /// somewhat arbitrary, but we did a lot of testing and determined that values outside of this
    /// range provided limited difference in performance.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - The buffer size to validate.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if the buffer size is within the allowed range, Error if the buffer size is invalid.
    fn validate_buffer_size(buffer_size: usize) -> Result<(), String> {
        // Buffer size must be between 100 and 2000. If the user made no specification of '-b' then
        // the default buffer size of 100 is used.
        if buffer_size < 100 || buffer_size > 2000 {
            return Err(format!("Invalid buffer size {}. The buffer size must be between 100 and 2000.", buffer_size));
        }
        Ok(())
    }

    /// Validate the user-supplied thread count. Allowable values are between 1 and 64. This range
    /// is based on extensive testing. Values outside of this range supplied negligible performance
    /// differences.
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads to validate.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if the thread count is within the allowed range, Error if the thread count is invalid.
    fn validate_thread_count(threads: Option<usize>) -> Result<(), String> {
        // Thread count must be between 1 and 64. If the user made no specification of '-t' then the
        // application will automatically calculate the number of threads to spawn
        if let Some(t) = threads {
            if t < 1 || t > 64 {
                return Err(format!("Invalid thread count {}. The thread count must be between 1 and 64.", t));
            }
        }
        Ok(())
    }
}
