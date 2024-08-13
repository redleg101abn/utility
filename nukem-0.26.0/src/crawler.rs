//! The `crawler` module provides functionality to crawl filesystem paths and collect metadata
//! on all objects including files, directories, and symlinks. Symlinks are not followed.

use std::sync::Arc;
use std::path::PathBuf;
use tokio::fs as async_fs;
use tokio::sync::{mpsc::Sender, Mutex};
use tokio::task;
use glob::glob;
use crate::logger::Logger;
use futures::future::BoxFuture;

/// This structure represents the file and directory crawler.
#[derive(Clone)]
pub struct Crawler {
    logger: Arc<Logger>,
    file_sender: Sender<PathBuf>,
    dir_sender: Sender<PathBuf>,
    total_files_symlinks: Arc<Mutex<usize>>,
    total_directories: Arc<Mutex<usize>>,
    total_crawling_ops: Arc<Mutex<usize>>,
    total_stat_ops: Arc<Mutex<usize>>,
    verbose: bool,
}

impl Crawler {
    /// Creates a new instance of the Crawler.
    ///
    /// # Arguments
    ///
    /// * `logger` - An instance of the Logger.
    /// * `file_sender` - A channel sender for file paths.
    /// * `dir_sender` - A channel sender for directory paths.
    /// * `total_files_symlinks` - A shared counter for the total number of files and symlinks.
    /// * `total_directories` - A shared counter for the total number of directories.
    /// * `total_crawling_ops` - A shared counter for the total number of crawling operations.
    /// * `total_stat_ops` - A shared counter for the total number of stat operations.
    /// * `verbose` - A boolean indicating whether to enable verbose logging.
    pub fn new(
        logger: Arc<Logger>,
        file_sender: Sender<PathBuf>,
        dir_sender: Sender<PathBuf>,
        total_files_symlinks: Arc<Mutex<usize>>,
        total_directories: Arc<Mutex<usize>>,
        total_crawling_ops: Arc<Mutex<usize>>,
        total_stat_ops: Arc<Mutex<usize>>,
        verbose: bool,
    ) -> Self {
        Self {
            logger,
            file_sender,
            dir_sender,
            total_files_symlinks,
            total_directories,
            total_crawling_ops,
            total_stat_ops,
            verbose,
        }
    }

    /// Runs crawlers to collect metadata on files.
    ///
    /// # Arguments
    ///
    /// * `patterns` - A vector of file path patterns to crawl.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>`
    ///   - Ok if successful, Err otherwise.
    pub async fn run_crawlers_files(
        self,
        patterns: Vec<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.run_crawlers(patterns, true).await
    }

    /// Runs crawlers to collect metadata on directories.
    ///
    /// # Arguments
    ///
    /// * `patterns` - A vector of directory path patterns to crawl.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>`
    ///   - Ok if successful, Err otherwise.
    pub async fn run_crawlers_dirs(
        self,
        patterns: Vec<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.run_crawlers(patterns, false).await
    }

    /// Internal function to run crawlers.
    ///
    /// # Arguments
    ///
    /// * `patterns` - A vector of path patterns to crawl.
    /// * `is_file` - A boolean indicating whether to crawl files.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>`
    ///   - Ok if successful, Err otherwise.
    async fn run_crawlers(
        self,
        patterns: Vec<PathBuf>,
        is_file: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // make a tasks vector
        let mut tasks = Vec::new();
        for pattern in patterns {
            let paths = glob(pattern.to_str().unwrap()).expect("Failed to read glob pattern").filter_map(Result::ok);
            for path in paths {
                let logger = self.logger.clone();
                let sender = if is_file { self.file_sender.clone() } else { self.dir_sender.clone() };
                let total_files_symlinks = self.total_files_symlinks.clone();
                let total_directories = self.total_directories.clone();
                let total_crawling_ops = self.total_crawling_ops.clone();
                let total_stat_ops = self.total_stat_ops.clone();
                let verbose = self.verbose;
                tasks.push(task::spawn(async move {
                    if is_file {
                        Crawler::process_path(path, sender, logger, total_files_symlinks, total_crawling_ops, total_stat_ops, verbose, true).await
                    } else {
                        Crawler::process_path(path, sender, logger, total_directories, total_crawling_ops, total_stat_ops, verbose, false).await
                    }
                }));
            }
        }
        for task in tasks {
            task.await??;
        }
        Ok(())
    }

    /// Processes paths and sends them through the provided channel.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to process.
    /// * `sender` - A channel sender for paths.
    /// * `logger` - An instance of the Logger.
    /// * `counter` - A shared counter for the total number of objects.
    /// * `total_crawling_ops` - A shared counter for the total number of crawling operations.
    /// * `total_stat_ops` - A shared counter for the total number of stat operations.
    /// * `verbose` - A boolean indicating whether to enable verbose logging.
    /// * `is_file` - A boolean indicating whether to process files or directories.
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn std::error::Error + Send + Sync>>`
    ///   - Ok if successful, Err otherwise.
    fn process_path(
        path: PathBuf,
        sender: Sender<PathBuf>,
        logger: Arc<Logger>,
        counter: Arc<Mutex<usize>>,
        total_crawling_ops: Arc<Mutex<usize>>,
        total_stat_ops: Arc<Mutex<usize>>,
        verbose: bool,
        is_file: bool,
    ) -> BoxFuture<'static, Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async move {
            *total_crawling_ops.lock().await += 1;
            let metadata = async_fs::metadata(&path).await?;
            *total_stat_ops.lock().await += 1;

            if verbose {
                logger.log(&format!("Found object: {:?}", path), false, false, true).await;
            }

            if metadata.is_file() || metadata.file_type().is_symlink() {
                if is_file {
                    *counter.lock().await += 1;
                    sender.send(path).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                } else {
                    Ok(())
                }
            } else if metadata.is_dir() {
                if !is_file {
                    *counter.lock().await += 1;
                    let mut entries = async_fs::read_dir(&path).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        let entry_path = entry.path();
                        Crawler::process_path(entry_path, sender.clone(), logger.clone(), counter.clone(), total_crawling_ops.clone(), total_stat_ops.clone(), verbose, is_file).await?;
                    }
                    sender.send(path).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        })
    }
}
