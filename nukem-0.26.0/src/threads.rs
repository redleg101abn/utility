//! This module computes the thread count used by crawler and deleter workers

use crate::args::Args;
use num_cpus;

/// Holds information about the number of CPU cores and total thread count.
#[derive(Clone)]
pub struct ThreadInfo {
    pub core_count: usize,
    pub total_thread_count: usize,
}

impl ThreadInfo {
    // Threads per CPU. Set this low for testing but higher for better performance.
    const DEFAULT_THREAD_RATIO: usize = 10;

    /// Determines the number of threads to use.
    ///
    /// There are two methods:
    /// 1. User specifies number of threads through the command-line '-t' option.
    /// 2. System automatically computes number of threads to use, based on
    ///    number of CPU cores in the system multiplied by the DEFAULT_THREAD_RATIO constant.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the command-line arguments.
    ///
    /// # Returns
    ///
    /// * `Result<ThreadInfo, String>` - Ok with ThreadInfo if successful.
    pub fn compute_thread_count(args: &Args) -> Result<ThreadInfo, String> {
        let core_count = num_cpus::get();
        // If the user specified '-t' then use that value. Otherwise, automatically
        // compute by multiplying the core_count by the DEFAULT_THREAD_RATIO constant.
        let total_thread_count = match args.threads {
            Some(t) => t,
            None => core_count * Self::DEFAULT_THREAD_RATIO,
        };

        Ok(ThreadInfo {
            core_count,
            total_thread_count,
        })
    }
}
