# Spec Tasks

## Tasks

- [ ] 1. Implement Plugin Flag System and CLI Extensions
  - [ ] 1.1 Write tests for command-line flag parsing and plugin activation
  - [ ] 1.2 Extend CLI structure with plugin-specific flags (--large-files, --size-threshold, --older-than)
  - [ ] 1.3 Implement Plugin trait with enable/disable and configuration methods
  - [ ] 1.4 Create FeaturePlugin trait for non-language plugins
  - [ ] 1.5 Add global plugin options parsing (--older-than for all plugins)
  - [ ] 1.6 Implement size string parser for human-readable formats (100MB, 1.5GB)
  - [ ] 1.7 Verify all tests pass

- [ ] 2. Build Core Large File Scanner
  - [ ] 2.1 Write tests for file scanning with size thresholds
  - [ ] 2.2 Implement LargeFilePlugin struct with configuration
  - [ ] 2.3 Create parallel directory scanning using rayon
  - [ ] 2.4 Add file metadata collection (size, timestamps, paths)
  - [ ] 2.5 Implement age-based filtering logic
  - [ ] 2.6 Handle permission errors and edge cases gracefully
  - [ ] 2.7 Add progress reporting during scan
  - [ ] 2.8 Verify all tests pass

- [ ] 3. Develop Smart Filtering Engine
  - [ ] 3.1 Write tests for risk level calculation and git integration
  - [ ] 3.2 Integrate git2 for repository discovery and file tracking status
  - [ ] 3.3 Implement multi-factor risk analysis (git status, access time, patterns)
  - [ ] 3.4 Create file type detection from extensions and patterns
  - [ ] 3.5 Add test data pattern recognition (fixture*, test-data-*, etc.)
  - [ ] 3.6 Implement protected file detection (.env, .db, etc.)
  - [ ] 3.7 Cache git repository information for performance
  - [ ] 3.8 Verify all tests pass

- [ ] 4. Create Interactive Selection UI
  - [ ] 4.1 Write tests for terminal UI components and user interactions
  - [ ] 4.2 Implement file list display with TUI framework
  - [ ] 4.3 Add selection controls (space to toggle, enter to confirm)
  - [ ] 4.4 Create risk level indicators with color coding
  - [ ] 4.5 Implement sorting options (size, age, risk, name)
  - [ ] 4.6 Add filtering capabilities in the UI
  - [ ] 4.7 Format file sizes and ages for human readability
  - [ ] 4.8 Verify all tests pass

- [ ] 5. Integration and End-to-End Testing
  - [ ] 5.1 Write integration tests for complete plugin workflow
  - [ ] 5.2 Test plugin activation with various flag combinations
  - [ ] 5.3 Verify git-tracked file protection works correctly
  - [ ] 5.4 Test interactive UI with mock terminal input
  - [ ] 5.5 Validate age-based filtering across different scenarios
  - [ ] 5.6 Test error handling and edge cases
  - [ ] 5.7 Create example documentation and usage guide
  - [ ] 5.8 Verify all tests pass and plugin works end-to-end