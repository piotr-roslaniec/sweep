# Product Mission

> Last Updated: 2025-09-13
> Version: 1.0.0

## Pitch

Sweep CLI is a developer productivity tool that automatically reclaims disk space by cleaning up old project dependencies and build artifacts across multiple programming languages. Think of it as a smart janitor for your development environment - it knows what's safe to delete and what isn't, helping individual developers maintain clean, efficient workspaces without the tedious manual cleanup.

## Users

**Primary Users:** Individual software developers managing multiple projects locally

**User Personas:**
- **The Polyglot Developer**: Works across JavaScript, Rust, Java, and Python projects, accumulates gigabytes of node_modules, target/, and build artifacts
- **The Project Collector**: Has dozens of repositories, side projects, and experiments taking up valuable disk space
- **The Clean Workspace Advocate**: Values organized development environments and regularly maintains project hygiene
- **The Storage-Conscious Developer**: Works on laptops with limited disk space or expensive SSD storage

**User Journey:** Developer notices low disk space → runs `sweep` → sees interactive list of cleanable projects → selectively removes old dependencies → reclaims significant storage space

## The Problem

**Core Problem:** Development projects accumulate massive amounts of regenerable artifacts (node_modules, target/, build/, __pycache__, etc.) that consume valuable disk space but are rarely cleaned up systematically.

**Pain Points:**
- Manual cleanup is time-consuming and error-prone
- Developers forget which directories are safe to delete
- No unified tool for multiple programming languages
- Disk space fills up gradually until it becomes critical
- Existing tools are language-specific or too aggressive
- Fear of deleting important files prevents regular cleanup

**Market Gap:** While language-specific cleaners exist (npm prune, cargo clean), no tool provides unified, safe, configurable cleanup across multiple programming ecosystems with intelligent project detection.

## Differentiators

**Unique Value Props:**
1. **Multi-Language Intelligence**: Understands project structures across Rust, JavaScript, Java, Python, and more
2. **Safety-First Approach**: Interactive confirmation prevents accidental deletion of important files
3. **Configurable via .swpfile**: Project-specific cleanup rules and exclusions
4. **Cross-Platform Consistency**: Works identically on Windows, macOS, and Linux
5. **Performance-Optimized**: Parallel processing for fast directory scanning and cleanup
6. **Developer-Friendly UX**: Colored output, clear progress indicators, and intuitive commands

**Competitive Advantages:**
- Only tool combining multi-language support with safety features
- Mature codebase (v1.0.3) with proven reliability
- Active maintenance and modernization roadmap
- NPM distribution for easy installation
- Comprehensive documentation and examples

## Key Features

**Core Functionality:**
- **Project Discovery**: Automatically finds projects across multiple directory paths
- **Language Support**: Rust (Cargo), JavaScript/Node.js (NPM/Yarn), Java (Maven/Gradle), Python (pip/Poetry/conda)
- **Interactive Deletion**: Confirmation prompts before removing any files
- **Configuration System**: .swpfile support for custom rules and exclusions
- **Parallel Processing**: Fast scanning and cleanup operations
- **Cross-Platform**: Consistent behavior across operating systems

**Advanced Features:**
- **Large File Detection**: Identifies oversized files with smart filtering (excludes test files)
- **Dependency Health**: Shows age and staleness of cached dependencies
- **Storage Analytics**: Reports potential space savings before cleanup
- **Batch Operations**: Clean multiple projects simultaneously
- **Selective Cleanup**: Choose specific artifact types to remove

**Success Metrics:**
- **Adoption**: Downloads via NPM, GitHub stars, user retention
- **Effectiveness**: Average disk space reclaimed per run (target: >1GB)
- **Safety**: Zero reported data loss incidents
- **Performance**: Sub-5 second scanning for typical development directories
- **Coverage**: Support for 90%+ of common development project types

**Long-term Vision:** Become the standard disk space management tool for developers, integrated into development workflows and IDE extensions, with intelligent predictive cleanup and automated maintenance scheduling.