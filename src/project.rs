use std::path::{Path, PathBuf};

use crate::output;
use crate::swpfile::parse_swpfile;

/// Describes a discovered cleanable project
#[derive(Debug)]
pub struct Project {
    /// The root directory of the project
    root: PathBuf,

    /// Directories containing dependencies
    dependency_dirs: Vec<PathBuf>,

    /// Timestamp indicating when the project was last modified
    #[allow(dead_code)]
    last_modified: u64,
}

impl Project {
    /// Initialises a new project
    ///
    /// # Arguments
    /// `root` - The root directory of the project
    pub fn new<P: Into<PathBuf>>(root: P) -> Project {
        Project {
            root: root.into(),
            dependency_dirs: Vec::new(),
            last_modified: 0,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Marks a subdirectory of this project's root directory as cleanable,
    /// if that directory exists. If the subdirectory doesn't exist, nothing
    /// happens.
    ///
    /// # Arguments
    /// `subdir` - Name of the subdirectory inside the project root directory
    pub fn add_cleanable_dir_if_exists<P: Into<PathBuf>>(&mut self, subdir: P) {
        let mut path = self.root.clone();
        path.push(subdir.into());

        if path.exists() && path.is_dir() && !self.dependency_dirs.contains(&path) {
            self.dependency_dirs.push(path);
        }
    }

    /// Add directories matching a pattern (e.g., "*.egg-info" for Python)
    pub fn add_cleanable_dirs_by_pattern(&mut self, pattern: &str) {
        if let Ok(entries) = std::fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(pattern) && entry.path().is_dir() {
                        let path = entry.path();
                        if !self.dependency_dirs.contains(&path) {
                            self.dependency_dirs.push(path);
                        }
                    }
                }
            }
        }
    }

    /// Recursively find and add directories with a specific name (e.g., "__pycache__")
    pub fn add_cleanable_dirs_recursive(&mut self, dir_name: &str, max_depth: usize) {
        self.find_dirs_recursive(&self.root.clone(), dir_name, 0, max_depth);
    }

    fn find_dirs_recursive(
        &mut self,
        path: &Path,
        target_name: &str,
        depth: usize,
        max_depth: usize,
    ) {
        if depth > max_depth {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name == target_name && !self.dependency_dirs.contains(&entry_path) {
                            self.dependency_dirs.push(entry_path.clone());
                        }
                        // Don't recurse into hidden directories or common large directories
                        if !name.starts_with('.') && name != "node_modules" && name != "target" {
                            self.find_dirs_recursive(
                                &entry_path,
                                target_name,
                                depth + 1,
                                max_depth,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn load_swpfile(&mut self, filename: &str) {
        let paths = match parse_swpfile(&self.root, &PathBuf::from(filename)) {
            Ok(paths) => paths,
            Err(e) => {
                output::error(format!(
                    "Could not read .swpfile file in {}",
                    self.root.to_str().unwrap_or("")
                ));
                output::println_info(e.to_string());
                std::process::exit(1);
            }
        };

        for path in paths {
            self.add_cleanable_dir_if_exists(path);
        }
    }

    /// Checks if the given path is listed as a cleanable directory of this
    /// project
    pub fn is_cleanable_dir<P: Into<PathBuf>>(&self, path: P) -> bool {
        self.dependency_dirs.contains(&path.into())
    }

    /// Consumes the project and returns the dependency directories
    pub fn into_cleanable_dirs(self) -> Vec<PathBuf> {
        self.dependency_dirs
    }
}
