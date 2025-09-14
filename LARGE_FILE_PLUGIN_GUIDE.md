# Large File Detection Plugin - Usage Guide

This guide explains how to use the Large File Detection Plugin that provides intelligent cleanup of large files with interactive selection and Git awareness.

## Overview

The Large File Plugin helps developers safely clean up disk space by:
- **Finding large files** above configurable size thresholds
- **Smart risk assessment** to protect important files
- **Git integration** to avoid deleting tracked files
- **Interactive UI** for safe file selection
- **Multiple filtering options** by age, type, and patterns

## Quick Start

### Basic Usage

```bash
# Find and interactively select large files over 100MB
./sweep --large-files /path/to/project

# Use custom size threshold
./sweep --large-files --size-threshold 500MB /path/to/project

# Include files older than 30 days only
./sweep --large-files --older-than 30 /path/to/project

# Include git-tracked files in scan (use with caution)
./sweep --large-files --include-git-tracked /path/to/project
```

### Interactive UI Controls

When the UI opens, use these controls:

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate up/down |
| `Space` | Toggle file selection |
| `Enter` | Confirm selection and proceed |
| `a` | Toggle all files |
| `s` | Cycle sort order (Size → Age → Risk → Name) |
| `h`/`?` | Toggle help screen |
| `q`/`Esc` | Cancel and exit |
| `PgUp`/`PgDn` | Page navigation |
| `Home`/`End` | Jump to first/last |

## Configuration Options

### Size Thresholds

The `--size-threshold` flag accepts human-readable formats:

```bash
--size-threshold 100MB    # 100 megabytes
--size-threshold 1.5GB    # 1.5 gigabytes
--size-threshold 2TB      # 2 terabytes
--size-threshold 500KB    # 500 kilobytes
```

### Age Filtering

Use `--older-than` to only scan files older than specified days:

```bash
--older-than 7     # Files not accessed in 7 days
--older-than 30    # Files not accessed in 30 days
--older-than 365   # Files not accessed in 1 year
```

### Git Integration

By default, git-tracked files are marked as **Critical Risk** to prevent accidental deletion. Use `--include-git-tracked` to include them in cleanup candidates (use with extreme caution).

## Risk Level System

The plugin uses a 5-level risk assessment:

### 🟢 Safe (Green)
- Files matching .gitignore patterns
- Log files (*.log)
- Archive files (*.zip, *.tar.gz)
- Files in ignored directories

### 🟡 Low (Yellow)
- Test data files (fixture*, test-data*, mock*)
- Files older than 30 days
- Media files not recently accessed

### 🟣 Medium (Magenta)
- Source code files
- Files modified in last week
- Unknown file types

### 🔴 High (Red)
- Database files (*.db, *.sqlite)
- Configuration files (*.json, *.yaml)
- Files modified in last 3 days
- Binary executables

### 🔴 Critical (Light Red)
- Git-tracked files (when include-git-tracked=false)
- Protected files (.env, .key, credentials*)
- Certificate files (*.pem, *.crt)
- Any file matching protected patterns

## Protected File Patterns

These files are automatically marked as **Critical Risk**:

```
.env, .env.*          # Environment variables
*.db, *.sqlite*       # Databases
*.key, *.pem, *.crt  # Certificates and keys
*.p12                # Certificate bundles
credentials*          # Credential files
secrets*             # Secret files
```

## Test Data Patterns

These files are marked as **Low Risk**:

```
test-data*, test_data*   # Test data files
fixture*, sample*       # Sample/fixture files
mock*                   # Mock data
*.test.*, *_test.*      # Test files
*.spec.*, *_spec.*      # Spec files
```

## Examples

### Clean Development Project

```bash
# Safe cleanup of a development project
./sweep --large-files --size-threshold 50MB --older-than 7 ~/my-project
```

This finds files:
- Larger than 50MB
- Not accessed in 7 days
- With git-tracked files protected
- Interactive selection enabled

### Aggressive Cleanup (Advanced)

```bash
# More aggressive cleanup including git files
./sweep --large-files --size-threshold 10MB --include-git-tracked ~/my-project
```

⚠️ **Warning**: This includes git-tracked files. Use extreme caution!

### Archive Directory Cleanup

```bash
# Clean old archive/download directories
./sweep --large-files --size-threshold 100MB --older-than 90 ~/Downloads
```

Perfect for cleaning download folders of old large files.

## Understanding the UI

### Main Screen Layout

```
┌─ Sweep Large File Cleanup ──────────────────────────────────────┐
│ Large Files - Selected: 3/15 (2.1 GB) - Sort: Size ↓ - Press h │
├──────────────────────────────────────────────────────────────────┤
│ Files                                                            │
│ ► ☑  1.2 GB     High  /path/to/large-file.bin                  │
│   ☐   800 MB   Medium /path/to/another-file.zip                │
│   ☑   500 MB     Low  /path/to/test-data.json                  │
│   ☐   200 MB Critical /path/to/.env                            │
├──────────────────────────────────────────────────────────────────┤
│        Space: Toggle | Enter: Confirm | a: Toggle All           │
└──────────────────────────────────────────────────────────────────┘
```

### Status Information

- **Selected**: Number of files selected and total size
- **Sort**: Current sort order (Size ↓, Age, Risk, Name)
- **►**: Current selection indicator
- **☑/☐**: Checkbox indicating if file is selected
- **Color coding**: Risk level colors (Green=Safe, Red=Critical, etc.)

### File Information Columns

1. **Size**: Human-readable file size (1.2 GB, 800 MB, etc.)
2. **Risk**: Risk level assessment
3. **Path**: Full path to the file

## Best Practices

### 1. Start Conservative
Begin with higher size thresholds and longer age filters:

```bash
./sweep --large-files --size-threshold 500MB --older-than 30
```

### 2. Review Risk Levels
Always check risk levels in the UI:
- Avoid deleting **Critical** and **High** risk files
- Be cautious with **Medium** risk files
- **Low** and **Safe** files are generally okay to delete

### 3. Use Git Protection
Keep git protection enabled (default) unless you absolutely need to clean tracked files.

### 4. Test First
Try the tool on a non-critical directory first to understand its behavior.

### 5. Regular Maintenance
Run periodic cleanups with consistent settings:

```bash
# Weekly cleanup script
./sweep --large-files --size-threshold 200MB --older-than 14 ~/projects
```

## Troubleshooting

### No Files Found
- Lower the `--size-threshold`
- Reduce `--older-than` days
- Check that path exists and is readable
- Verify files actually exist above threshold

### Permission Errors
- Ensure read permissions on target directory
- Run with appropriate user permissions
- Some system directories may be protected

### UI Doesn't Appear
- Verify terminal supports interactive mode
- Check that stdout is not redirected
- Ensure terminal has sufficient size (80x24 minimum)

### Git Repository Not Detected
- Ensure `.git` directory exists in project root
- Check that git repository is valid
- Try running from repository root directory

## Advanced Usage

### Combining with Other Tools

```bash
# Generate report without interactive UI (for scripts)
./sweep --large-files --size-threshold 100MB /path > large_files_report.txt

# Pipe to other tools for analysis
./sweep --large-files /path | grep "Critical" | wc -l
```

### Integration with Build Scripts

```bash
# Clean build artifacts after development
./sweep --large-files --size-threshold 50MB --older-than 1 ./target ./build
```

### Batch Processing Multiple Projects

```bash
# Clean all projects in a directory
for project in ~/projects/*/; do
    echo "Cleaning $project"
    ./sweep --large-files --size-threshold 100MB --older-than 14 "$project"
done
```

## Technical Details

### Performance
- Uses parallel scanning with rayon for speed
- Caches git repository information
- Streams results to minimize memory usage
- Optimized for directories with 100,000+ files

### Safety Features
- No files are deleted without explicit user confirmation
- Comprehensive risk assessment prevents accidents
- Git integration protects version-controlled files
- Pattern matching protects sensitive files

### File Type Detection
The plugin recognizes 20+ file extensions across categories:
- **Source**: .rs, .py, .js, .java, .c, .cpp, etc.
- **Database**: .db, .sqlite, .sql, etc.
- **Media**: .jpg, .mp4, .mp3, .png, etc.
- **Archive**: .zip, .tar, .gz, .rar, etc.
- **Config**: .json, .yaml, .toml, .ini, etc.

## Support

For issues, feature requests, or questions:
1. Check existing files match expected patterns
2. Verify command-line flags are correct
3. Test with verbose output if available
4. Report bugs with sample commands and expected vs. actual behavior

---

**Remember**: Always review selections carefully before confirming deletions. This tool helps identify candidates but the final decision is yours!