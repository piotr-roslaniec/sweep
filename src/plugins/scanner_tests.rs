/// Tests for the large file scanner functionality
#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::plugins::{FeaturePlugin, Plugin};
    use crate::settings::Settings;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::time::{Duration, SystemTime};
    use tempdir::TempDir;

    /// Helper to create a file with specified size
    fn create_file_with_size(path: &Path, size_mb: usize) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        for _ in 0..size_mb {
            file.write_all(&buffer)?;
        }
        Ok(())
    }

    /// Helper to create a test directory structure
    fn setup_test_directory() -> TempDir {
        let temp_dir = TempDir::new("sweep_test").unwrap();
        let base_path = temp_dir.path();

        // Create various files
        create_file_with_size(&base_path.join("large_file.dat"), 150).unwrap();
        create_file_with_size(&base_path.join("medium_file.dat"), 75).unwrap();
        create_file_with_size(&base_path.join("small_file.txt"), 10).unwrap();

        // Create nested directories with files
        let nested_dir = base_path.join("nested");
        fs::create_dir(&nested_dir).unwrap();
        create_file_with_size(&nested_dir.join("nested_large.bin"), 200).unwrap();
        create_file_with_size(&nested_dir.join("nested_small.txt"), 5).unwrap();

        // Create a .git directory to test git-tracked file detection
        let git_dir = base_path.join(".git");
        fs::create_dir(&git_dir).unwrap();

        temp_dir
    }

    #[test]
    fn test_scan_finds_large_files() {
        let temp_dir = setup_test_directory();
        let mut plugin = LargeFilePlugin::new();

        // Configure with 100MB threshold
        let settings = create_test_settings(true, "100MB", false, None);
        plugin.configure(&settings).unwrap();

        // Scan the directory
        let results = plugin.scan(temp_dir.path()).unwrap();

        // Should find 2 files over 100MB
        assert_eq!(results.len(), 2);

        // Check that large files were found
        let file_names: Vec<String> = results
            .iter()
            .map(|r| r.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"large_file.dat".to_string()));
        assert!(file_names.contains(&"nested_large.bin".to_string()));
    }

    #[test]
    fn test_scan_respects_size_threshold() {
        let temp_dir = setup_test_directory();
        let mut plugin = LargeFilePlugin::new();

        // Configure with 50MB threshold
        let settings = create_test_settings(true, "50MB", false, None);
        plugin.configure(&settings).unwrap();

        // Scan the directory
        let results = plugin.scan(temp_dir.path()).unwrap();

        // Should find 3 files over 50MB
        assert_eq!(results.len(), 3);

        let file_names: Vec<String> = results
            .iter()
            .map(|r| r.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(file_names.contains(&"large_file.dat".to_string()));
        assert!(file_names.contains(&"medium_file.dat".to_string()));
        assert!(file_names.contains(&"nested_large.bin".to_string()));
    }

    #[test]
    fn test_scan_excludes_small_files() {
        let temp_dir = setup_test_directory();
        let mut plugin = LargeFilePlugin::new();

        // Configure with 100MB threshold
        let settings = create_test_settings(true, "100MB", false, None);
        plugin.configure(&settings).unwrap();

        // Scan the directory
        let results = plugin.scan(temp_dir.path()).unwrap();

        // Should not include small files
        let file_names: Vec<String> = results
            .iter()
            .map(|r| r.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(!file_names.contains(&"small_file.txt".to_string()));
        assert!(!file_names.contains(&"nested_small.txt".to_string()));
    }

    #[test]
    fn test_age_based_filtering() {
        let temp_dir = setup_test_directory();
        let mut plugin = LargeFilePlugin::new();

        // Configure with age filter (files older than 0 days - should include all)
        let settings = create_test_settings(true, "50MB", false, Some(0));
        plugin.configure(&settings).unwrap();

        let results = plugin.scan(temp_dir.path()).unwrap();

        // All large files should be included when age filter is 0
        assert!(results.len() >= 3);
    }

    #[test]
    fn test_scan_handles_empty_directory() {
        let temp_dir = TempDir::new("empty_test").unwrap();
        let mut plugin = LargeFilePlugin::new();

        let settings = create_test_settings(true, "100MB", false, None);
        plugin.configure(&settings).unwrap();

        let results = plugin.scan(temp_dir.path()).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_scan_handles_nonexistent_path() {
        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", false, None);
        plugin.configure(&settings).unwrap();

        let result = plugin.scan(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_collection() {
        let temp_dir = setup_test_directory();
        let mut plugin = LargeFilePlugin::new();

        let settings = create_test_settings(true, "50MB", false, None);
        plugin.configure(&settings).unwrap();

        let results = plugin.scan(temp_dir.path()).unwrap();

        for result in results {
            // Verify metadata is collected
            assert!(result.size > 0);
            assert!(!result.description.is_empty());
            assert!(result.path.exists());
        }
    }

    #[test]
    fn test_parallel_scanning_performance() {
        // Create a larger test structure
        let temp_dir = TempDir::new("perf_test").unwrap();
        let base_path = temp_dir.path();

        // Create multiple directories with files
        for i in 0..5 {
            let dir = base_path.join(format!("dir_{}", i));
            fs::create_dir(&dir).unwrap();

            for j in 0..3 {
                create_file_with_size(
                    &dir.join(format!("file_{}_{}.dat", i, j)),
                    if j == 0 { 150 } else { 50 },
                )
                .unwrap();
            }
        }

        let mut plugin = LargeFilePlugin::new();
        let settings = create_test_settings(true, "100MB", false, None);
        plugin.configure(&settings).unwrap();

        let start = SystemTime::now();
        let results = plugin.scan(temp_dir.path()).unwrap();
        let duration = start.elapsed().unwrap();

        // Should find 5 files (one 150MB file per directory)
        assert_eq!(results.len(), 5);

        // Parallel scanning should be reasonably fast (this is a soft assertion)
        // In practice, this should complete in under 1 second for this test size
        assert!(duration < Duration::from_secs(2));
    }

    /// Helper function to create test settings
    fn create_test_settings(
        enable_large_files: bool,
        size_threshold: &str,
        include_git_tracked: bool,
        older_than_days: Option<u64>,
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
}
