use std::fmt::Debug;
use std::path::Path;

use crate::settings::Settings;

/// Base trait for all plugins (language and feature plugins)
pub trait Plugin: Send + Sync + Debug {
    /// Unique identifier for the plugin
    fn name(&self) -> &str;

    /// Version of the plugin
    fn version(&self) -> &str;

    /// Check if this plugin is enabled based on CLI flags
    fn is_enabled(&self, settings: &Settings) -> bool;

    /// Configure the plugin based on CLI settings
    fn configure(&mut self, settings: &Settings) -> Result<(), PluginError>;

    /// Apply age filter if --older-than flag is provided
    fn apply_age_filter(&mut self, days: u64) -> Result<(), PluginError>;
}

/// Trait for feature plugins like large file detection
pub trait FeaturePlugin: Plugin {
    /// Scan for items this plugin can clean
    fn scan(&self, path: &Path) -> Result<Vec<ScanResult>, PluginError>;

    /// Present interactive selection to user
    fn interactive_select(&self, results: Vec<ScanResult>) -> Result<Vec<ScanResult>, PluginError>;

    /// Clean selected items
    fn clean(&self, selected: Vec<ScanResult>) -> Result<CleanupReport, PluginError>;
}

/// Result of a plugin scan
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub path: std::path::PathBuf,
    pub size: u64,
    pub description: String,
    pub risk_level: RiskLevel,
}

/// Risk level for cleanup operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

/// Report after cleanup operations
#[derive(Debug)]
pub struct CleanupReport {
    pub items_cleaned: usize,
    pub space_freed: u64,
    pub errors: Vec<String>,
}

/// Plugin-specific errors
#[derive(Debug)]
pub enum PluginError {
    Configuration(String),
    Scan(String),
    Cleanup(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for PluginError {
    fn from(error: std::io::Error) -> Self {
        PluginError::Io(error)
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            PluginError::Scan(msg) => write!(f, "Scan error: {}", msg),
            PluginError::Cleanup(msg) => write!(f, "Cleanup error: {}", msg),
            PluginError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for PluginError {}

pub mod filter;
pub mod large_files;
pub mod ui;
pub mod utils;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod tests;
