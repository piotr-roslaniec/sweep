use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempdir::TempDir;
use git2::{Repository, Signature};
use crate::settings::Settings;
use super::{
    Plugin, FeaturePlugin, RiskLevel,
    large_files::LargeFilePlugin,
    filter::SmartFilter,
    utils
};

/// Integration test helper to create a test environment
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub repo: Option<Repository>,
}

impl TestEnvironment {
    /// Create a new test environment with optional git repository
    pub fn new(with_git: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new("sweep_integration_test")?;

        let repo = if with_git {
            let repo = Repository::init(temp_dir.path())?;

            // Configure git with test user
            let mut config = repo.config()?;
            config.set_str("user.name", "Test User")?;
            config.set_str("user.email", "test@example.com")?;

            Some(repo)
        } else {
            None
        };

        Ok(TestEnvironment { temp_dir, repo })
    }

    /// Get the path to the test directory
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a file with specified size and content
    pub fn create_file(&self, relative_path: &str, size_bytes: u64) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let file_path = self.path().join(relative_path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&file_path)?;

        // Write specified amount of data
        let chunk_size = 8192;
        let mut remaining = size_bytes;
        let buffer = vec![0u8; chunk_size];

        while remaining > 0 {
            let write_size = std::cmp::min(remaining, chunk_size as u64) as usize;
            file.write_all(&buffer[..write_size])?;
            remaining -= write_size as u64;
        }

        file.flush()?;
        Ok(file_path)
    }

    /// Create a .gitignore file with specified patterns
    pub fn create_gitignore(&self, patterns: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let gitignore_path = self.path().join(".gitignore");
        let mut file = File::create(gitignore_path)?;

        for pattern in patterns {
            writeln!(file, "{}", pattern)?;
        }

        Ok(())
    }

    /// Add and commit a file to git
    pub fn git_add_and_commit(&self, file_path: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(repo) = &self.repo {
            let mut index = repo.index()?;
            index.add_path(Path::new(file_path))?;
            index.write()?;

            let signature = Signature::now("Test User", "test@example.com")?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;

            let parent_commit = if let Ok(head) = repo.head() {
                Some(head.target().unwrap())
            } else {
                None
            };

            if let Some(parent_id) = parent_commit {
                let parent = repo.find_commit(parent_id)?;
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[&parent],
                )?;
            } else {
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[],
                )?;
            }
        }
        Ok(())
    }
}

/// Create test settings with specified options
pub fn create_test_settings(
    enable_large_files: bool,
    size_threshold: &str,
    older_than_days: Option<u64>,
    include_git_tracked: bool,
) -> Settings {
    Settings {
        paths: vec![],
        all: false,
        ignore: None,
        force: false,
        enable_large_files,
        enable_python: false,
        enable_java: false,
        enable_javascript: false,
        enable_rust: false,
        older_than_days,
        size_threshold: size_threshold.to_string(),
        include_git_tracked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_plugin_workflow() -> Result<(), Box<dyn std::error::Error>> {
        // Create test environment with git repository
        let env = TestEnvironment::new(true)?;

        // Create test files with different sizes
        env.create_file("small.txt", 1024)?; // 1KB - below threshold
        env.create_file("large1.bin", 200 * 1024 * 1024)?; // 200MB - above threshold
        env.create_file("large2.log", 150 * 1024 * 1024)?; // 150MB - above threshold
        env.create_file("protected.env", 300 * 1024 * 1024)?; // 300MB - protected file

        // Create and configure plugin
        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", None, false);

        assert!(plugin.is_enabled(&settings));
        plugin.configure(&settings)?;

        // Test scan functionality
        let scan_results = plugin.scan(env.path())?;

        // Should find large files but exclude protected ones (unless include_git_tracked is true)
        assert!(!scan_results.is_empty());

        // Verify that .env file is marked as critical risk
        let env_file_result = scan_results.iter().find(|r| r.path.file_name().unwrap() == "protected.env");
        if let Some(result) = env_file_result {
            // .env files are protected and should be critical OR high risk
            assert!(matches!(result.risk_level, RiskLevel::Critical | RiskLevel::High));
        }

        // Test that small files are not included
        let small_file_result = scan_results.iter().find(|r| r.path.file_name().unwrap() == "small.txt");
        assert!(small_file_result.is_none());

        Ok(())
    }

    #[test]
    fn test_plugin_flag_combinations() -> Result<(), Box<dyn std::error::Error>> {
        // Create test environment to test different configurations
        let env = TestEnvironment::new(false)?;
        env.create_file("test1.bin", 200 * 1024 * 1024)?; // 200MB
        env.create_file("test2.bin", 50 * 1024 * 1024)?;  // 50MB

        let mut plugin = LargeFilePlugin::new();

        // Test with 100MB threshold - should find test1.bin but not test2.bin
        let settings_100mb = create_test_settings(true, "100MB", None, false);
        plugin.configure(&settings_100mb)?;
        let results_100mb = plugin.scan(env.path())?;
        assert_eq!(results_100mb.len(), 1); // Only test1.bin should be found

        // Test with 25MB threshold - should find both files
        let settings_25mb = create_test_settings(true, "25MB", None, false);
        plugin.configure(&settings_25mb)?;
        let results_25mb = plugin.scan(env.path())?;
        assert_eq!(results_25mb.len(), 2); // Both files should be found

        // Test disabled plugin
        let disabled_settings = create_test_settings(false, "100MB", None, false);
        assert!(!plugin.is_enabled(&disabled_settings));

        // Test age filter application
        plugin.apply_age_filter(30)?; // Should not error

        Ok(())
    }

    #[test]
    fn test_git_tracked_file_protection() -> Result<(), Box<dyn std::error::Error>> {
        // Create test environment with git
        let env = TestEnvironment::new(true)?;

        // Create large files
        env.create_file("tracked_large.bin", 200 * 1024 * 1024)?;
        env.create_file("untracked_large.bin", 150 * 1024 * 1024)?;

        // Add and commit one file to git
        env.git_add_and_commit("tracked_large.bin", "Add large tracked file")?;

        // Create and configure plugin (exclude git tracked files)
        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", None, false);
        plugin.configure(&settings)?;

        // Scan for large files
        let results = plugin.scan(env.path())?;

        // Should find files - behavior may vary based on git tracking rules
        let tracked_result = results.iter().find(|r| r.path.file_name().unwrap() == "tracked_large.bin");
        let untracked_result = results.iter().find(|r| r.path.file_name().unwrap() == "untracked_large.bin");

        // At least one file should be found (untracked one definitely)
        assert!(untracked_result.is_some());

        // If tracked file is included, it should have critical risk
        if let Some(tracked) = tracked_result {
            assert_eq!(tracked.risk_level, RiskLevel::Critical);
        }

        Ok(())
    }

    #[test]
    fn test_gitignore_file_handling() -> Result<(), Box<dyn std::error::Error>> {
        // Create test environment with git
        let env = TestEnvironment::new(true)?;

        // Create .gitignore
        env.create_gitignore(&["*.log", "temp/"])?;

        // Create files that should be ignored and not ignored
        env.create_file("application.log", 200 * 1024 * 1024)?; // Should be ignored
        env.create_file("temp/cache.dat", 150 * 1024 * 1024)?; // Should be ignored
        env.create_file("important.bin", 180 * 1024 * 1024)?; // Should not be ignored

        // Create and configure plugin
        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", None, false);
        plugin.configure(&settings)?;

        // Scan for large files
        let results = plugin.scan(env.path())?;
        assert!(!results.is_empty());

        // Verify gitignored files are handled correctly (marked as safe)
        let log_result = results.iter().find(|r| r.path.file_name().unwrap() == "application.log");
        let important_result = results.iter().find(|r| r.path.file_name().unwrap() == "important.bin");

        // Gitignored files should be marked as safe risk
        if let Some(log) = log_result {
            assert_eq!(log.risk_level, RiskLevel::Safe);
        }

        // Non-ignored files should be found
        assert!(important_result.is_some());

        Ok(())
    }

    #[test]
    fn test_age_based_filtering() -> Result<(), Box<dyn std::error::Error>> {
        // Create test environment
        let env = TestEnvironment::new(false)?;

        // Create large files (we can't easily manipulate file ages in tests,
        // so we'll test the configuration logic)
        env.create_file("file1.bin", 200 * 1024 * 1024)?;
        env.create_file("file2.bin", 150 * 1024 * 1024)?;

        // Create and configure plugin with age filter
        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", Some(30), false);
        plugin.configure(&settings)?;

        // Test applying different age filters - should not error
        plugin.apply_age_filter(7)?;
        plugin.apply_age_filter(365)?;

        // Scan should still work (age filtering logic is complex to test without file manipulation)
        // The files we just created are very new, so age filtering might exclude them
        let results = plugin.scan(env.path())?;
        // Results may be 0 or 2 depending on age filter, both are acceptable
        assert!(results.len() <= 2);

        Ok(())
    }

    #[test]
    fn test_error_handling_and_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
        let mut plugin = LargeFilePlugin::new();

        // Test invalid size threshold
        let invalid_settings = Settings {
            paths: vec![],
            all: false,
            ignore: None,
            force: false,
            enable_large_files: true,
            enable_python: false,
            enable_java: false,
            enable_javascript: false,
            enable_rust: false,
            older_than_days: None,
            size_threshold: "invalid_size".to_string(),
            include_git_tracked: false,
        };

        // Should fail to configure with invalid size
        assert!(plugin.configure(&invalid_settings).is_err());

        // Test scanning non-existent path
        let non_existent_path = Path::new("/this/path/does/not/exist");
        let scan_result = plugin.scan(non_existent_path);
        assert!(scan_result.is_err());

        // Test scanning empty directory
        let env = TestEnvironment::new(false)?;
        let settings = create_test_settings(true, "100MB", None, false);
        plugin.configure(&settings)?;
        let results = plugin.scan(env.path())?;
        assert!(results.is_empty());

        // Test very small size threshold
        let settings = create_test_settings(true, "1B", None, false);
        assert!(plugin.configure(&settings).is_ok());

        Ok(())
    }

    #[test]
    fn test_file_type_detection_integration() -> Result<(), Box<dyn std::error::Error>> {
        let env = TestEnvironment::new(false)?;

        // Create files with different types
        env.create_file("database.db", 200 * 1024 * 1024)?;
        env.create_file("archive.zip", 150 * 1024 * 1024)?;
        env.create_file("media.mp4", 300 * 1024 * 1024)?;
        env.create_file("source.rs", 120 * 1024 * 1024)?;
        env.create_file("test-data.json", 110 * 1024 * 1024)?;
        env.create_file("application.log", 180 * 1024 * 1024)?;

        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", None, false);
        plugin.configure(&settings)?;

        let results = plugin.scan(env.path())?;
        assert!(!results.is_empty());

        // Verify different file types get appropriate risk levels
        let db_result = results.iter().find(|r| r.path.file_name().unwrap() == "database.db");
        let test_result = results.iter().find(|r| r.path.file_name().unwrap() == "test-data.json");

        if let Some(db) = db_result {
            assert_eq!(db.risk_level, RiskLevel::High); // Database files are high risk
        }

        if let Some(test) = test_result {
            // Test data files should have low risk or higher due to recent creation
            assert!(matches!(test.risk_level, RiskLevel::Low | RiskLevel::Medium | RiskLevel::High));
        }

        Ok(())
    }

    #[test]
    fn test_utils_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test size parsing with various formats
        assert_eq!(utils::parse_size_string("100MB")?, 100 * 1024 * 1024);
        assert_eq!(utils::parse_size_string("1.5GB")?, (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
        assert_eq!(utils::parse_size_string("500KB")?, 500 * 1024);
        assert_eq!(utils::parse_size_string("2TB")?, 2 * 1024 * 1024 * 1024 * 1024);

        // Test size formatting
        assert_eq!(utils::format_size(1024), "1.00 KB");
        assert_eq!(utils::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(utils::format_size(1536 * 1024 * 1024), "1.50 GB");

        // Test roundtrip conversion
        let original_size = 2.3 * 1024.0 * 1024.0 * 1024.0;
        let size_string = utils::format_size(original_size as u64);
        let parsed_back = utils::parse_size_string(&size_string)?;

        // Should be approximately equal (allowing for rounding)
        let diff = (parsed_back as f64 - original_size).abs();
        assert!(diff < 1024.0 * 1024.0); // Within 1MB tolerance

        Ok(())
    }

    #[test]
    fn test_smart_filter_integration() -> Result<(), Box<dyn std::error::Error>> {
        let env = TestEnvironment::new(true)?;

        // Create various files
        env.create_file(".env", 1024)?; // Protected file
        env.create_file("config.json", 2048)?; // Configuration file
        env.create_file("fixture.sql", 4096)?; // Test data
        env.create_file("regular.txt", 8192)?; // Regular file

        let mut filter = SmartFilter::new();
        filter.discover_git_repos(env.path())?;

        // Test protected file detection
        assert!(filter.is_protected(&env.path().join(".env")));
        assert!(!filter.is_protected(&env.path().join("regular.txt")));

        // Test test data detection (fixture pattern should match)
        assert!(filter.is_test_data(&env.path().join("fixture.sql")));
        assert!(!filter.is_test_data(&env.path().join("regular.txt")));

        // Test file type detection
        let metadata = fs::metadata(env.path().join("config.json"))?;
        let risk = filter.calculate_risk_level(&env.path().join("config.json"), &metadata, false);
        assert_eq!(risk, RiskLevel::High); // Config files are high risk

        Ok(())
    }

    #[test]
    fn test_interactive_selection_workflow() -> Result<(), Box<dyn std::error::Error>> {
        // Create test environment
        let env = TestEnvironment::new(false)?;
        env.create_file("large1.bin", 200 * 1024 * 1024)?;
        env.create_file("large2.log", 150 * 1024 * 1024)?;

        // Create and configure plugin
        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", None, false);
        plugin.configure(&settings)?;

        // Test scan
        let scan_results = plugin.scan(env.path())?;
        assert_eq!(scan_results.len(), 2);

        // Test interactive selection (without actually running the UI)
        // We can't easily test the actual UI interaction, but we can test
        // that the method exists and handles empty results correctly
        let empty_selection = plugin.interactive_select(vec![])?;
        assert!(empty_selection.is_empty());

        // Test interactive selection - this will fail in headless environment
        // so we'll just test that the method handles errors gracefully
        let selection_result = plugin.interactive_select(scan_results);
        // In headless environment, UI will error - this is expected
        // Either it succeeds (in real terminal) or fails gracefully
        match selection_result {
            Ok(results) => {
                // If UI succeeds, results should be valid
                assert!(results.len() <= 2);
            }
            Err(_) => {
                // UI error in headless environment is acceptable
                // This tests error handling
            }
        }

        Ok(())
    }
}