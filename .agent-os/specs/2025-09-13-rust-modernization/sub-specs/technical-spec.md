# Technical Specification

This is the technical specification for the spec detailed in @.agent-os/specs/2025-09-13-rust-modernization/spec.md

## Technical Requirements

### Rust Edition Migration
- Update `Cargo.toml` edition field from "2018" to "2021"
- Replace deprecated syntax patterns:
  - Convert `extern crate` declarations to use statements
  - Update macro imports to use 2021 resolution rules
  - Migrate to new closure capture syntax where applicable
- Update minimum Rust version to latest stable (1.75+)
- Enable new edition lints and warnings

### Dependency Updates

#### CLI Framework Migration
- Replace structopt 0.3.5 with clap 4.5.x
- Migrate from derive macro patterns to clap's builder or derive API
- Update all command definitions and argument parsing
- Implement new help text formatting and validation rules

#### Parallel Processing Updates
- Update crossbeam 0.7.3 to crossbeam-channel 0.5.x
- Add rayon 1.8.x for data parallelism
- Refactor thread pool management for better CPU utilization
- Implement work-stealing algorithms for directory traversal

#### Core Library Updates
- regex 1.3.1 → regex 1.10.x (includes security fixes)
- yansi 0.5.0 → yansi 1.0.x (or evaluate crossterm for cross-platform color)
- term_size 0.3.1 → terminal_size 0.3.x
- num_cpus 1.11.1 → num_cpus 1.16.x

### Async I/O Implementation
- Integrate tokio 1.35.x runtime with multi-threaded scheduler
- Convert file system operations to async using tokio::fs
- Implement async directory walking with tokio-stream
- Add proper cancellation and timeout handling
- Maintain sync API compatibility layer for gradual migration

### Testing Infrastructure
- Add integration test harness using tempfile for isolated testing
- Implement performance benchmarks using criterion
- Create property-based tests using proptest for path handling
- Add async test support with tokio::test macro
- Target 80%+ code coverage with tarpaulin

### Performance Optimizations
- Implement concurrent directory scanning with bounded channels
- Add caching layer for repeated project type detection
- Use memory-mapped I/O for large .swpfile parsing
- Optimize regex compilation with once_cell lazy statics
- Profile and optimize hot paths identified by flamegraph

### Error Handling Improvements
- Migrate to thiserror for error type definitions
- Implement context-aware error messages with color-eyre
- Add structured logging with tracing
- Improve error recovery and partial operation success

### CI/CD Modernization
- Update GitHub Actions to use latest Rust toolchain actions
- Add matrix testing for multiple Rust versions and platforms
- Implement automated dependency updates with dependabot
- Add security scanning with cargo-audit
- Enable incremental compilation caching

## External Dependencies

### New Core Dependencies
- **clap** v4.5.x - Modern CLI argument parsing to replace deprecated structopt
  - **Justification:** structopt is deprecated and clap 4.x offers better performance, compile times, and features

- **tokio** v1.35.x - Async runtime for I/O operations
  - **Justification:** Industry standard async runtime, required for performant directory scanning

- **rayon** v1.8.x - Data parallelism library
  - **Justification:** Better parallel iteration performance than manual thread management

- **thiserror** v1.0.x - Error type derivation
  - **Justification:** Reduces boilerplate in error handling code

- **tracing** v0.1.x - Structured logging and diagnostics
  - **Justification:** Better debugging and performance profiling capabilities

### New Dev Dependencies
- **criterion** v0.5.x - Benchmarking framework
  - **Justification:** Track performance improvements during modernization

- **proptest** v1.4.x - Property-based testing
  - **Justification:** Better test coverage for path handling edge cases

- **tempfile** v3.8.x - Temporary file handling for tests
  - **Justification:** Safe, isolated testing environment

- **tarpaulin** v0.27.x - Code coverage tool
  - **Justification:** Ensure comprehensive test coverage

### Migration Dependencies (Temporary)
- **cargo-fix** - Automated edition migration assistance
  - **Justification:** Helps automate syntax updates for edition migration

- **cargo-udeps** - Find unused dependencies
  - **Justification:** Clean up legacy dependencies during modernization