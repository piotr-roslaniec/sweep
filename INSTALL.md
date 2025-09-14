# Installation Guide for Sweep

Sweep is an intelligent disk cleanup tool that helps developers safely clean up large files and build artifacts with Git-aware protection.

## Quick Install (Recommended)

### Using Install Script

```bash
# Clone the repository
git clone https://github.com/piotr-roslaniec/sweep.git
cd sweep

# Run the install script
./install.sh
```

The install script will:
1. Build sweep in release mode
2. Try to install system-wide to `/usr/local/bin/sweep`
3. Fall back to user installation at `~/.local/bin/sweep` if no sudo access
4. Test the installation and display usage information

### Manual Installation

If you prefer to install manually:

```bash
# Build the release binary
cargo build --release

# Install system-wide (requires sudo)
sudo cp target/release/swp /usr/local/bin/sweep

# OR install for current user only
mkdir -p ~/.local/bin
cp target/release/swp ~/.local/bin/sweep

# Make sure ~/.local/bin is in your PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Verify Installation

After installation, verify sweep is working:

```bash
sweep --version
sweep --help
```

## Quick Start

Try the new large file detection plugin:

```bash
# Find large files in your projects directory
sweep --large-files ~/projects

# Use custom threshold and age filter
sweep --large-files --size-threshold 500MB --older-than 30 ~/downloads
```

## Uninstall

To remove sweep from your system:

```bash
./uninstall.sh
```

Or manually:

```bash
# Remove system installation
sudo rm /usr/local/bin/sweep

# Remove user installation
rm ~/.local/bin/sweep
```

## System Requirements

- **Rust**: 1.40.0 or later
- **OS**: Linux, macOS, or Windows
- **Terminal**: For interactive UI features
- **Git**: Optional but recommended for git-aware file protection

## Dependencies

Sweep uses these external dependencies:
- `crossterm`: Terminal manipulation
- `tui`: Terminal user interface
- `git2`: Git repository integration
- `rayon`: Parallel processing
- `walkdir`: Directory traversal

## Troubleshooting

### Installation Issues

**Permission denied during system install:**
- The installer will automatically fall back to user installation
- Make sure `~/.local/bin` is in your PATH

**PATH not updated:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"
source ~/.bashrc
```

**Rust not installed:**
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Runtime Issues

**UI doesn't appear:**
- Ensure terminal supports ANSI colors
- Check terminal size (minimum 80x24)
- Verify stdout is not redirected

**Git repository not detected:**
- Run from repository root directory
- Ensure `.git` directory exists
- Check git repository is valid

**No files found:**
- Lower `--size-threshold`
- Reduce `--older-than` days
- Check file permissions

## Features

### Large File Detection Plugin

- **Interactive UI**: Select files safely with visual feedback
- **Git Integration**: Protects tracked files automatically
- **Risk Assessment**: 5-level color-coded risk system
- **Smart Filtering**: File type detection and pattern matching
- **Parallel Scanning**: High-performance directory traversal

### Safety Features

- **Protected Patterns**: Automatic detection of sensitive files
- **Git Awareness**: Prevents accidental deletion of version-controlled files
- **Interactive Confirmation**: No files deleted without explicit user approval
- **Risk Level System**: Clear indicators for file deletion safety

### Command Line Options

```bash
# Enable large file detection
--large-files

# Set size threshold (supports KB, MB, GB, TB)
--size-threshold 100MB

# Age-based filtering
--older-than 30

# Include git-tracked files (use with caution)
--include-git-tracked

# Multiple directory scanning
sweep --large-files ~/projects ~/downloads ~/temp
```

## Development

To contribute or modify sweep:

```bash
# Clone and build
git clone https://github.com/piotr-roslaniec/sweep.git
cd sweep
cargo build

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Support

- **Documentation**: `./LARGE_FILE_PLUGIN_GUIDE.md`
- **Issues**: [GitHub Issues](https://github.com/piotr-roslaniec/sweep/issues)
- **Help**: `sweep --help`

---

Happy cleaning! 🧹