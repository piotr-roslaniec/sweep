# Spec Requirements Document

> Spec: Python Support with Plugin Architecture
> Created: 2025-09-14

## Overview

Implement comprehensive Python project detection and cleanup capabilities through a new plugin architecture system. This will enable Sweep to identify and clean Python projects across all major package managers (pip, Poetry, conda, virtualenv) while establishing an extensible framework for future language support additions.

## User Stories

### Python Developer Cleanup Story

As a Python developer, I want Sweep to detect and clean all my Python projects regardless of package manager, so that I can reclaim disk space from virtual environments, caches, and build artifacts without manually navigating each project.

The workflow involves Sweep automatically detecting Python projects by identifying setup.py, pyproject.toml, or requirements.txt files, then safely removing __pycache__, .pyc files, virtual environments (venv/, .venv/, env/), test caches (.pytest_cache, .mypy_cache), and build artifacts (dist/, build/, *.egg-info) while respecting .gitignore patterns. This solves the problem where Python projects accumulate gigabytes of regenerable artifacts across different package managers and development tools.

### Plugin Architecture Story

As a Sweep maintainer, I want a plugin system for language detection, so that I can easily add support for new languages without modifying core code and enable community contributions.

The plugin system will define a trait-based interface for language detectors, allowing each plugin to specify project identification patterns, cleanable directories, and safety rules. This creates a scalable architecture where adding support for Go, Ruby, or other languages becomes a matter of implementing a simple plugin rather than modifying core application logic.

### Safety-First Cleanup Story

As a developer with sensitive Python projects, I want Sweep to respect .gitignore patterns and never delete critical files, so that I can trust the tool won't destroy my local development environment or data.

Sweep will parse .gitignore files to exclude protected patterns, maintain a hardcoded blocklist for sensitive files (.env, *.db, *.sqlite), and provide clear warnings when detecting potentially important files. This addresses the fear of accidentally deleting database files, environment configurations, or other non-recoverable project data.

## Spec Scope

1. **Plugin Architecture System** - Trait-based plugin interface for language detection with dynamic loading capability
2. **Python Language Plugin** - Complete Python project detection supporting pip, Poetry, conda, and virtualenv
3. **Cleanup Target Implementation** - Safe removal of Python-specific artifacts including caches, build files, and virtual environments
4. **Safety Integration** - .gitignore parsing and sensitive file protection with configurable blocklists
5. **Migration Strategy** - Refactor existing Java/JavaScript detection to use the new plugin system

## Out of Scope

- GUI or web interface for plugin management
- Remote plugin downloading or marketplace functionality
- Python version management or switching
- Modification of existing CLI commands or flags
- Auto-detection of custom virtual environment names beyond standard patterns

## Expected Deliverable

1. Successfully detect Python projects using any major package manager and clean appropriate artifacts with zero data loss
2. Demonstrate plugin architecture by migrating existing language support and adding Python as a plugin
3. Show respect for .gitignore patterns and protection of sensitive files through comprehensive test coverage