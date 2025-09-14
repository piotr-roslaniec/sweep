/// Progress indicator for long-running operations
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Progress tracker for file scanning operations
pub struct ScanProgress {
    bar: ProgressBar,
    found_count: AtomicUsize,
    scanned_count: AtomicUsize,
}

impl ScanProgress {
    /// Create a new progress bar for scanning
    pub fn new(estimated_files: u64) -> Self {
        let bar = ProgressBar::new(estimated_files);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files | Found: {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("##-"),
        );

        bar.enable_steady_tick(Duration::from_millis(100));

        Self {
            bar,
            found_count: AtomicUsize::new(0),
            scanned_count: AtomicUsize::new(0),
        }
    }

    /// Update progress with current file being scanned
    pub fn update(&self, path: &Path) {
        let scanned = self.scanned_count.fetch_add(1, Ordering::SeqCst) + 1;
        let found = self.found_count.load(Ordering::SeqCst);

        self.bar.set_position(scanned as u64);
        self.bar.set_message(format!("{} large files", found));

        // Show current file being scanned in the prefix
        if let Some(file_name) = path.file_name() {
            self.bar
                .set_prefix(format!("Scanning: {}", file_name.to_string_lossy()));
        }
    }

    /// Increment the count of found large files
    pub fn found_file(&self) {
        self.found_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Finish the progress bar with a summary
    pub fn finish(&self) {
        let found = self.found_count.load(Ordering::SeqCst);
        let scanned = self.scanned_count.load(Ordering::SeqCst);

        self.bar.finish_with_message(format!(
            "Complete! Found {} large files in {} files scanned",
            found, scanned
        ));
    }

    /// Finish with an error message
    #[allow(dead_code)]
    pub fn finish_with_error(&self, error: &str) {
        self.bar.finish_with_message(format!("Error: {}", error));
    }
}

impl Drop for ScanProgress {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.finish();
        }
    }
}

/// Progress tracker for cleanup operations
#[allow(dead_code)]
pub struct CleanupProgress {
    bar: ProgressBar,
    space_freed: AtomicUsize,
}

impl CleanupProgress {
    /// Create a new progress bar for cleanup
    #[allow(dead_code)]
    pub fn new(total_files: u64) -> Self {
        let bar = ProgressBar::new(total_files);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.green/red} {pos}/{len} | Space freed: {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("##-"),
        );

        Self {
            bar,
            space_freed: AtomicUsize::new(0),
        }
    }

    /// Update progress when a file is deleted
    #[allow(dead_code)]
    pub fn file_deleted(&self, path: &Path, size: u64) {
        let freed = self.space_freed.fetch_add(size as usize, Ordering::SeqCst) + size as usize;

        self.bar.inc(1);
        self.bar.set_message(format_size(freed as u64));

        if let Some(file_name) = path.file_name() {
            self.bar
                .set_prefix(format!("Deleted: {}", file_name.to_string_lossy()));
        }
    }

    /// Mark cleanup as complete
    pub fn finish(&self) {
        let freed = self.space_freed.load(Ordering::SeqCst);
        self.bar
            .finish_with_message(format!("Complete! Freed {}", format_size(freed as u64)));
    }
}

impl Drop for CleanupProgress {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.finish();
        }
    }
}

/// Format bytes as human-readable size
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1536 * 1024), "1.50 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_scan_progress() {
        let progress = ScanProgress::new(100);
        let test_path = PathBuf::from("/test/file.txt");

        // Simulate scanning
        for i in 0..10 {
            progress.update(&test_path);
            if i % 3 == 0 {
                progress.found_file();
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(progress.scanned_count.load(Ordering::SeqCst), 10);
        assert_eq!(progress.found_count.load(Ordering::SeqCst), 4);

        progress.finish();
    }

    #[test]
    fn test_cleanup_progress() {
        let progress = CleanupProgress::new(5);
        let test_path = PathBuf::from("/test/large_file.dat");

        // Simulate cleanup
        progress.file_deleted(&test_path, 1024 * 1024 * 100); // 100MB
        progress.file_deleted(&test_path, 1024 * 1024 * 50); // 50MB

        assert_eq!(
            progress.space_freed.load(Ordering::SeqCst),
            1024 * 1024 * 150
        );

        progress.finish();
    }
}
