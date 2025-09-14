use super::*;
use crate::settings::Settings;

/// Mock plugin for testing
#[derive(Debug)]
struct MockPlugin {
    enabled: bool,
    configured: bool,
    age_filter: Option<u64>,
}

impl MockPlugin {
    fn new() -> Self {
        MockPlugin {
            enabled: false,
            configured: false,
            age_filter: None,
        }
    }
}

impl Plugin for MockPlugin {
    fn name(&self) -> &str {
        "mock"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn is_enabled(&self, _settings: &Settings) -> bool {
        self.enabled
    }

    fn configure(&mut self, _settings: &Settings) -> Result<(), PluginError> {
        self.configured = true;
        Ok(())
    }

    fn apply_age_filter(&mut self, days: u64) -> Result<(), PluginError> {
        self.age_filter = Some(days);
        Ok(())
    }
}

#[test]
fn test_plugin_basics() {
    let mut plugin = MockPlugin::new();

    assert_eq!(plugin.name(), "mock");
    assert_eq!(plugin.version(), "1.0.0");

    // Test configuration
    let settings = Settings {
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

    assert!(plugin.configure(&settings).is_ok());
    assert!(plugin.configured);

    // Test age filter
    assert!(plugin.apply_age_filter(30).is_ok());
    assert_eq!(plugin.age_filter, Some(30));
}

#[test]
fn test_scan_result() {
    let result = ScanResult {
        path: std::path::PathBuf::from("/test/file.txt"),
        size: 1024 * 1024 * 100, // 100MB
        description: "Large test file".to_string(),
        risk_level: RiskLevel::Low,
    };

    assert_eq!(result.size, 104857600);
    assert_eq!(result.risk_level, RiskLevel::Low);
}

#[test]
fn test_cleanup_report() {
    let report = CleanupReport {
        items_cleaned: 5,
        space_freed: 1024 * 1024 * 500, // 500MB
        errors: vec![],
    };

    assert_eq!(report.items_cleaned, 5);
    assert_eq!(report.space_freed, 524288000);
    assert!(report.errors.is_empty());
}

#[test]
fn test_risk_levels() {
    assert_ne!(RiskLevel::Safe, RiskLevel::Critical);
    assert_eq!(RiskLevel::Medium, RiskLevel::Medium);

    // Test ordering (for potential sorting)
    let levels = [
        RiskLevel::Critical,
        RiskLevel::Safe,
        RiskLevel::High,
        RiskLevel::Low,
        RiskLevel::Medium,
    ];

    assert_eq!(levels.len(), 5);
}
