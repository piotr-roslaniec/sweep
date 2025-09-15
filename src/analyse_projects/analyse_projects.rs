use std::path::PathBuf;

use crossbeam::queue::SegQueue;
use yansi::Color;

use crate::output;
use crate::Project;
use crate::Settings;

use super::filter_by_modified_date::filter_by_modified_date;

/// Analyses a queue of projects loaded from `discover_projects()`
///
/// # Arguments
/// `projects` - The discovered projects
/// `settings` - The application settings struct
///
/// # Returns
/// All discovered cleanable directories
pub fn analyse_projects(projects: SegQueue<Project>, settings: &Settings) -> Vec<PathBuf> {
    let filtered = if settings.all {
        output::println(
            "Skip",
            Color::Yellow,
            "--all flag set, ignoring last used time",
        );
        projects
    } else {
        filter_by_modified_date(projects)
    };

    if filtered.is_empty() {
        return Vec::new();
    }

    let mut dirs = Vec::new();
    while let Ok(project) = filtered.pop() {
        dirs.append(&mut project.into_cleanable_dirs());
    }

    dirs.sort();

    // Filter out subdirectories when their parent directory is already in the list
    // This prevents "No such file or directory" errors when trying to delete
    // a subdirectory after its parent has already been deleted
    let mut filtered_dirs = Vec::new();
    for dir in dirs {
        // Check if any already-accepted directory is a parent of this one
        let is_subdirectory = filtered_dirs
            .iter()
            .any(|parent: &PathBuf| dir.starts_with(parent) && &dir != parent);

        if !is_subdirectory {
            // Remove any existing subdirectories of this new directory
            filtered_dirs.retain(|existing| !existing.starts_with(&dir) || existing == &dir);
            filtered_dirs.push(dir);
        }
    }

    filtered_dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filters_nested_directories() {
        // Test data - simulating directories that would be collected from projects
        let test_dirs = vec![
            PathBuf::from("/test/project/node_modules"),
            PathBuf::from("/test/project/node_modules/.pnpm/some-package/dist"),
            PathBuf::from("/test/project/build"),
            PathBuf::from("/test/project/build/cache"),
        ];

        // Simulate the sorting that happens in analyse_projects
        let mut dirs = test_dirs.clone();
        dirs.sort();

        // Apply the filtering logic
        let mut filtered_dirs = Vec::new();
        for dir in dirs {
            let is_subdirectory = filtered_dirs
                .iter()
                .any(|parent: &PathBuf| dir.starts_with(parent) && &dir != parent);

            if !is_subdirectory {
                filtered_dirs.retain(|existing| !existing.starts_with(&dir) || existing == &dir);
                filtered_dirs.push(dir);
            }
        }

        // Should only contain parent directories, not nested ones
        assert_eq!(filtered_dirs.len(), 2);
        assert!(filtered_dirs.contains(&PathBuf::from("/test/project/build")));
        assert!(filtered_dirs.contains(&PathBuf::from("/test/project/node_modules")));

        // Should not contain subdirectories
        assert!(!filtered_dirs.contains(&PathBuf::from(
            "/test/project/node_modules/.pnpm/some-package/dist"
        )));
        assert!(!filtered_dirs.contains(&PathBuf::from("/test/project/build/cache")));
    }

    #[test]
    fn test_handles_overlapping_directories() {
        // Test that the algorithm correctly handles overlapping directory paths
        let mut dirs = vec![
            PathBuf::from("/project/dist"),
            PathBuf::from("/project/dist/assets"),
            PathBuf::from("/project/node_modules"),
            PathBuf::from("/project/node_modules/@types"),
            PathBuf::from("/project/node_modules/@types/node"),
            PathBuf::from("/other/project/build"),
        ];

        dirs.sort();

        let mut filtered_dirs = Vec::new();
        for dir in dirs {
            let is_subdirectory = filtered_dirs
                .iter()
                .any(|parent: &PathBuf| dir.starts_with(parent) && &dir != parent);

            if !is_subdirectory {
                filtered_dirs.retain(|existing| !existing.starts_with(&dir) || existing == &dir);
                filtered_dirs.push(dir);
            }
        }

        assert_eq!(filtered_dirs.len(), 3);
        assert!(filtered_dirs.contains(&PathBuf::from("/other/project/build")));
        assert!(filtered_dirs.contains(&PathBuf::from("/project/dist")));
        assert!(filtered_dirs.contains(&PathBuf::from("/project/node_modules")));
    }
}
