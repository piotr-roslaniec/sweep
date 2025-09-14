# Spec Requirements Document

> Spec: Rust 2021 Edition & Dependency Modernization
> Created: 2025-09-13

## Overview

Modernize the Sweep CLI codebase by upgrading from Rust 2018 to 2021 edition and updating all dependencies that are 5-6 years outdated. This modernization will establish a solid foundation for future feature development, improve performance, and resolve security vulnerabilities in outdated dependencies.

## User Stories

### Developer Modernization Story

As a maintainer of Sweep, I want to upgrade the codebase to modern Rust standards, so that I can leverage new language features, improve performance, and ensure long-term maintainability.

The workflow involves incrementally updating dependencies with testing between each major change, migrating from deprecated crates (structopt to clap 4.x), and adopting async I/O patterns with tokio. This solves the technical debt problem where outdated dependencies prevent us from using modern Rust patterns and may contain security vulnerabilities.

### Performance Improvement Story

As a Sweep user, I want faster directory scanning and cleanup operations, so that I can reclaim disk space more efficiently even in large monorepo environments.

Users will experience significantly faster scanning through async I/O implementation, better CPU utilization via modern parallel processing libraries, and more responsive terminal feedback. This addresses the current limitation where synchronous I/O blocks during large directory traversals.

## Spec Scope

1. **Rust Edition Upgrade** - Migrate from Rust 2018 to 2021 edition with all necessary syntax updates
2. **Dependency Modernization** - Update all core dependencies including structopt→clap, crossbeam, regex, yansi, and term_size
3. **Async Runtime Integration** - Implement tokio-based async I/O for file system operations
4. **Test Suite Enhancement** - Add comprehensive integration tests for modernized components
5. **CI/CD Pipeline Update** - Modernize GitHub Actions workflows for latest Rust toolchain

## Out of Scope

- New feature development (Python support, large file detection)
- Changes to existing CLI commands or user-facing behavior (except minor breaking changes from clap migration)
- Documentation site migration from VuePress
- NPM wrapper modifications

## Expected Deliverable

1. Successfully compile and run Sweep with Rust 2021 edition and all modern dependencies
2. Pass all existing tests plus new integration test suite with >80% code coverage
3. Demonstrate 50%+ performance improvement in directory scanning benchmarks