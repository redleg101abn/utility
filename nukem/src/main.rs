//! Main entry point for the application. Controls program flow
//! and ensures that resources are cleaned up when complete.

mod args;
mod validator;
mod logger;
mod threads;
mod utility;
mod crawler;
mod deleter;
mod config;

use std::sync::Arc;
use std::time::Instant;
use std::path::PathBuf;
use tokio::sync::{mpsc, Mutex};
use crate::crawler::Crawler;
use crate::deleter::Deleter;
use crate::logger::Logger;
use crate::utility::{setup_channels, print_info, finalize};
use crate::config::{define_threads, initialize_arguments};
use crate::threads::ThreadInfo;
use crate::args::Args;

// this is an alias to improve readability and understandability
type BoxedError = Box<dyn std::error::Error + Send + Sync>;

/// Main function is asynchronous and makes heavy use of tokio concurrent tasks.
///
/// This function initializes the application by performing the following steps:
/// 1. Parse command-line arguments and validate them.
/// 2. Set up the logger.
/// 3. Define the number of threads to use based on the arguments.
/// 4. Set up channels for inter-task communication.
/// 5. Spawn tasks for file and directory crawling and deletion.
/// 6. Await the completion of all tasks and print a summary report.
#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    // Parse command-line arguments and validate them.
    let args = initialize_arguments()?;
    // Initialize the logger.
    let logger = initialize_logger(&args, args.buffer_size).await?;

    // Define the number of threads to use based on the arguments.
    let thread_info = define_threads(&args)?;

    // Log the start of the application.
    logger.log("--------------- Starting Application Run -----------------------", false, false, false).await;

    // Get the start time for calculating application runtime.
    let start = Instant::now();

    // Print initial information about the run.
    print_info(&args, &logger, &thread_info, thread_info.total_thread_count).await?;

    // Set up channels for inter-task communication.
    let (file_sender, dir_sender, file_receiver, dir_receiver) = setup_channels(&args);
    // Set up the deleter and shared state.
    let (deleter, total_directories, total_files_symlinks, total_crawling_ops, total_stat_ops, total_deletion_ops) = setup_deleter(&args);

    // Spawn deleter tasks for files and directories.
    let deleter_handle_files = spawn_deleter_task(
        &deleter, Arc::clone(&file_receiver), Arc::clone(&logger), args.verbose.clone(),
        Arc::clone(&total_deletion_ops), Arc::clone(&total_directories), thread_info.clone(), true
    );

    let deleter_handle_dirs = spawn_deleter_task(
        &deleter, Arc::clone(&dir_receiver), Arc::clone(&logger), args.verbose.clone(),
        Arc::clone(&total_deletion_ops), Arc::clone(&total_directories), thread_info.clone(), false
    );

    // Initialize the crawler.
    let crawler = Crawler::new(
        Arc::clone(&logger), file_sender.clone(), dir_sender.clone(), Arc::clone(&total_files_symlinks),
        Arc::clone(&total_directories), Arc::clone(&total_crawling_ops), Arc::clone(&total_stat_ops), args.verbose
    );

    // Run crawler tasks for files and directories.
    let crawler_handle_files = tokio::spawn(crawler.clone().run_crawlers_files(args.paths.clone()));
    let crawler_handle_dirs = tokio::spawn(crawler.run_crawlers_dirs(args.paths.clone()));

    // Use the join! macro to run crawler and deleter tasks concurrently, then wait for all
    // of them to complete. If any errors happen, log them and continue working.
    tokio::join!(
    async {
        if let Err(e) = crawler_handle_files.await {
            logger.log(&format!("Crawler error: {:?}", e), true, false, false).await;
        }
        drop(file_sender);
    },
    async {
        if let Err(e) = crawler_handle_dirs.await {
            logger.log(&format!("Crawler error: {:?}", e), true, false, false).await;
        }
        drop(dir_sender);
    },
    async {
        if let Err(e) = deleter_handle_files.await {
            logger.log(&format!("Deletion error: {:?}", e), true, false, false).await;
        }
    },
    async {
        if let Err(e) = deleter_handle_dirs.await {
            logger.log(&format!("Deletion error: {:?}", e), true, false, false).await;
        }
    }
);
    // Print the final summary and report.
    finalize(&deleter, &logger, start, total_directories, total_files_symlinks, total_crawling_ops, total_stat_ops, total_deletion_ops).await;

    Ok(())
}

/// Initializes the logger.
///
/// # Arguments
///
/// * `args` - A reference to the parsed command-line arguments.
///
/// # Returns
///
/// * `Result<Arc<Logger>, BoxedError>` - Ok with `Logger` if successful.
async fn initialize_logger(args: &Args, buffer_size: usize) -> Result<Arc<Logger>, BoxedError> {
    Ok(Logger::new(args.resolve_logfile_name(), args.verbose, buffer_size))
}

/// Sets up the deleter and shared state.
///
/// # Arguments
///
/// * `args` - A reference to the parsed command-line arguments.
///
/// # Returns
///
/// * `(Arc<Mutex<Deleter>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>)`
fn setup_deleter(args: &Args) -> (Arc<Mutex<Deleter>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>, Arc<Mutex<usize>>) {
    let deleter = Arc::new(Mutex::new(Deleter::new(args.dry_run)));
    let total_directories = Arc::new(Mutex::new(0));
    let total_files_symlinks = Arc::new(Mutex::new(0));
    let total_crawling_ops = Arc::new(Mutex::new(0));
    let total_stat_ops = Arc::new(Mutex::new(0));
    let total_deletion_ops = Arc::new(Mutex::new(0));
    (deleter, total_directories, total_files_symlinks, total_crawling_ops, total_stat_ops, total_deletion_ops)
}

/// Spawns a deleter task.
///
/// # Arguments
///
/// * `deleter` - A reference to the `Arc<Mutex<Deleter>>`.
/// * `receiver` - A reference to the `Arc<Mutex<mpsc::Receiver<PathBuf>>>`.
/// * `logger` - A reference to the `Arc<Logger`>.
/// * `verbose` - A boolean indicating whether to enable verbose logging.
/// * `total_deletion_ops` - A reference to the `Arc<Mutex<usize>>`.
/// * `total_directories` - A reference to the `Arc<Mutex<usize>>`.
/// * `thread_info` - A reference to the ThreadInfo struct.
/// * `_is_file` - A boolean indicating whether the task is for files or directories (unused).
///
/// # Returns
///
/// * `tokio::task::JoinHandle<Result<(), BoxedError>>>`
fn spawn_deleter_task(
    deleter: &Arc<Mutex<Deleter>>, receiver: Arc<Mutex<mpsc::Receiver<PathBuf>>>, logger: Arc<Logger>,
    verbose: bool, total_deletion_ops: Arc<Mutex<usize>>, total_directories: Arc<Mutex<usize>>,
    thread_info: ThreadInfo, _is_file: bool
) -> tokio::task::JoinHandle<Result<(), BoxedError>> {
    let deleter_clone = Arc::clone(deleter);
    let receiver_clone = Arc::clone(&receiver);
    let logger_clone = Arc::clone(&logger);
    let total_deletion_ops_clone = Arc::clone(&total_deletion_ops);
    let total_directories_clone = Arc::clone(&total_directories);
    let thread_info_clone = thread_info.clone();

    tokio::spawn(async move {
        deleter_clone.lock().await.delete_all(
            receiver_clone, thread_info_clone.total_thread_count, logger_clone, verbose,
            total_deletion_ops_clone, total_directories_clone
        ).await
    })
}
