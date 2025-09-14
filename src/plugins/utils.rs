/// Utility functions for plugin operations
#![allow(dead_code)]

use super::PluginError;
use regex::Regex;

/// Parse a human-readable size string into bytes
/// Supports formats like "100MB", "1.5GB", "500K", etc.
pub fn parse_size_string(size_str: &str) -> Result<u64, PluginError> {
    // Regex to match number (with optional decimal) and unit
    let re = Regex::new(r"^(\d+(?:\.\d+)?)\s*([KMGT]?B?)$")
        .map_err(|e| PluginError::Configuration(format!("Invalid regex: {}", e)))?;

    let size_str_upper = size_str.to_uppercase();
    let captures = re
        .captures(&size_str_upper)
        .ok_or_else(|| PluginError::Configuration(format!("Invalid size format: {}", size_str)))?;

    let number = captures
        .get(1)
        .ok_or_else(|| PluginError::Configuration("No number found".to_string()))?
        .as_str()
        .parse::<f64>()
        .map_err(|e| PluginError::Configuration(format!("Invalid number: {}", e)))?;

    let unit = captures.get(2).map(|m| m.as_str()).unwrap_or("B");

    let multiplier = match unit {
        "B" | "" => 1.0,
        "K" | "KB" => 1024.0,
        "M" | "MB" => 1024.0 * 1024.0,
        "G" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "T" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => {
            return Err(PluginError::Configuration(format!(
                "Unknown unit: {}",
                unit
            )))
        }
    };

    Ok((number * multiplier) as u64)
}

/// Format bytes into human-readable string
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else if size >= 100.0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else if size >= 10.0 {
        format!("{:.1} {}", size, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_string() {
        // Test various formats
        assert_eq!(parse_size_string("100").unwrap(), 100);
        assert_eq!(parse_size_string("100B").unwrap(), 100);
        assert_eq!(parse_size_string("1KB").unwrap(), 1024);
        assert_eq!(parse_size_string("1K").unwrap(), 1024);
        assert_eq!(parse_size_string("100MB").unwrap(), 100 * 1024 * 1024);
        assert_eq!(parse_size_string("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(
            parse_size_string("1.5GB").unwrap(),
            (1.5 * 1024.0 * 1024.0 * 1024.0) as u64
        );
        assert_eq!(
            parse_size_string("2TB").unwrap(),
            2 * 1024 * 1024 * 1024 * 1024
        );

        // Test with spaces
        assert_eq!(parse_size_string("100 MB").unwrap(), 100 * 1024 * 1024);
        assert_eq!(
            parse_size_string("1.5 GB").unwrap(),
            (1.5 * 1024.0 * 1024.0 * 1024.0) as u64
        );

        // Test case insensitive
        assert_eq!(parse_size_string("100mb").unwrap(), 100 * 1024 * 1024);
        assert_eq!(parse_size_string("1gb").unwrap(), 1024 * 1024 * 1024);

        // Test invalid formats
        assert!(parse_size_string("invalid").is_err());
        assert!(parse_size_string("100XB").is_err());
        assert!(parse_size_string("MB100").is_err());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(100 * 1024 * 1024), "100 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(
            format_size((1.5 * 1024.0 * 1024.0 * 1024.0) as u64),
            "1.50 GB"
        );
        assert_eq!(format_size(1024_u64 * 1024 * 1024 * 1024), "1.00 TB");

        // Test edge cases
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1025), "1.00 KB");
        assert_eq!(format_size(10 * 1024), "10.0 KB");
        assert_eq!(format_size(100 * 1024), "100 KB");
    }

    #[test]
    fn test_roundtrip() {
        // Test that parsing and formatting are consistent
        let sizes = vec![
            100,
            1024,
            100 * 1024,
            1024 * 1024,
            100 * 1024 * 1024,
            1024 * 1024 * 1024,
        ];

        for size in sizes {
            let formatted = format_size(size);
            let parsed = parse_size_string(&formatted).unwrap();
            // Allow small rounding differences
            assert!((parsed as i64 - size as i64).abs() < 1024);
        }
    }
}
