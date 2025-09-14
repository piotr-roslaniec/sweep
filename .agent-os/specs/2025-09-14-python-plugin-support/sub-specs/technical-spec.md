# Technical Specification

This is the technical specification for the spec detailed in @.agent-os/specs/2025-09-14-python-plugin-support/spec.md

## Technical Requirements

### Plugin Architecture Design

#### Core Plugin Trait
```rust
pub trait LanguagePlugin: Send + Sync {
    // Unique identifier for the plugin
    fn name(&self) -> &str;

    // Version of the plugin
    fn version(&self) -> &str;

    // Check if a directory contains this language's project
    fn detect_project(&self, path: &Path) -> bool;

    // Return list of cleanable directories/patterns
    fn cleanable_patterns(&self) -> Vec<CleanablePattern>;

    // Get files/patterns that should never be deleted
    fn protected_patterns(&self) -> Vec<String>;

    // Calculate space that would be freed
    fn calculate_size(&self, path: &Path) -> Result<u64>;

    // Perform the actual cleanup
    fn clean(&self, path: &Path, dry_run: bool) -> Result<CleanupReport>;
}
```

#### Plugin Registry System
- Central registry to manage all language plugins
- Dynamic plugin loading at runtime
- Plugin discovery from designated directories
- Configuration file for enabling/disabling plugins
- Priority system for plugin execution order

#### Migration of Existing Languages
- Refactor current Java detection into JavaPlugin
- Refactor current JavaScript detection into JavaScriptPlugin
- Refactor current Rust detection into RustPlugin
- Maintain backwards compatibility with existing .swpfile format

### Python Plugin Implementation

#### Detection Logic
```rust
// Detect Python projects by presence of:
- setup.py
- pyproject.toml
- requirements.txt, requirements-*.txt
- Pipfile, Pipfile.lock
- poetry.lock
- environment.yml, environment.yaml
- tox.ini
- .python-version
```

#### Cleanable Patterns
```rust
CleanablePattern {
    // Virtual Environments
    - "venv/", ".venv/", "env/", ".env/" (directories)
    - Custom patterns from VIRTUAL_ENV detection

    // Python Bytecode
    - "**/__pycache__/" (recursive)
    - "**/*.pyc", "**/*.pyo", "**/*.pyd"
    - "**/*.so" (compiled extensions)

    // Package Management
    - ".tox/" (tox testing)
    - "*.egg-info/", "*.dist-info/"
    - "build/", "dist/" (setuptools)
    - ".eggs/" (setuptools cache)
    - "pip-wheel-metadata/"

    // Testing & Analysis Caches
    - ".pytest_cache/"
    - ".mypy_cache/"
    - ".coverage", "htmlcov/"
    - ".hypothesis/"
    - ".ruff_cache/"

    // Jupyter
    - ".ipynb_checkpoints/"

    // Documentation
    - "docs/_build/", "docs/.doctrees/"
}
```

#### Protected Patterns
```rust
// Never delete these files
- "*.db", "*.sqlite", "*.sqlite3" (databases)
- ".env", ".env.*" (environment configs)
- "*.key", "*.pem", "*.crt" (certificates)
- ".git/" (version control)
- User-defined patterns from .swpfile
```

### Safety Features

#### Gitignore Integration
- Parse .gitignore files in project root
- Respect nested .gitignore files
- Use ignore crate for efficient pattern matching
- Cache parsed patterns for performance
- Option to override with --ignore-gitignore flag

#### Sensitive File Detection
- Implement heuristic detection for sensitive files
- Check file headers for database signatures
- Warn user before deleting files over certain size threshold
- Maintain blocklist of known sensitive patterns
- Allow user to add custom protected patterns

### Performance Optimizations

#### Parallel Processing
- Use rayon for parallel directory traversal
- Implement work-stealing queue for plugin execution
- Batch file operations for efficiency
- Cache detection results to avoid repeated checks

#### Memory Management
- Stream large directory listings instead of loading all at once
- Use memory-mapped I/O for large file inspection
- Implement size calculation with running totals
- Early termination when threshold reached

### Configuration Management

#### Plugin Configuration
```toml
# .sweep/plugins.toml
[plugins]
enabled = ["rust", "javascript", "java", "python"]

[python]
additional_patterns = ["custom_env/"]
protected_files = ["local_settings.py"]
respect_gitignore = true
max_env_size_gb = 5.0
```

#### User Overrides
- Allow per-project plugin configuration
- Support global user configuration in ~/.sweep/config
- Command-line flags override configuration files
- Environment variables for CI/CD environments

### Testing Strategy

#### Unit Tests
- Test each plugin method independently
- Mock file system operations
- Test pattern matching accuracy
- Verify size calculations

#### Integration Tests
- Create test Python projects with various configurations
- Test cleanup with different package managers
- Verify gitignore respect
- Test protection of sensitive files

#### Plugin System Tests
- Test plugin loading and registration
- Test plugin priority and ordering
- Test configuration parsing and overrides
- Test backwards compatibility

## External Dependencies

### New Core Dependencies

- **ignore** v0.4.x - Gitignore parsing and pattern matching
  - **Justification:** Industry standard for respecting gitignore patterns, used by ripgrep

- **globset** v0.4.x - Glob pattern matching for file paths
  - **Justification:** Efficient pattern matching for cleanable and protected patterns

- **serde** v1.0.x - Serialization for plugin configuration
  - **Justification:** Required for TOML configuration parsing

- **toml** v0.8.x - TOML configuration file parsing
  - **Justification:** User-friendly configuration format

### Optional Dependencies

- **pyo3** v0.20.x - Python interpreter bindings (optional)
  - **Justification:** Could enable more intelligent Python project detection by actually parsing Python files

- **notify** v6.1.x - File system watching (optional)
  - **Justification:** Could enable real-time monitoring of project changes for cache invalidation