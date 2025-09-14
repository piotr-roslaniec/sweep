#!/bin/bash
set -e

echo "Building sweep in release mode..."
cargo build --release

echo "Installing sweep..."

# Try to install to /usr/local/bin first (system-wide)
if sudo cp target/release/swp /usr/local/bin/sweep 2>/dev/null; then
    INSTALL_LOCATION="/usr/local/bin/sweep"
    echo "✓ Installed to /usr/local/bin/sweep (system-wide)"
else
    # Fall back to user's home bin directory
    mkdir -p "$HOME/.local/bin"
    cp target/release/swp "$HOME/.local/bin/sweep"
    INSTALL_LOCATION="$HOME/.local/bin/sweep"
    echo "✓ Installed to $HOME/.local/bin/sweep (user-specific)"

    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo "⚠ Warning: $HOME/.local/bin is not in your PATH"
        echo "Add this line to your ~/.bashrc or ~/.zshrc:"
        echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo "Then run: source ~/.bashrc (or restart your terminal)"
        echo
    fi
fi

echo "Testing installation..."
if command -v sweep &> /dev/null; then
    echo "✓ sweep installed successfully!"
    sweep --version
    echo
    echo "=== Sweep - Intelligent Disk Cleanup Tool ==="
    echo "sweep has been installed with the new Large File Detection Plugin!"
    echo
    echo "=== Quick Start ==="
    echo "Basic usage examples:"
    echo "  sweep --large-files ~/projects          # Find large files with interactive UI"
    echo "  sweep --large-files --size-threshold 500MB ~/downloads"
    echo "  sweep --large-files --older-than 30 ~/temp"
    echo
    echo "=== Available Features ==="
    echo "  --large-files           Enable large file detection plugin"
    echo "  --size-threshold SIZE   Set minimum file size (100MB, 1.5GB, etc.)"
    echo "  --older-than DAYS       Only show files older than N days"
    echo "  --include-git-tracked   Include git-tracked files (use with caution)"
    echo
    echo "=== Interactive UI Controls ==="
    echo "  Space     Toggle file selection"
    echo "  Enter     Confirm selection"
    echo "  h/?       Show help"
    echo "  s         Cycle sort order"
    echo "  a         Toggle all files"
    echo "  q/Esc     Cancel"
    echo
    echo "=== Safety Features ==="
    echo "  • Git-aware file protection"
    echo "  • 5-level risk assessment (Safe/Low/Medium/High/Critical)"
    echo "  • Protected file pattern detection (.env, .key, credentials)"
    echo "  • Interactive confirmation before any deletions"
    echo
    echo "=== Documentation ==="
    echo "  Full documentation: ./LARGE_FILE_PLUGIN_GUIDE.md"
    echo "  Get help:           sweep --help"
    echo "  Check version:      sweep --version"
    echo
    echo "Happy cleaning! 🧹"
    echo
else
    echo "✗ Installation failed"
    exit 1
fi