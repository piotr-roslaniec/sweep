use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use std::fs;
use std::sync::{Arc, Mutex};
use crate::settings::Settings;
use super::{Plugin, FeaturePlugin, PluginError, ScanResult, CleanupReport, RiskLevel};
use super::filter::{SmartFilter, FileType, GitFileStatus};
use walkdir::{WalkDir, DirEntry};
use rayon::prelude::*;
use crossbeam::channel::unbounded;

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

/// Enhanced large file detection plugin with smart filtering
#[derive(Debug)]
pub struct LargeFilePluginEnhanced {
    size_threshold_bytes: u64,
    older_than_days: Option<u64>,
    include_git_tracked: bool,
    filter: Arc<Mutex<SmartFilter>>,
}

impl LargeFilePluginEnhanced {
    /// Create a new large file plugin with enhanced filtering
    pub fn new() -> Self {
        LargeFilePluginEnhanced {
            size_threshold_bytes: 100 * 1024 * 1024, // 100MB default
            older_than_days: None,
            include_git_tracked: false,
            filter: Arc::new(Mutex::new(SmartFilter::new())),
        }
    }

    /// Initialize git repositories and gitignore caches for a path
    fn initialize_filters(&self, root: &Path) -> Result<(), PluginError> {
        let mut filter = self.filter.lock().map_err(|e|
            PluginError::Configuration(format!("Failed to lock filter: {}", e)))?;

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

    /// Check if a file should be included based on age filter
    fn should_include_by_age(&self, metadata: &fs::Metadata) -> bool {
        match self.older_than_days {
            None => true,
            Some(days) => {
                match metadata.accessed() {
                    Ok(accessed) => {
                        match SystemTime::now().duration_since(accessed) {
                            Ok(age) => age > Duration::from_secs(days * 24 * 60 * 60),
                            Err(_) => true,
                        }
                    },
                    Err(_) => true,
                }
            }
        }
    }

    /// Process a single directory entry with enhanced filtering
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

    /// Scan directory in parallel with caching
    fn scan_parallel(&self, root: &Path) -> Result<Vec<LargeFile>, PluginError> {
        // Initialize filters with git repo and gitignore discovery
        self.initialize_filters(root)?;

        let (tx, rx) = unbounded();

        // Clone Arc for parallel processing
        let filter_arc = Arc::clone(&self.filter);
        let size_threshold = self.size_threshold_bytes;
        let older_than_days = self.older_than_days;
        let include_git_tracked = self.include_git_tracked;

        // Collect entries first
        let entries: Vec<_> = WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        // Process entries in parallel
        entries.par_iter().for_each_with(tx, |tx, entry| {
            // Skip directories
            if !entry.file_type().is_file() {
                return;
            }

            // Get metadata
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => return,
            };

            // Quick size check
            if metadata.len() < size_threshold {
                return;
            }

            // Create a temporary plugin instance for this thread
            let plugin = LargeFilePluginEnhanced {
                size_threshold_bytes: size_threshold,
                older_than_days,
                include_git_tracked,
                filter: Arc::clone(&filter_arc),
            };

            if let Some(large_file) = plugin.process_entry(entry.clone()) {
                let _ = tx.send(large_file);
            }
        });

        // Collect results
        let mut results = Vec::new();
        while let Ok(file) = rx.try_recv() {
            results.push(file);
        }

        // Sort by risk level (critical first) then by size (largest first)
        results.sort_by(|a, b| {
            match b.risk_level.cmp(&a.risk_level) {
                std::cmp::Ordering::Equal => b.size.cmp(&a.size),
                other => other,
            }
        });

        Ok(results)
    }
}

impl Plugin for LargeFilePluginEnhanced {
    fn name(&self) -> &str {
        "large-files-enhanced"
    }

    fn version(&self) -> &str {
        "2.0.0"
    }

    fn is_enabled(&self, settings: &Settings) -> bool {
        settings.enable_large_files
    }

    fn configure(&mut self, settings: &Settings) -> Result<(), PluginError> {
        self.size_threshold_bytes = super::utils::parse_size_string(&settings.size_threshold)?;
        self.older_than_days = settings.older_than_days;
        self.include_git_tracked = settings.include_git_tracked;
        Ok(())
    }

    fn apply_age_filter(&mut self, days: u64) -> Result<(), PluginError> {
        self.older_than_days = Some(days);
        Ok(())
    }
}

impl FeaturePlugin for LargeFilePluginEnhanced {
    fn scan(&self, path: &Path) -> Result<Vec<ScanResult>, PluginError> {
        if !path.exists() {
            return Err(PluginError::Scan(format!("Path does not exist: {:?}", path)));
        }

        // Perform enhanced parallel scan
        let large_files = self.scan_parallel(path)?;

        // Convert to ScanResult with detailed information
        let results: Vec<ScanResult> = large_files.into_iter().map(|file| {
            let size_str = super::utils::format_size(file.size);
            let age_days = if let Ok(modified) = SystemTime::now().duration_since(file.last_modified) {
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
        }).collect();

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
    use tempdir::TempDir;
    use std::fs::File;
    use std::io::Write;
    use git2::Repository;

    #[test]
    fn test_enhanced_plugin_creation() {
        let plugin = LargeFilePluginEnhanced::new();
        assert_eq!(plugin.name(), "large-files-enhanced");
        assert_eq!(plugin.version(), "2.0.0");
    }

    #[test]
    fn test_git_integration() {
        let temp_dir = TempDir::new("git_enhanced_test").unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create a large file
        let file_path = temp_dir.path().join("large.dat");
        let mut file = File::create(&file_path).unwrap();
        let buffer = vec![0u8; 150 * 1024 * 1024]; // 150MB
        file.write_all(&buffer).unwrap();

        // Add to git
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("large.dat")).unwrap();
        index.write().unwrap();

        let plugin = LargeFilePluginEnhanced::new();
        let results = plugin.scan(temp_dir.path()).unwrap();

        // File should be found but marked as critical risk (git tracked)
        assert!(!results.is_empty());
        assert_eq!(results[0].risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_filter_initialization() {
        let temp_dir = TempDir::new("filter_test").unwrap();

        // Create .gitignore
        let gitignore_path = temp_dir.path().join(".gitignore");
        std::fs::write(&gitignore_path, "*.log\n").unwrap();

        let plugin = LargeFilePluginEnhanced::new();
        assert!(plugin.initialize_filters(temp_dir.path()).is_ok());
    }
}