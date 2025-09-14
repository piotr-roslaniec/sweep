use super::filter::{FileType, GitFileStatus, SmartFilter};
use super::progress::ScanProgress;
use super::{CleanupReport, FeaturePlugin, Plugin, PluginError, RiskLevel, ScanResult};
use crate::settings::Settings;
use crossbeam::channel::unbounded;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use walkdir::{DirEntry, WalkDir};

/// File information for large file detection
#[derive(Debug, Clone)]
pub struct LargeFile {
    pub path: PathBuf,
    pub size: u64,
    pub last_modified: SystemTime,
    pub last_accessed: SystemTime,
    pub risk_level: RiskLevel,
    pub file_type: FileType,
    pub git_status: GitFileStatus,
}

/// Large file detection plugin with smart filtering
#[derive(Debug)]
pub struct LargeFilePlugin {
    size_threshold_bytes: u64,
    older_than_days: Option<u64>,
    include_git_tracked: bool,
    filter: Arc<Mutex<SmartFilter>>,
}

impl LargeFilePlugin {
    /// Create a new large file plugin with default settings
    pub fn new() -> Self {
        LargeFilePlugin {
            size_threshold_bytes: 100 * 1024 * 1024, // 100MB default
            older_than_days: None,
            include_git_tracked: false,
            filter: Arc::new(Mutex::new(SmartFilter::new())),
        }
    }

    /// Check if a file should be included based on age filter
    fn should_include_by_age(&self, metadata: &fs::Metadata) -> bool {
        match self.older_than_days {
            None => true,
            Some(days) => {
                match metadata.accessed() {
                    Ok(accessed) => {
                        match SystemTime::now().duration_since(accessed) {
                            Ok(age) => age > Duration::from_secs(days * 24 * 60 * 60),
                            Err(_) => true, // If we can't determine age, include it
                        }
                    }
                    Err(_) => true, // If we can't get access time, include it
                }
            }
        }
    }

    /// Process a single directory entry
    fn process_entry(&self, entry: DirEntry) -> Option<LargeFile> {
        // Skip directories and symlinks
        let file_type = entry.file_type();
        if !file_type.is_file() {
            return None;
        }

        // Get metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => return None,
        };

        // Check size threshold
        let size = metadata.len();
        if size < self.size_threshold_bytes {
            return None;
        }

        // Check age filter
        if !self.should_include_by_age(&metadata) {
            return None;
        }

        // Get timestamps
        let last_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let last_accessed = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);

        // Use smart filter for enhanced analysis
        let filter = self.filter.lock().ok()?;
        let path = entry.path();
        let file_type = filter.detect_file_type(&path);
        let git_status = filter.get_git_status(&path);
        let risk_level = filter.calculate_risk_level(&path, &metadata, self.include_git_tracked);

        // Skip critical risk files unless explicitly included
        if risk_level == RiskLevel::Critical && !self.include_git_tracked {
            return None;
        }

        Some(LargeFile {
            path: path.to_path_buf(),
            size,
            last_modified,
            last_accessed,
            risk_level,
            file_type,
            git_status,
        })
    }

    /// Initialize git repositories and gitignore caches for a path
    fn initialize_filters(&self, root: &Path) -> Result<(), PluginError> {
        let mut filter = self
            .filter
            .lock()
            .map_err(|e| PluginError::Configuration(format!("Failed to lock filter: {}", e)))?;

        // Discover git repositories
        filter.discover_git_repos(root)?;

        // Load gitignore files
        for entry in WalkDir::new(root)
            .max_depth(5) // Limit depth for performance
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == ".gitignore" {
                if let Some(parent) = entry.path().parent() {
                    let _ = filter.load_gitignore(parent);
                }
            }
        }

        Ok(())
    }

    /// Scan directory in parallel
    fn scan_parallel(&self, root: &Path) -> Result<Vec<LargeFile>, PluginError> {
        // Initialize filters with git repo and gitignore discovery
        self.initialize_filters(root)?;

        let (tx, rx) = unbounded();

        // Clone Arc for parallel processing
        let filter_arc = Arc::clone(&self.filter);
        let size_threshold = self.size_threshold_bytes;
        let older_than_days = self.older_than_days;
        let include_git_tracked = self.include_git_tracked;

        // Create a plugin instance for the parallel context
        let plugin_for_scan = LargeFilePlugin {
            size_threshold_bytes: size_threshold,
            older_than_days,
            include_git_tracked,
            filter: filter_arc,
        };

        // Collect entries first to enable parallel processing
        let entries: Vec<_> = WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        // Create progress bar
        let progress = Arc::new(ScanProgress::new(entries.len() as u64));
        let progress_clone = Arc::clone(&progress);

        // Process entries in parallel
        entries.par_iter().for_each_with(tx, |tx, entry| {
            // Update progress
            progress_clone.update(entry.path());

            if let Some(large_file) = plugin_for_scan.process_entry(entry.clone()) {
                progress_clone.found_file();
                let _ = tx.send(large_file);
            }
        });

        // Collect results
        let mut results = Vec::new();
        while let Ok(file) = rx.try_recv() {
            results.push(file);
        }

        // Finish progress bar
        progress.finish();

        // Sort by size (largest first)
        results.sort_by(|a, b| b.size.cmp(&a.size));

        Ok(results)
    }
}

impl Plugin for LargeFilePlugin {
    fn name(&self) -> &str {
        "large-files"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn is_enabled(&self, settings: &Settings) -> bool {
        settings.enable_large_files
    }

    fn configure(&mut self, settings: &Settings) -> Result<(), PluginError> {
        // Parse size threshold
        self.size_threshold_bytes = super::utils::parse_size_string(&settings.size_threshold)?;

        // Set age filter if provided
        self.older_than_days = settings.older_than_days;

        // Set git tracking preference
        self.include_git_tracked = settings.include_git_tracked;

        Ok(())
    }

    fn apply_age_filter(&mut self, days: u64) -> Result<(), PluginError> {
        self.older_than_days = Some(days);
        Ok(())
    }
}

impl FeaturePlugin for LargeFilePlugin {
    fn scan(&self, path: &Path) -> Result<Vec<ScanResult>, PluginError> {
        // Check if path exists
        if !path.exists() {
            return Err(PluginError::Scan(format!(
                "Path does not exist: {:?}",
                path
            )));
        }

        // Perform parallel scan
        let large_files = self.scan_parallel(path)?;

        // Convert to ScanResult with enhanced information
        let results: Vec<ScanResult> = large_files
            .into_iter()
            .map(|file| {
                let size_str = super::utils::format_size(file.size);
                let age_days =
                    if let Ok(modified) = SystemTime::now().duration_since(file.last_modified) {
                        modified.as_secs() / (24 * 60 * 60)
                    } else {
                        0
                    };

                let type_str = format!("{:?}", file.file_type);
                let git_str = format!("{:?}", file.git_status);

                ScanResult {
                    path: file.path,
                    size: file.size,
                    description: format!(
                        "{} | {} days old | Type: {} | Git: {}",
                        size_str, age_days, type_str, git_str
                    ),
                    risk_level: file.risk_level,
                }
            })
            .collect();

        Ok(results)
    }

    fn interactive_select(&self, results: Vec<ScanResult>) -> Result<Vec<ScanResult>, PluginError> {
        if results.is_empty() {
            return Ok(results);
        }

        // Use the interactive UI for selection
        let mut selector = super::ui::InteractiveSelector::new(results);
        match selector.run() {
            Ok(selected) => Ok(selected),
            Err(e) => Err(PluginError::Configuration(format!("UI error: {}", e))),
        }
    }

    fn clean(&self, _selected: Vec<ScanResult>) -> Result<CleanupReport, PluginError> {
        // TODO: Implement cleanup logic
        // This is a placeholder implementation
        Ok(CleanupReport {
            items_cleaned: 0,
            space_freed: 0,
            errors: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_file_plugin_creation() {
        let plugin = LargeFilePlugin::new();
        assert_eq!(plugin.name(), "large-files");
        assert_eq!(plugin.version(), "1.0.0");
        assert_eq!(plugin.size_threshold_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn test_large_file_plugin_configuration() {
        let mut plugin = LargeFilePlugin::new();

        let settings = Settings {
            paths: vec![],
            all: false,
            ignore: None,
            force: false,
            enable_large_files: true,
            enable_python: false,
            enable_java: false,
            enable_javascript: false,
            enable_rust: false,
            older_than_days: Some(30),
            size_threshold: "500MB".to_string(),
            include_git_tracked: true,
        };

        assert!(plugin.is_enabled(&settings));
        assert!(plugin.configure(&settings).is_ok());

        assert_eq!(plugin.size_threshold_bytes, 500 * 1024 * 1024);
        assert_eq!(plugin.older_than_days, Some(30));
        assert!(plugin.include_git_tracked);
    }

    #[test]
    fn test_age_filter() {
        let mut plugin = LargeFilePlugin::new();
        assert!(plugin.apply_age_filter(60).is_ok());
        assert_eq!(plugin.older_than_days, Some(60));
    }

    #[test]
    fn test_filter_integration() {
        let plugin = LargeFilePlugin::new();

        // Test that filter is initialized
        let filter = plugin.filter.lock().unwrap();
        assert!(filter.is_protected(Path::new(".env")));
        assert!(filter.is_test_data(Path::new("test-data.json")));
    }
}

// Include scanner tests module
#[cfg(test)]
#[path = "scanner_tests.rs"]
mod scanner_tests;
