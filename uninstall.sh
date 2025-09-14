#!/bin/bash
set -e

echo "Uninstalling sweep..."

REMOVED=false

# Check system-wide installation
if [ -f /usr/local/bin/sweep ]; then
    if sudo rm /usr/local/bin/sweep 2>/dev/null; then
        echo "✓ Removed system-wide installation (/usr/local/bin/sweep)"
        REMOVED=true
    else
        echo "⚠ Found sweep in /usr/local/bin but couldn't remove (permission denied)"
    fi
fi

# Check user-specific installation
if [ -f "$HOME/.local/bin/sweep" ]; then
    rm "$HOME/.local/bin/sweep"
    echo "✓ Removed user-specific installation ($HOME/.local/bin/sweep)"
    REMOVED=true
fi

if [ "$REMOVED" = false ]; then
    echo "No sweep installations found in standard locations"
fi

echo "Checking for any remaining sweep installations..."
if command -v sweep &> /dev/null; then
    echo "⚠ Warning: sweep is still available in PATH"
    echo "Location: $(which sweep)"
    echo "You may need to remove it manually or check other installation locations."
else
    echo "✓ sweep is completely removed from PATH"
fi

echo
echo "Uninstall complete! Thank you for using sweep 🧹"