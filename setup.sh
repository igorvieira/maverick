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

HERMES_SKILLS_DIR="$HOME/.hermes/skills/autonomous-ai-agents"
MAVERICK_CONFIG_DIR="$HOME/.config/maverick"
LOCAL_BIN="$HOME/.local/bin"

print_usage() {
    cat <<EOF
Usage: ./setup.sh [claude|codex|all|hermes]
       ./setup.sh project /path/to/repo [--pack <id>]...

Packs: go, typescript (Node.js backend + frontend), rust, python, rails, elixir.
Without --pack, packs are detected from the repo's files (go.mod, package.json,
Cargo.toml, pyproject.toml, Gemfile + config/application.rb, mix.exs).

Targets:
  claude   Install Claude Code MCP configuration
  codex    Install Codex Maverick skill/templates
  all      Install both Claude Code and Codex assets
  hermes   Install the Hermes maverick-dispatch skill, maverick-dispatch/maverick-report
           into ~/.local/bin and a dispatch.json registry (never overwritten)
  project  Install the Maverick workflow into a repository's .claude/:
             - skills/maverick + commands (thin /maverick entry point, maverick-single,
               review-resolver, senior-*)
             - each --pack <name>: agents copied flat into .claude/agents/ and the pack
               JSON manifest into .claude/maverick/packs/<name>.json
             - shared backend principles into .claude/maverick/principles/
             - the project adapter template into .claude/maverick/project.md
               (never overwrites an existing adapter)

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
    if ! read -r -p "Selection [1-3]: " selection; then
        echo "claude"
        return
    fi

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

    # Claude Code loads MCP servers from ~/.claude.json (user scope), never from settings.json,
    # so register them through the CLI instead of editing JSON by hand.
    GLOBAL_JSON="$SCRIPT_DIR/claude/mcp-servers/global.json"
    for name in $(jq -r '.mcpServers | keys[]' "$GLOBAL_JSON"); do
        if claude mcp get "$name" &> /dev/null; then
            echo "  $name already configured, skipping"
            continue
        fi
        server=$(jq -c --arg name "$name" '.mcpServers[$name]' "$GLOBAL_JSON")
        if claude mcp add-json --scope user "$name" "$server"; then
            echo "  $name added (user scope)"
        else
            echo "  failed to add $name; run: claude mcp add-json --scope user $name '$server'" >&2
        fi
    done

    # Older versions of this script wrote the servers into settings.json, where they are ignored.
    if [ -f "$CLAUDE_SETTINGS_FILE" ] && jq -e '.mcpServers' "$CLAUDE_SETTINGS_FILE" &> /dev/null; then
        CLEANED=$(jq --slurpfile catalog "$GLOBAL_JSON" '
            .mcpServers |= with_entries(select(.key as $k | $catalog[0].mcpServers | has($k) | not))
            | if .mcpServers == {} then del(.mcpServers) else . end' "$CLAUDE_SETTINGS_FILE")
        echo "$CLEANED" > "$CLAUDE_SETTINGS_FILE"
        LEFTOVER=$(jq -r '(.mcpServers // {}) | keys | join(", ")' "$CLAUDE_SETTINGS_FILE")
        if [ -n "$LEFTOVER" ]; then
            echo "  note: settings.json still lists MCPs Claude Code ignores: $LEFTOVER"
            echo "        move them with: claude mcp add-json --scope user <name> '<json>'"
        fi
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
            if [ ! -f "$CLAUDE_SETTINGS_FILE" ]; then
                mkdir -p "$CLAUDE_DIR"
                echo '{}' > "$CLAUDE_SETTINGS_FILE"
            fi
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
    echo "To pick per-project MCPs (Claude, Codex, Grok):"
    echo "  cargo run -p maverick-cli -- mcp"
    echo ""
    echo "To use the CLAUDE.md template:"
    echo "  cp $SCRIPT_DIR/claude/templates/linear-figma.md /path/to/project/CLAUDE.md"
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

install_hermes() {
    echo ""
    echo "Hermes Setup"
    echo "============"

    if ! command -v hermes &> /dev/null; then
        echo "Hermes Agent not found. Install it first: https://github.com/NousResearch/hermes-agent" >&2
        exit 1
    fi

    echo "Hermes found"

    mkdir -p "$HERMES_SKILLS_DIR" "$LOCAL_BIN" "$MAVERICK_CONFIG_DIR"

    rm -rf "$HERMES_SKILLS_DIR/maverick-dispatch"
    cp -R "$SCRIPT_DIR/hermes/skills/maverick-dispatch" "$HERMES_SKILLS_DIR/maverick-dispatch"
    echo "Hermes skill installed: $HERMES_SKILLS_DIR/maverick-dispatch"

    local tool
    for tool in maverick-dispatch maverick-report; do
        install -m 755 "$SCRIPT_DIR/hermes/bin/$tool" "$LOCAL_BIN/$tool"
        echo "Installed: $LOCAL_BIN/$tool"
    done

    if [ ! -f "$MAVERICK_CONFIG_DIR/dispatch.json" ]; then
        cp "$SCRIPT_DIR/hermes/dispatch.json.example" "$MAVERICK_CONFIG_DIR/dispatch.json"
        echo "Registry template installed: $MAVERICK_CONFIG_DIR/dispatch.json"
        echo ">> Add your repos there (name -> path, base branch)."
    else
        echo "Existing registry preserved: $MAVERICK_CONFIG_DIR/dispatch.json"
    fi

    echo ""
    echo "Hermes config is not edited automatically. Add to ~/.hermes/config.yaml:"
    echo ""
    echo "  discord:"
    echo "    free_response_channels: [\"<CHANNEL_ID>\"]"
    echo "    free_response_auto_thread: true"
    echo "    allowed_channels: [\"<CHANNEL_ID>\"]"
    echo "    allow_mentions: { replied_user: false }"
    echo "    channel_prompts:"
    echo "      \"<CHANNEL_ID>\": |"
    sed 's/^/        /' "$SCRIPT_DIR/hermes/channel-prompt.md"
    echo ""
    echo "Put the channel webhook in $MAVERICK_CONFIG_DIR/secrets.env (chmod 600):"
    echo "  MAVERICK_DISCORD_WEBHOOK=https://discord.com/api/webhooks/..."
    echo ""
    echo "Hermes setup complete."
}

# Prints the IDs of packs whose detection rules match the project (needs jq).
detect_packs() {
    local project_path="$1" manifest found=()
    if ! command -v jq &> /dev/null; then
        echo "jq not found: pass --pack explicitly to install language packs." >&2
        return
    fi
    for manifest in "$SCRIPT_DIR"/claude/agents/*/pack.json; do
        local all_ok=true any_ok=true file
        while IFS= read -r file; do
            [ -z "$file" ] && continue
            [ -f "$project_path/$file" ] || all_ok=false
        done < <(jq -r '.detection.files[]?' "$manifest")
        if [ "$(jq '.detection.any_files | length' "$manifest")" -gt 0 ]; then
            any_ok=false
            while IFS= read -r file; do
                [ -f "$project_path/$file" ] && any_ok=true
            done < <(jq -r '.detection.any_files[]' "$manifest")
        fi
        if [ "$(jq '(.detection.files | length) + (.detection.any_files | length)' "$manifest")" -eq 0 ]; then
            continue
        fi
        if $all_ok && $any_ok; then
            found+=("$(jq -r '.id' "$manifest")")
        fi
    done
    echo "${found[*]}"
}

install_project() {
    local project_path="."
    if [ $# -gt 0 ] && [[ "$1" != --* ]]; then
        project_path="$1"
        shift
    fi

    echo ""
    echo "Maverick Project Install"
    echo "========================"

    if [ -z "$project_path" ] || [ ! -d "$project_path" ]; then
        echo "Project path not found: '$project_path'" >&2
        echo "Usage: ./setup.sh project /path/to/repo [--pack <id>]..." >&2
        exit 1
    fi

    local packs=()
    while [ $# -gt 0 ]; do
        case "$1" in
            --pack)
                if [ -z "${2:-}" ]; then
                    echo "--pack requires a value (e.g., --pack go)" >&2
                    exit 1
                fi
                packs+=("$2")
                shift 2
                ;;
            *)
                echo "Unknown option: $1" >&2
                exit 1
                ;;
        esac
    done

    if [ ${#packs[@]} -eq 0 ]; then
        local detected
        detected=$(detect_packs "$project_path")
        if [ -n "$detected" ]; then
            # shellcheck disable=SC2206
            packs=($detected)
            echo "Detected packs: ${packs[*]} (pass --pack to choose explicitly)"
        else
            echo "No language pack detected; installing the workflow only."
        fi
    fi

    local target="$project_path/.claude"
    mkdir -p "$target/commands" "$target/skills/maverick" "$target/agents" "$target/maverick/packs"

    cp "$SCRIPT_DIR"/claude/commands/*.md "$target/commands/"
    echo "Commands installed: $target/commands/"

    cp "$SCRIPT_DIR/claude/skills/maverick/SKILL.md" "$target/skills/maverick/SKILL.md"
    echo "Maverick skill installed: $target/skills/maverick/SKILL.md"

    local pack
    for pack in ${packs[@]+"${packs[@]}"}; do
        if [[ ! "$pack" =~ ^[a-z][a-z0-9]*([-_][a-z0-9]+)*$ ]]; then
            echo "Invalid pack ID: '$pack'" >&2
            exit 1
        fi
        local pack_dir="$SCRIPT_DIR/claude/agents/$pack"
        if [ ! -d "$pack_dir" ]; then
            echo "Unknown pack: '$pack'. Available packs:" >&2
            ls "$SCRIPT_DIR/claude/agents" >&2
            exit 1
        fi

        if [ ! -f "$pack_dir/pack.json" ]; then
            echo "Missing structured manifest: $pack_dir/pack.json" >&2
            exit 1
        fi
        cp "$pack_dir/pack.json" "$target/maverick/packs/$pack.json"

        local agent_file
        for agent_file in "$pack_dir"/*.md; do
            [ "$(basename "$agent_file")" = "pack.md" ] && continue
            cp "$agent_file" "$target/agents/"
        done

        if [ -f "$pack_dir/pack.md" ]; then
            cp "$pack_dir/pack.md" "$target/maverick/packs/$pack.md"
        fi
        echo "Pack '$pack' installed: agents in $target/agents/, manifest in $target/maverick/packs/$pack.json"
    done

    if [ ${#packs[@]} -gt 0 ]; then
        mkdir -p "$target/maverick/principles"
        cp "$SCRIPT_DIR"/claude/principles/*.md "$target/maverick/principles/"
        echo "Shared principles installed: $target/maverick/principles/"
    fi

    # Personal knowledge (optional, git-ignored in Maverick): linked, never copied.
    local knowledge_dir="${MAVERICK_KNOWLEDGE:-$SCRIPT_DIR/.knowledge}"
    if [ -f "$knowledge_dir/index.json" ]; then
        if [ -e "$target/maverick/knowledge" ] && [ ! -L "$target/maverick/knowledge" ]; then
            echo "Existing knowledge directory preserved: $target/maverick/knowledge"
        else
            ln -sfn "$knowledge_dir" "$target/maverick/knowledge"
            echo "Knowledge linked: $target/maverick/knowledge -> $knowledge_dir"
            echo ">> If .claude/ is committed in this repo, ignore .claude/maverick/knowledge"
        fi
    fi

    if [ ! -f "$target/maverick/project.md" ]; then
        cp "$SCRIPT_DIR/claude/templates/maverick-project.md" "$target/maverick/project.md"
        echo "Adapter template installed: $target/maverick/project.md"
        echo ">> Fill it with YOUR project's facts (commands, layout, conventions) — it is the"
        echo ">> only place project-specific knowledge should live. Never put secrets in it."
    else
        echo "Existing adapter preserved: $target/maverick/project.md"
    fi

    echo ""
    echo "Project install complete."
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
    hermes)
        install_hermes
        ;;
    all)
        install_claude
        install_codex
        ;;
    project)
        shift
        install_project "$@"
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
