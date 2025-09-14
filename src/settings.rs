use std::path::{Path, PathBuf};

use dunce::canonicalize;
use regex::Regex;
use structopt::StructOpt;

pub enum SettingsError {
    InvalidPath(PathBuf),
}

pub type Result<T> = std::result::Result<T, SettingsError>;

/// Deletes unnecessary build artifacts and dependency directories in your projects.
///
/// Detects Rust, Java and NodeJS projects by default, or define your own cleanable directories by adding a `.cleanuprc` file to your project directory.
///
/// Questions, bugs & other issues: https://github.com/woubuc/sweep/issues
#[derive(Debug, StructOpt)]
pub struct Settings {
    /// One or more directories where `swp` should start searching for projects.
    /// Defaults to the current working directory if no paths are given.
    #[structopt(name = "PATH...")]
    pub paths: Vec<PathBuf>,

    /// Sweep even projects that were modified within the last 30 days.
    #[structopt(short = "a", long = "all")]
    pub all: bool,

    /// Exclude projects in directories matched by this regex pattern.
    #[structopt(short = "i", long = "ignore")]
    pub ignore: Option<Regex>,

    /// Skip confirmation prompt before removing directories. Use at your own risk.
    #[structopt(short = "f", long = "force")]
    pub force: bool,

    // Plugin activation flags
    /// Enable large file detection plugin
    #[structopt(long = "large-files")]
    pub enable_large_files: bool,

    /// Enable Python language plugin
    #[structopt(long = "python")]
    pub enable_python: bool,

    /// Enable Java language plugin
    #[structopt(long = "java")]
    pub enable_java: bool,

    /// Enable JavaScript/Node.js language plugin
    #[structopt(long = "javascript")]
    pub enable_javascript: bool,

    /// Enable Rust language plugin
    #[structopt(long = "rust")]
    pub enable_rust: bool,

    // Global plugin options
    /// Only clean files older than specified days (applies to all enabled plugins)
    #[structopt(long = "older-than", value_name = "DAYS")]
    pub older_than_days: Option<u64>,

    // Large file plugin specific options
    /// Size threshold for large file detection (e.g., "100MB", "1.5GB")
    #[structopt(long = "size-threshold", default_value = "100MB")]
    pub size_threshold: String,

    /// Include git-tracked files in large file detection
    #[structopt(long = "include-git-tracked")]
    pub include_git_tracked: bool,
}

impl Settings {
    /// Gets a Settings struct from the CLI arguments
    pub fn get() -> Result<Settings> {
        let mut settings: Settings = Settings::from_args();

        settings.validate()?;

        Ok(settings)
    }

    /// Validates the application-specific values in a settings struct.
    ///
    /// This method is called automatically when calling `.get()`, but it
    /// should be called manually when creating a custom settings object.
    pub fn validate(&mut self) -> Result<()> {
        // If no paths are set, add the current path
        if self.paths.is_empty() {
            self.paths.push(".".into());
        }

        // Resolve to absolute paths
        self.paths = {
            let paths: Result<Vec<PathBuf>> = self
                .paths
                .iter()
                .map(|p| canonicalize(p).map_err(|_| SettingsError::InvalidPath(p.clone())))
                .collect();

            paths?
        };

        Ok(())
    }

    /// Checks if a given path is ignored
    ///
    /// # Arguments
    /// * `ignore` - The ignore regex, if set
    /// * `path`   - Path to check against the ignore regex
    ///
    /// # Returns
    /// * `true`  - If the path matches the regex
    /// * `false` - If the regex and path don't match, if no ignore
    ///             regex was given, or if the path is empty
    pub fn is_path_ignored(&self, path: &Path) -> bool {
        if self.ignore.is_none() {
            return false;
        }

        let re = self.ignore.as_ref().unwrap();
        let path = path.to_str().unwrap_or("");

        if path.len() == 0 {
            return false;
        } else {
            return re.is_match(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_settings() {
        let mut settings = Settings {
            paths: vec![],
            all: false,
            ignore: None,
            force: false,
            enable_large_files: false,
            enable_python: false,
            enable_java: false,
            enable_javascript: false,
            enable_rust: false,
            older_than_days: None,
            size_threshold: "100MB".to_string(),
            include_git_tracked: false,
        };

        assert!(
            settings.validate().is_ok(),
            "An error occured while validating settings struct"
        );
        assert!(settings.paths.len() > 0, "Settings contains no paths");
    }

    #[test]
    fn invalid_path() {
        let mut settings = Settings {
            paths: vec!["./this_path_does_not_exist_1".into()],
            all: false,
            ignore: None,
            force: false,
            enable_large_files: false,
            enable_python: false,
            enable_java: false,
            enable_javascript: false,
            enable_rust: false,
            older_than_days: None,
            size_threshold: "100MB".to_string(),
            include_git_tracked: false,
        };

        let validate = settings.validate();
        assert!(
            validate.is_err(),
            "No error occured while validating invalid settings struct"
        );

        match validate.unwrap_err() {
            SettingsError::InvalidPath(_) => (),
        }
    }

    #[test]
    fn ignore_flag() {
        let settings = Settings {
            paths: vec![],
            all: false,
            ignore: Some(Regex::new("src").unwrap()),
            force: false,
            enable_large_files: false,
            enable_python: false,
            enable_java: false,
            enable_javascript: false,
            enable_rust: false,
            older_than_days: None,
            size_threshold: "100MB".to_string(),
            include_git_tracked: false,
        };

        assert_eq!(settings.is_path_ignored(Path::new("./src")), true);
        assert_eq!(settings.is_path_ignored(Path::new("./foo")), false);
    }

    #[test]
    fn plugin_flags() {
        let settings = Settings {
            paths: vec![],
            all: false,
            ignore: None,
            force: false,
            enable_large_files: true,
            enable_python: true,
            enable_java: false,
            enable_javascript: false,
            enable_rust: false,
            older_than_days: Some(30),
            size_threshold: "500MB".to_string(),
            include_git_tracked: false,
        };

        assert!(settings.enable_large_files);
        assert!(settings.enable_python);
        assert!(!settings.enable_java);
        assert_eq!(settings.older_than_days, Some(30));
        assert_eq!(settings.size_threshold, "500MB");
    }
}
