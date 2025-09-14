# Spec Requirements Document

> Spec: Large File Detection Plugin
> Created: 2025-09-14

## Overview

Implement a large file detection plugin that identifies files over configurable size thresholds across projects, with smart filtering to protect important files and interactive user selection for cleanup. This plugin will integrate with the new plugin architecture, providing flags for activation and configuration options like size thresholds and age filtering.

## User Stories

### Developer Space Recovery Story

As a developer with limited disk space, I want to find and selectively remove large files across my projects, so that I can quickly free up significant space without accidentally deleting important test data or resources.

The workflow starts with running Sweep with a flag like `--large-files` or `--size-threshold 100MB`, which scans all projects for files exceeding the threshold. The system presents an interactive list showing file paths, sizes, last modified dates, and whether files are git-tracked. Users can select which files to delete, with the system protecting git-tracked files and recently accessed files by default. This solves the problem where developers accumulate large test files, database dumps, and build artifacts that aren't caught by standard language-specific cleanup.

### Plugin Configuration Story

As a power user, I want to configure the large file plugin with specific parameters, so that I can customize detection based on my workflow and project types.

Users can combine plugin flags like `swp --large-files --size-threshold 500MB --older-than 30d` to find large files that haven't been accessed in 30 days. Each plugin (language or feature) has its own set of options that can be combined, such as `--python --older-than 60d` to clean only old Python artifacts. This provides granular control over what gets detected and cleaned, addressing diverse developer workflows and project requirements.

### Safe Cleanup Story

As a developer working with important data files, I want the large file detector to intelligently identify which files are safe to delete, so that I never lose critical test data or resources.

The plugin analyzes files using multiple heuristics: checking if files are tracked in git (protected by default), examining access times, matching against common test data patterns, and respecting .gitignore rules. Before deletion, users see a detailed report with visual indicators (color coding, icons) showing risk levels for each file. This prevents accidental deletion of important fixtures, test datasets, or manually curated resources.

## Spec Scope

1. **Plugin Flag System** - Command-line flags to enable/disable plugins with per-plugin configuration options
2. **Large File Scanner** - Efficient recursive scanning with configurable size thresholds (default 100MB)
3. **Smart Filtering Engine** - Multi-factor analysis including git tracking, access times, and file patterns
4. **Interactive Selection UI** - Terminal-based file list with selection, sorting, and filtering capabilities
5. **Age-Based Filtering** - `--older-than` flag support across all plugins for time-based cleanup

## Out of Scope

- Automatic deletion without user confirmation
- Cloud storage or network drive scanning
- File compression or archiving features
- Integration with external storage analysis tools
- Real-time file system monitoring

## Expected Deliverable

1. Successfully detect large files with `--large-files` flag showing interactive selection interface
2. Demonstrate plugin flag system working with both language and feature plugins using various configuration options
3. Protect all git-tracked and recently accessed files while allowing safe cleanup of identified large files