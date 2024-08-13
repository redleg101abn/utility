//! The deleter module provides functionality to delete files and directories
//! based on the paths received from crawlers.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use std::path::PathBuf;
use tokio::task;
use tokio::fs;
use crate::logger::Logger;

/// The Deleter struct is responsible for deleting files and directories.
pub struct Deleter {
    pub failed_deletions: Arc<Mutex<u64>>,
    pub total_size: Arc<Mutex<u64>>,
    pub dry_run: bool,
}

impl Deleter {
    /// Creates a new Deleter instance.
    ///
    /// # Arguments
    ///
    /// * dry_run - A boolean indicating whether to perform a dry run.
    ///
    /// # Returns
    ///
    /// * 'Self' - A new instance of the Deleter.
    pub fn new(dry_run: bool) -> Self {
        Self {
            failed_deletions: Arc::new(Mutex::new(0)),
            total_size: Arc::new(Mutex::new(0)),
            dry_run,
        }
    }

    /// Retrieves the total size of deleted files.
    ///
    /// # Returns
    ///
    /// * 'u64' - The total size of deleted files in bytes.
    pub async fn get_total_size(&self) -> u64 {
        *self.total_size.lock().await
    }

    /// Retrieves the total number of failed deletions.
    ///
    /// # Returns
    ///
    /// * 'u64' - The total number of failed deletions.
    pub async fn get_failed_deletions(&self) -> u64 {
        *self.failed_deletions.lock().await
    }

    /// Deletes all paths received through the channel.
    ///
    /// # Arguments
    ///
    /// * receiver - A receiver for paths to delete.
    /// * worker_tasks_count - The number of worker tasks to spawn.
    /// * logger - An instance of the Logger.
    /// * verbose - A boolean indicating whether to enable verbose logging.
    /// * total_deletion_ops - A shared counter for the total number of deletion operations.
    /// * total_directories - A shared counter for the total number of directories.
    ///
    /// # Returns
    ///
    /// * 'Result<(), Box<dyn std::error::Error + Send + Sync>>' - Ok if successful, Err otherwise.
    pub async fn delete_all(
        &self,
        receiver: Arc<Mutex<mpsc::Receiver<PathBuf>>>,
        worker_tasks_count: usize,
        logger: Arc<Logger>,
        verbose: bool,
        total_deletion_ops: Arc<Mutex<usize>>,
        total_directories: Arc<Mutex<usize>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // vector to hold metadata from crawlers
        let mut handles = vec![];

        for i in 0..worker_tasks_count {
            let logger = logger.clone();
            let failed_deletions = self.failed_deletions.clone();
            let total_size = self.total_size.clone();
            let receiver = receiver.clone();
            let total_deletion_ops = total_deletion_ops.clone();
            let total_directories = total_directories.clone();
            let dry_run = self.dry_run;

            // push object to vector
            handles.push(task::spawn(async move {
                while let Some(path) = receiver.lock().await.recv().await {
                    if verbose {
                        logger.log(&format!("Worker {} picked up path: {:?}", i, &path), false, true, true).await;
                    }

                    if let Err(e) = Deleter::process_path(
                        &path,
                        logger.clone(),
                        verbose,
                        total_size.clone(),
                        failed_deletions.clone(),
                        total_deletion_ops.clone(),
                        total_directories.clone(),
                        dry_run,
                    ).await {
                        logger.log(&format!("[ERROR] Worker {} failed to process path {:?}: {:?}", i, &path, e), true, false, false).await;
                        // Increment failed_deletions count
                        *failed_deletions.lock().await += 1;
                    }
                }
                if verbose {
                    logger.log(&format!("Worker {} finished processing paths", i), false, true, true).await;
                }
            }));
        }

        for handle in handles {
            // Ensure all tasks are complete
            match handle.await {
                Ok(result) => result,
                Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            };
        }

        if verbose {
            logger.log("All workers finished", false, true, true).await;
        }

        Ok(())
    }

    /// Processes a path and deletes it if it's a file or recursively deletes if it's a directory.
    ///
    /// # Arguments
    ///
    /// * path - The path to process.
    /// * logger - An instance of the Logger.
    /// * verbose - A boolean indicating whether to enable verbose logging.
    /// * total_size - A shared counter for the total size of deleted files.
    /// * failed_deletions - A shared counter for the number of failed deletions.
    /// * total_deletion_ops - A shared counter for the total number of deletion operations.
    /// * total_directories - A shared counter for the total number of directories.
    /// * dry_run - A boolean indicating whether to perform a dry run.
    ///
    /// # Returns
    ///
    /// * 'Result<(), Box<dyn std::error::Error + Send + Sync>>' - Ok if successful, Err otherwise.
    async fn process_path(
        path: &PathBuf,
        logger: Arc<Logger>,
        verbose: bool,
        total_size: Arc<Mutex<u64>>,
        failed_deletions: Arc<Mutex<u64>>,
        total_deletion_ops: Arc<Mutex<usize>>,
        total_directories: Arc<Mutex<usize>>,
        dry_run: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metadata = fs::metadata(path).await?;
        if metadata.is_file() || metadata.file_type().is_symlink() {
            if !dry_run {
                if let Err(e) = fs::remove_file(path).await {
                    *failed_deletions.lock().await += 1;
                    return Err(Box::new(e));
                }
                *total_deletion_ops.lock().await += 1;
                *total_size.lock().await += metadata.len();
            }
            if verbose {
                logger.log(&format!("Deleted file/symlink: {:?}", path), false, true, true).await;
            }
        } else if metadata.is_dir() {
            if !dry_run {
                if let Err(e) = fs::remove_dir_all(path).await {
                    *failed_deletions.lock().await += 1;
                    return Err(Box::new(e));
                }
                *total_deletion_ops.lock().await += 1;
                *total_directories.lock().await += 1;
            }
            if verbose {
                logger.log(&format!("Deleted directory: {:?}", path), false, true, true).await;
            }
        }
        Ok(())
    }

    /// Shuts down the deleter, performing any necessary cleanup.
    pub async fn shutdown(&self) {
        // Perform any necessary cleanup here.
    }
}
