//! This module provides various utilitarian functions used across the application.

use std::sync::Arc;
use crate::args::Args;
use crate::logger::Logger;
use crate::threads::ThreadInfo;
use crate::deleter::Deleter;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use std::path::PathBuf;
use std::time::Instant;

/// Sets up channels for inter-task communication.
///
/// # Arguments
///
/// * `args` - The parsed command-line arguments.
///
/// # Returns
///
/// * `(mpsc::Sender<PathBuf>, mpsc::Sender<PathBuf>, Arc<Mutex<mpsc::Receiver<PathBuf>>>, Arc<Mutex<mpsc::Receiver<PathBuf>>>)`
pub fn setup_channels(args: &Args) -> (mpsc::Sender<PathBuf>, mpsc::Sender<PathBuf>, Arc<Mutex<mpsc::Receiver<PathBuf>>>, Arc<Mutex<mpsc::Receiver<PathBuf>>>) {
    // communication channels use the buffer_size specified by the '-b' commandline option
    let (file_sender, file_receiver) = mpsc::channel(args.buffer_size);
    let (dir_sender, dir_receiver) = mpsc::channel(args.buffer_size);
    let file_receiver = Arc::new(Mutex::new(file_receiver));
    let dir_receiver = Arc::new(Mutex::new(dir_receiver));
    (file_sender, dir_sender, file_receiver, dir_receiver)
}

/// Informational report that shows paths, threads, and workers.
///
/// # Arguments
///
/// * `args` - Command-line arguments.
/// * `logger` - An instance of the `Logger`.
/// * `thread_info` - Information about the threads being used.
/// * `worker_tasks_count` - The number of worker tasks.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Ok if successful, Err otherwise.
pub async fn print_info(args: &Args, logger: &Arc<Logger>, thread_info: &ThreadInfo, worker_tasks_count: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // calculate variables used by report
    let thread_count = thread_info.total_thread_count;
    let core_count = thread_info.core_count;
    let full_logfile_name = args.resolve_logfile_name().unwrap_or_else(|| "None".into());
    let buffer_size = args.buffer_size;

    // print the report
    logger.log(&format!("Logfile path: {}", full_logfile_name), false, false, false).await;
    logger.log(&format!("Core count: {}", core_count), false, false, false).await;
    logger.log(&format!("Threads: {}", thread_count), false, false, false).await;
    logger.log(&format!("Worker tasks count: {}", worker_tasks_count), false, false, false).await;
    logger.log(&format!("Number of Buffers: {}", buffer_size), false, false, false).await;
    logger.log("----------------------------------------------------------------", false, false, false).await;
    Ok(())
}

/// Prints a summary of the filesystem objects that the Crawler found.
///
/// # Arguments
///
/// * `total_directories` - Total number of directories found.
/// * `total_files_symlinks` - Total number of files and symlinks found.
/// * `logger` - An instance of the `Logger`.
pub async fn print_crawler_summary(
    total_directories: usize,
    total_files_symlinks: usize,
    logger: &Arc<Logger>,
) {
    logger.log("----------------------------------------------------------------", false, false, false).await;
    logger.log(&format!("Total directories: {}", total_directories), false, false, false).await;
    logger.log(&format!("Total files and symlinks: {}", total_files_symlinks), false, false, false).await;
}

/// Prints a final report of deletion statistics.
///
/// # Arguments
///
/// * `deleter` - An instance of the `Deleter`.
/// * `logger` - An instance of the `Logger`.
/// * `elapsed` - The duration of the application run.
/// * `total_operations` - The total number of metadata operations performed.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error + Send + Sync>>` - Ok if successful, Err otherwise.
pub async fn print_final_report(deleter: &Deleter, logger: &Arc<Logger>, elapsed: Duration, total_operations: usize) {
    //outputs total size of deleted files in MB, which is more user-friendly
    let total_size_in_megabytes = deleter.get_total_size().await as f64 / 1024.0 / 1024.0;
    //keep track of the number of objects that couldn't be deleted
    let failed_deletions = *deleter.failed_deletions.lock().await;
    // number of seconds elapsed since application began
    let elapsed_secs = elapsed.as_secs_f64();
    //compute operations per second
    let ops_per_sec = if elapsed_secs > 0.0 {
        total_operations as f64 / elapsed_secs
    } else {
        0.0
    };

    // print the report
    logger.log("----------------------------------------------------------------", false, false, false).await;
    logger.log(&format!("Failed deletions: {}", failed_deletions), false, false, false).await;
    logger.log(&format!("Deletion completed. Total size: {:.2} MB", total_size_in_megabytes), false, false, false).await;
    logger.log(&format!("Execution time: {:?}", elapsed), false, false, false).await;
    logger.log(&format!("Metadata operations per second: {:.2} ops/s", ops_per_sec), false, false, false).await;
    logger.log("--------------- Application Run Complete -----------------------", false, false, false).await;
}

/// Finalizes the application by printing summaries and reports.
///
/// # Arguments
///
/// * `deleter` - A reference to the Deleter.
/// * `logger` - A reference to the Logger.
/// * `start` - The start time of the application.
/// * `total_directories` - Total count of directories deleted.
/// * `total_files_symlinks` - Total count of files and symlinks deleted.
/// * `total_crawling_ops` - Total count of Crawler metadata operations.
/// * `total_stat_ops` - Total count of filesystem stat metadata operations.
/// * `total_deletion_ops` - Total count of deletion metadata operations.
pub async fn finalize(
    deleter: &Arc<Mutex<Deleter>>, logger: &Arc<Logger>, start: Instant, total_directories: Arc<Mutex<usize>>,
    total_files_symlinks: Arc<Mutex<usize>>, total_crawling_ops: Arc<Mutex<usize>>, total_stat_ops: Arc<Mutex<usize>>,
    total_deletion_ops: Arc<Mutex<usize>>
) {
    // get values for variables
    let total_directories = *total_directories.lock().await;
    let total_files_symlinks = *total_files_symlinks.lock().await;
    let total_crawling_ops = *total_crawling_ops.lock().await;
    let total_stat_ops = *total_stat_ops.lock().await;
    let total_deletion_ops = *total_deletion_ops.lock().await;

    // compute total number of metadata operations
    let total_operations = total_crawling_ops + total_stat_ops + total_deletion_ops;
    // print the summary of crawler activity
    print_crawler_summary(total_directories, total_files_symlinks, &logger).await;
    // wait for deleter tasks to finish, then shutdown the deleter workers
    let deleter = deleter.lock().await;
    deleter.shutdown().await;
    // calculate elapsed time of application run
    let elapsed = start.elapsed();
    // print final report
    print_final_report(&*deleter, &logger, elapsed, total_operations).await;
}
