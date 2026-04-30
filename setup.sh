#!/bin/bash

# Maverick Setup Script
# Installs Claude Code and/or Codex configuration assets.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

CLAUDE_DIR="$HOME/.claude"
CLAUDE_SETTINGS_FILE="$CLAUDE_DIR/settings.json"

CODEX_DIR="$HOME/.codex"
CODEX_SKILLS_DIR="$CODEX_DIR/skills"
CODEX_CONFIG="$CODEX_DIR/config.toml"

print_usage() {
    cat <<EOF
Usage: ./setup.sh [claude|codex|all]

Targets:
  claude   Install Claude Code MCP configuration
  codex    Install Codex Maverick skill/templates
  all      Install both Claude Code and Codex assets

If no target is provided, setup runs interactively.
EOF
}

choose_target() {
    echo "Maverick Setup" >&2
    echo "==============" >&2
    echo "" >&2
    echo "Choose what to configure:" >&2
    echo "  1) Claude Code" >&2
    echo "  2) Codex" >&2
    echo "  3) Both" >&2
    echo "" >&2
    read -r -p "Selection [1-3]: " selection

    case "$selection" in
        1) echo "claude" ;;
        2) echo "codex" ;;
        3) echo "all" ;;
        *)
            echo "Invalid selection: $selection" >&2
            exit 1
            ;;
    esac
}

ensure_jq() {
    if command -v jq &> /dev/null; then
        return
    fi

    echo "Installing jq..."
    if command -v brew &> /dev/null; then
        brew install jq
    else
        echo "jq not found. Install manually: brew install jq" >&2
        exit 1
    fi
}

install_claude() {
    echo ""
    echo "Claude Code Setup"
    echo "================="

    if ! command -v claude &> /dev/null; then
        echo "Claude Code not found. Install it first: npm install -g @anthropic-ai/claude-code" >&2
        exit 1
    fi

    echo "Claude Code found"
    ensure_jq

    if [ -f "$CLAUDE_SETTINGS_FILE" ]; then
        cp "$CLAUDE_SETTINGS_FILE" "$CLAUDE_SETTINGS_FILE.backup"
        echo "Backup created: $CLAUDE_SETTINGS_FILE.backup"
    fi

    echo ""
    echo "Configuring global MCPs..."

    GLOBAL_MCPS=$(cat "$SCRIPT_DIR/mcp-servers/global.json")

    if [ -f "$CLAUDE_SETTINGS_FILE" ]; then
        EXISTING=$(cat "$CLAUDE_SETTINGS_FILE")
        MERGED=$(echo "$EXISTING" | jq --argjson new "$(echo "$GLOBAL_MCPS" | jq '.mcpServers')" '.mcpServers = (.mcpServers // {}) + $new')
        echo "$MERGED" > "$CLAUDE_SETTINGS_FILE"
    else
        mkdir -p "$CLAUDE_DIR"
        echo "$GLOBAL_MCPS" | jq '{mcpServers: .mcpServers}' > "$CLAUDE_SETTINGS_FILE"
    fi

    echo "Global MCPs configured"

    echo ""
    echo "Checking dependencies..."

    if ! command -v uvx &> /dev/null; then
        echo "Installing uv..."
        curl -LsSf https://astral.sh/uv/install.sh | sh
    fi

    echo "Dependencies verified"

    echo ""
    echo "Configuring notification sound..."

    if [[ "$OSTYPE" == "darwin"* ]]; then
        SOUND_FILE="/System/Library/Sounds/Funk.aiff"
        if [ -f "$SOUND_FILE" ]; then
            EXISTING=$(cat "$CLAUDE_SETTINGS_FILE")
            HAS_HOOKS=$(echo "$EXISTING" | jq 'has("hooks")')
            if [ "$HAS_HOOKS" = "false" ]; then
                MERGED=$(echo "$EXISTING" | jq '.hooks.Notification = [{"hooks": [{"type": "command", "command": "afplay /System/Library/Sounds/Funk.aiff &", "timeout": 5}]}]')
                echo "$MERGED" > "$CLAUDE_SETTINGS_FILE"
                echo "Sound notification configured (Funk)"
            else
                echo "Hooks already configured, skipping sound setup"
            fi
        else
            echo "System sound not found, skipping"
        fi
    else
        echo "Not macOS, skipping sound setup"
    fi

    echo ""
    echo "Claude Code setup complete."
    echo ""
    echo "Global MCPs installed:"
    echo "  - serena"
    echo "  - figma"
    echo ""
    echo "To add per-project MCPs, copy the content from:"
    echo "  $SCRIPT_DIR/mcp-servers/project.json"
    echo ""
    echo "To use the CLAUDE.md template:"
    echo "  cp $SCRIPT_DIR/templates/linear-figma.md /path/to/project/CLAUDE.md"
}

install_codex() {
    echo ""
    echo "Codex Setup"
    echo "==========="

    if ! command -v codex &> /dev/null; then
        echo "Codex CLI was not found. Install or open Codex before using this setup." >&2
        exit 1
    fi

    echo "Codex CLI found"

    mkdir -p "$CODEX_SKILLS_DIR"

    if [ -d "$SCRIPT_DIR/codex/skills/maverick" ]; then
        rm -rf "$CODEX_SKILLS_DIR/maverick"
        cp -R "$SCRIPT_DIR/codex/skills/maverick" "$CODEX_SKILLS_DIR/maverick"
        echo "Maverick Codex skill installed: $CODEX_SKILLS_DIR/maverick"
    else
        echo "Missing Codex skill directory: $SCRIPT_DIR/codex/skills/maverick" >&2
        exit 1
    fi

    if [ -f "$CODEX_CONFIG" ]; then
        cp "$CODEX_CONFIG" "$CODEX_CONFIG.backup"
        echo "Backup created: $CODEX_CONFIG.backup"
    else
        mkdir -p "$CODEX_DIR"
        touch "$CODEX_CONFIG"
        echo "Created config file: $CODEX_CONFIG"
    fi

    echo ""
    echo "MCP configuration was not merged automatically."
    echo "Review and copy the snippets you need from:"
    echo "  $SCRIPT_DIR/codex/config/config.toml.example"
    echo ""
    echo "Project template:"
    echo "  cp $SCRIPT_DIR/codex/AGENTS.md /path/to/project/AGENTS.md"
    echo ""
    echo "Codex setup complete."
}

TARGET="${1:-}"

if [ -z "$TARGET" ]; then
    TARGET=$(choose_target)
fi

case "$TARGET" in
    claude)
        install_claude
        ;;
    codex)
        install_codex
        ;;
    all)
        install_claude
        install_codex
        ;;
    -h|--help|help)
        print_usage
        ;;
    *)
        echo "Unknown target: $TARGET" >&2
        echo "" >&2
        print_usage >&2
        exit 1
        ;;
esac

echo ""
echo "Setup complete."
