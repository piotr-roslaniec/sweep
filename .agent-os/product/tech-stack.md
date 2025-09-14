# Technical Stack

> Last Updated: 2025-09-13
> Version: 1.0.0

## Application Framework

- **Language:** Rust
- **Current Version:** Rust 2018 Edition (upgrading to 2021+ Edition)
- **Architecture:** Cross-platform CLI with parallel processing
- **Build System:** Cargo with workspace management

## Core Dependencies (Current → Planned)

- **CLI Framework:** structopt 0.3.5 → clap 4.x (structopt is deprecated)
- **Parallel Processing:** crossbeam 0.7.3 → crossbeam-channel 0.5.x + rayon 1.8.x
- **Regular Expressions:** regex 1.3.1 → regex 1.10.x (security updates)
- **Terminal Colors:** yansi 0.5.0 → yansi 1.0.x or crossterm
- **Terminal Size:** term_size 0.3.1 → terminal_size 0.3.x
- **Path Handling:** dunce 1.0.0 → maintained (already modern)
- **File System:** std::fs → async-std or tokio for better performance
- **Configuration:** Custom .swpfile parser → serde-based structured config

## Distribution & Packaging

- **Primary:** NPM wrapper for cross-platform installation
- **Native:** Cargo for Rust ecosystem users
- **Binaries:** GitHub Releases with automated CI/CD
- **Documentation:** VuePress (current) → modern docs framework

## Database

- **Configuration Storage:** File-based .swpfile system
- **Cache:** In-memory project discovery cache
- **State:** Stateless operations with optional persistence

## Build & Deployment

- **CI/CD:** GitHub Actions (needs modernization)
- **Testing:** Rust standard testing + integration tests
- **Cross-Platform Builds:** Multiple target compilation
- **Release Automation:** Semantic versioning with automated publishing

## Development Tools

- **Linting:** Clippy (legacy rules → modern best practices)
- **Formatting:** rustfmt with 2021 edition standards
- **Documentation:** rustdoc with enhanced examples
- **Debugging:** Standard Rust debugging tools + tracing

## Modernization Roadmap

**Phase 1: Foundation Updates**
- Upgrade to Rust 2021 Edition
- Update all dependencies (5-6 years of technical debt)
- Modernize CI/CD pipeline and GitHub Actions
- Enhanced error handling and logging

**Phase 2: Performance & UX**
- Async I/O implementation for faster scanning
- Enhanced terminal UI with progress indicators
- Improved parallel processing algorithms
- Better cross-platform file system handling

**Phase 3: Feature Expansion**
- Plugin architecture for language detection
- Configuration management improvements
- Integration with development environment tools
- Advanced analytics and reporting features

## Architecture Patterns

- **Command Pattern:** CLI subcommands with clear separation
- **Strategy Pattern:** Language-specific cleanup implementations
- **Observer Pattern:** Progress reporting and user feedback
- **Factory Pattern:** Project type detection and handler creation

## Security Considerations

- **File System Safety:** Read-only scanning, explicit confirmation for deletions
- **Input Validation:** Path sanitization and bounds checking
- **Permissions:** Respect file system permissions and ownership
- **Configuration Security:** Safe parsing of .swpfile contents

## Performance Targets

- **Scanning Speed:** <5 seconds for typical development directory
- **Memory Usage:** <50MB RSS during operation
- **Parallel Efficiency:** Utilize all available CPU cores
- **Startup Time:** <1 second cold start