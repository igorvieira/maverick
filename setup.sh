#!/bin/bash

# Claude Code Setup Script
# Installs MCPs and configures the environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="$HOME/.claude"
SETTINGS_FILE="$CLAUDE_DIR/settings.json"

echo "🚀 Claude Code Setup"
echo "===================="

# Check if Claude Code is installed
if ! command -v claude &> /dev/null; then
    echo "❌ Claude Code not found. Install it first: npm install -g @anthropic-ai/claude-code"
    exit 1
fi

echo "✅ Claude Code found"

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "📦 Installing jq..."
    if command -v brew &> /dev/null; then
        brew install jq
    else
        echo "❌ jq not found. Install manually: brew install jq"
        exit 1
    fi
fi

# Backup settings.json
if [ -f "$SETTINGS_FILE" ]; then
    cp "$SETTINGS_FILE" "$SETTINGS_FILE.backup"
    echo "📋 Backup created: $SETTINGS_FILE.backup"
fi

# Add global MCPs
echo ""
echo "📡 Configuring global MCPs..."

GLOBAL_MCPS=$(cat "$SCRIPT_DIR/mcp-servers/global.json")

if [ -f "$SETTINGS_FILE" ]; then
    # Merge global MCPs with existing settings
    EXISTING=$(cat "$SETTINGS_FILE")
    MERGED=$(echo "$EXISTING" | jq --argjson new "$(echo "$GLOBAL_MCPS" | jq '.mcpServers')" '.mcpServers = (.mcpServers // {}) + $new')
    echo "$MERGED" > "$SETTINGS_FILE"
else
    # Create new settings.json
    echo "$GLOBAL_MCPS" | jq '{mcpServers: .mcpServers}' > "$SETTINGS_FILE"
fi

echo "✅ Global MCPs configured"

# Install MCP dependencies
echo ""
echo "📦 Checking dependencies..."

# uvx (for serena and basic-memory)
if ! command -v uvx &> /dev/null; then
    echo "  Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
fi

echo "✅ Dependencies verified"

# Summary
echo ""
echo "🎉 Setup complete!"
echo ""
echo "Global MCPs installed:"
echo "  - serena"
echo ""
echo "To add per-project MCPs, copy the content from:"
echo "  $SCRIPT_DIR/mcp-servers/project.json"
echo ""
echo "To use the CLAUDE.md template:"
echo "  cp $SCRIPT_DIR/templates/linear-figma.md /path/to/project/CLAUDE.md"
