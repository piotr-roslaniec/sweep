#![allow(dead_code)]

use super::{PluginError, RiskLevel};
use git2::{Repository, Status};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashMap;
use std::fs::Metadata;
/// Smart filtering engine for file analysis
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// File type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    TestData,
    Database,
    Archive,
    Media,
    Log,
    Binary,
    Document,
    Source,
    Configuration,
    Unknown,
}

/// Git file status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatus {
    Tracked,
    Modified,
    Untracked,
    Ignored,
    NotInRepo,
}

/// Smart filter for analyzing files
pub struct SmartFilter {
    git_repos: HashMap<PathBuf, Repository>,
    gitignore_cache: HashMap<PathBuf, Gitignore>,
    protected_patterns: Vec<String>,
    test_data_patterns: Vec<String>,
}

impl std::fmt::Debug for SmartFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmartFilter")
            .field("git_repos_count", &self.git_repos.len())
            .field("gitignore_cache_count", &self.gitignore_cache.len())
            .field("protected_patterns", &self.protected_patterns)
            .field("test_data_patterns", &self.test_data_patterns)
            .finish()
    }
}

impl SmartFilter {
    /// Create a new smart filter
    pub fn new() -> Self {
        SmartFilter {
            git_repos: HashMap::new(),
            gitignore_cache: HashMap::new(),
            protected_patterns: vec![
                ".env".to_string(),
                ".env.*".to_string(),
                "*.db".to_string(),
                "*.sqlite".to_string(),
                "*.sqlite3".to_string(),
                "*.key".to_string(),
                "*.pem".to_string(),
                "*.crt".to_string(),
                "*.p12".to_string(),
                "credentials*".to_string(),
                "secrets*".to_string(),
            ],
            test_data_patterns: vec![
                "test-data*".to_string(),
                "test_data*".to_string(),
                "fixture*".to_string(),
                "sample*".to_string(),
                "mock*".to_string(),
                "*.test.*".to_string(),
                "*.spec.*".to_string(),
                "*_test.*".to_string(),
                "*_spec.*".to_string(),
            ],
        }
    }

    /// Discover git repositories in a path and its parents
    pub fn discover_git_repos(&mut self, path: &Path) -> Result<(), PluginError> {
        let mut current = path;

        loop {
            // Try to open repository at current path
            if let Ok(repo) = Repository::open(current) {
                if let Some(workdir) = repo.workdir() {
                    if let Ok(canonical) = workdir.canonicalize() {
                        self.git_repos.insert(canonical, repo);
                        break;
                    }
                }
            }

            // Check for .git directory
            let git_dir = current.join(".git");
            if git_dir.exists() && git_dir.is_dir() {
                if let Ok(repo) = Repository::open(current) {
                    self.git_repos.insert(current.to_path_buf(), repo);
                    break;
                }
            }

            // Move to parent directory
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }

        Ok(())
    }

    /// Get git status for a file
    pub fn get_git_status(&self, file_path: &Path) -> GitFileStatus {
        // Find the repository containing this file
        for (repo_path, repo) in &self.git_repos {
            if file_path.starts_with(repo_path) {
                // Get relative path from repository root
                if let Ok(relative_path) = file_path.strip_prefix(repo_path) {
                    // Check file status
                    if let Ok(status) = repo.status_file(&relative_path) {
                        if status.contains(Status::IGNORED) {
                            return GitFileStatus::Ignored;
                        } else if status.contains(Status::WT_NEW) {
                            return GitFileStatus::Untracked;
                        } else if status.contains(Status::WT_MODIFIED)
                            || status.contains(Status::INDEX_MODIFIED)
                        {
                            return GitFileStatus::Modified;
                        } else if !status.is_empty() {
                            return GitFileStatus::Tracked;
                        } else {
                            return GitFileStatus::Tracked;
                        }
                    }
                }
            }
        }

        GitFileStatus::NotInRepo
    }

    /// Load gitignore patterns for a directory
    pub fn load_gitignore(&mut self, dir: &Path) -> Result<(), PluginError> {
        let gitignore_path = dir.join(".gitignore");

        if gitignore_path.exists() {
            let mut builder = GitignoreBuilder::new(dir);

            // add() returns Option<Error>, not Result
            if let Some(e) = builder.add(&gitignore_path) {
                return Err(PluginError::Configuration(format!(
                    "Failed to parse .gitignore: {}",
                    e
                )));
            }

            match builder.build() {
                Ok(gitignore) => {
                    self.gitignore_cache.insert(dir.to_path_buf(), gitignore);
                }
                Err(e) => {
                    return Err(PluginError::Configuration(format!(
                        "Failed to build gitignore: {}",
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if a file matches gitignore patterns
    pub fn is_gitignored(&self, file_path: &Path) -> bool {
        for (dir_path, gitignore) in &self.gitignore_cache {
            if file_path.starts_with(dir_path) {
                if let Ok(relative) = file_path.strip_prefix(dir_path) {
                    let is_dir = file_path.is_dir();
                    let matched = gitignore.matched(&relative, is_dir);
                    if matched.is_ignore() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Detect file type from extension and content patterns
    pub fn detect_file_type(&self, path: &Path) -> FileType {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            match ext_str.as_str() {
                // Test data
                "test" | "fixture" | "sample" | "mock" => return FileType::TestData,

                // Database
                "db" | "sqlite" | "sqlite3" | "sql" | "dump" => return FileType::Database,

                // Archive
                "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" => return FileType::Archive,

                // Media
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "ico" | "mp4" | "avi" | "mkv"
                | "mov" | "wmv" | "mp3" | "wav" | "flac" | "ogg" => return FileType::Media,

                // Log
                "log" | "out" | "err" => return FileType::Log,

                // Document
                "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods"
                | "odp" | "txt" | "md" | "rst" => return FileType::Document,

                // Source code
                "rs" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "h" | "hpp" | "go" | "rb"
                | "php" | "cs" | "swift" | "kt" => return FileType::Source,

                // Configuration
                "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" => {
                    return FileType::Configuration
                }

                // Binary
                "exe" | "dll" | "so" | "dylib" | "o" | "a" => return FileType::Binary,

                _ => {}
            }
        }

        // Check filename patterns
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy().to_lowercase();

            if name_str.contains("test")
                || name_str.contains("fixture")
                || name_str.contains("sample")
                || name_str.contains("mock")
            {
                return FileType::TestData;
            }

            if name_str.contains("log") {
                return FileType::Log;
            }
        }

        FileType::Unknown
    }

    /// Check if a file is protected (should never be deleted)
    pub fn is_protected(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();

            for pattern in &self.protected_patterns {
                if Self::matches_pattern(&name_str, pattern) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a file matches test data patterns
    pub fn is_test_data(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();

            for pattern in &self.test_data_patterns {
                if Self::matches_pattern(&name_str, pattern) {
                    return true;
                }
            }
        }
        false
    }

    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(text: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();

            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];

                if prefix.is_empty() && suffix.is_empty() {
                    return true;
                } else if prefix.is_empty() {
                    return text.ends_with(suffix);
                } else if suffix.is_empty() {
                    return text.starts_with(prefix);
                } else {
                    return text.starts_with(prefix) && text.ends_with(suffix);
                }
            } else if parts.len() == 3 {
                // Handle patterns like "*.test.*"
                let prefix = parts[0];
                let middle = parts[1];
                let suffix = parts[2];

                if prefix.is_empty() && suffix.is_empty() {
                    // Pattern is like "*middle*"
                    return text.contains(middle);
                }
            }
        }

        text == pattern
    }

    /// Calculate comprehensive risk level for a file
    pub fn calculate_risk_level(
        &self,
        path: &Path,
        metadata: &Metadata,
        include_git_tracked: bool,
    ) -> RiskLevel {
        // Check if file is protected - never delete
        if self.is_protected(path) {
            return RiskLevel::Critical;
        }

        // Check git status
        let git_status = self.get_git_status(path);
        match git_status {
            GitFileStatus::Tracked | GitFileStatus::Modified if !include_git_tracked => {
                return RiskLevel::Critical;
            }
            GitFileStatus::Ignored => {
                // Ignored files are generally safe to delete
                return RiskLevel::Safe;
            }
            _ => {}
        }

        // Check if file is in gitignore
        if self.is_gitignored(path) {
            return RiskLevel::Safe;
        }

        // Check modification time
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = SystemTime::now().duration_since(modified) {
                if age < Duration::from_secs(3 * 24 * 60 * 60) {
                    return RiskLevel::High; // Modified in last 3 days
                }
                if age < Duration::from_secs(7 * 24 * 60 * 60) {
                    return RiskLevel::Medium; // Modified in last week
                }
                if age < Duration::from_secs(30 * 24 * 60 * 60) {
                    return RiskLevel::Low; // Modified in last month
                }
            }
        }

        // Check file type
        let file_type = self.detect_file_type(path);
        match file_type {
            FileType::Database | FileType::Configuration => return RiskLevel::High,
            FileType::Source => return RiskLevel::Medium,
            FileType::TestData => return RiskLevel::Low,
            FileType::Log | FileType::Archive => return RiskLevel::Safe,
            _ => {}
        }

        // Check if it's test data
        if self.is_test_data(path) {
            return RiskLevel::Low;
        }

        // Default to low risk for old files
        RiskLevel::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempdir::TempDir;

    #[test]
    fn test_file_type_detection() {
        let filter = SmartFilter::new();

        assert_eq!(
            filter.detect_file_type(Path::new("test.db")),
            FileType::Database
        );
        assert_eq!(
            filter.detect_file_type(Path::new("archive.zip")),
            FileType::Archive
        );
        assert_eq!(
            filter.detect_file_type(Path::new("image.jpg")),
            FileType::Media
        );
        assert_eq!(filter.detect_file_type(Path::new("app.log")), FileType::Log);
        assert_eq!(
            filter.detect_file_type(Path::new("main.rs")),
            FileType::Source
        );
        assert_eq!(
            filter.detect_file_type(Path::new("config.json")),
            FileType::Configuration
        );
        assert_eq!(
            filter.detect_file_type(Path::new("test-data.csv")),
            FileType::TestData
        );
    }

    #[test]
    fn test_protected_file_detection() {
        let filter = SmartFilter::new();

        assert!(filter.is_protected(Path::new(".env")));
        assert!(filter.is_protected(Path::new(".env.local")));
        assert!(filter.is_protected(Path::new("database.db")));
        assert!(filter.is_protected(Path::new("private.key")));
        assert!(filter.is_protected(Path::new("cert.pem")));
        assert!(!filter.is_protected(Path::new("regular.txt")));
    }

    #[test]
    fn test_pattern_matching() {
        assert!(SmartFilter::matches_pattern("test.txt", "*.txt"));
        assert!(SmartFilter::matches_pattern("test-data.csv", "test-data*"));
        assert!(SmartFilter::matches_pattern("file.test.js", "*.test.*"));
        assert!(!SmartFilter::matches_pattern("test.txt", "*.csv"));
    }

    #[test]
    fn test_test_data_detection() {
        let filter = SmartFilter::new();

        assert!(filter.is_test_data(Path::new("test-data.json")));
        assert!(filter.is_test_data(Path::new("fixture.sql")));
        assert!(filter.is_test_data(Path::new("sample.csv")));
        assert!(filter.is_test_data(Path::new("mock_data.txt")));
        assert!(filter.is_test_data(Path::new("file.test.js")));
        assert!(!filter.is_test_data(Path::new("production.db")));
    }

    #[test]
    fn test_git_repo_discovery() {
        let temp_dir = TempDir::new("git_test").unwrap();
        let mut filter = SmartFilter::new();

        // Create a git repo
        let repo_path = temp_dir.path();
        Repository::init(repo_path).unwrap();

        // Discover the repo
        filter.discover_git_repos(repo_path).unwrap();

        assert!(!filter.git_repos.is_empty());
    }

    #[test]
    fn test_gitignore_loading() {
        let temp_dir = TempDir::new("gitignore_test").unwrap();
        let mut filter = SmartFilter::new();

        // Create a .gitignore file
        let gitignore_path = temp_dir.path().join(".gitignore");
        fs::write(&gitignore_path, "*.log\ntemp/\n").unwrap();

        // Load gitignore
        filter.load_gitignore(temp_dir.path()).unwrap();

        assert!(!filter.gitignore_cache.is_empty());
    }

    #[test]
    fn test_risk_level_calculation() {
        let filter = SmartFilter::new();
        let temp_dir = TempDir::new("risk_test").unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let metadata = fs::metadata(&test_file).unwrap();

        // Protected files should be critical
        let env_file = temp_dir.path().join(".env");
        let risk = filter.calculate_risk_level(&env_file, &metadata, false);
        assert_eq!(risk, RiskLevel::Critical);

        // Recent files should be high risk
        let risk = filter.calculate_risk_level(&test_file, &metadata, false);
        assert_eq!(risk, RiskLevel::High); // Just created, so very recent
    }
}
