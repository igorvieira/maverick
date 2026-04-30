#!/bin/bash

# Test suite for setup.sh
# Run: bash test_setup.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SETUP_SCRIPT="$SCRIPT_DIR/setup.sh"
TEST_DIR=$(mktemp -d)
PASSED=0
FAILED=0
TOTAL=0

cleanup() {
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

assert_eq() {
    local desc="$1" expected="$2" actual="$3"
    TOTAL=$((TOTAL + 1))
    if [ "$expected" = "$actual" ]; then
        echo -e "  ${GREEN}✓${NC} $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} $desc"
        echo -e "    expected: $expected"
        echo -e "    actual:   $actual"
        FAILED=$((FAILED + 1))
    fi
}

assert_contains() {
    local desc="$1" haystack="$2" needle="$3"
    TOTAL=$((TOTAL + 1))
    if echo "$haystack" | grep -q "$needle"; then
        echo -e "  ${GREEN}✓${NC} $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} $desc"
        echo -e "    '$needle' not found in output"
        FAILED=$((FAILED + 1))
    fi
}

assert_file_exists() {
    local desc="$1" file="$2"
    TOTAL=$((TOTAL + 1))
    if [ -f "$file" ]; then
        echo -e "  ${GREEN}✓${NC} $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} $desc"
        echo -e "    file not found: $file"
        FAILED=$((FAILED + 1))
    fi
}

assert_json_key() {
    local desc="$1" file="$2" key="$3"
    TOTAL=$((TOTAL + 1))
    if jq -e "$key" "$file" &>/dev/null; then
        echo -e "  ${GREEN}✓${NC} $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} $desc"
        echo -e "    key '$key' not found in $file"
        FAILED=$((FAILED + 1))
    fi
}

# ============================================================
echo ""
echo "=============================="
echo " Maverick Setup Test Suite"
echo "=============================="

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}1. Prerequisites${NC}"

assert_file_exists "setup.sh exists" "$SETUP_SCRIPT"

TOTAL=$((TOTAL + 1))
if [ -x "$SETUP_SCRIPT" ]; then
    echo -e "  ${GREEN}✓${NC} setup.sh is executable"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} setup.sh is executable (run: chmod +x setup.sh)"
    FAILED=$((FAILED + 1))
fi

TOTAL=$((TOTAL + 1))
if command -v jq &>/dev/null; then
    echo -e "  ${GREEN}✓${NC} jq is installed"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} jq is installed"
    FAILED=$((FAILED + 1))
fi

TOTAL=$((TOTAL + 1))
if command -v claude &>/dev/null; then
    echo -e "  ${GREEN}✓${NC} claude CLI is installed"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} claude CLI is installed"
    FAILED=$((FAILED + 1))
fi

TOTAL=$((TOTAL + 1))
if command -v uvx &>/dev/null; then
    echo -e "  ${GREEN}✓${NC} uvx is installed (needed for serena)"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} uvx is installed (needed for serena)"
    FAILED=$((FAILED + 1))
fi

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}2. Required files${NC}"

assert_file_exists "global.json exists" "$SCRIPT_DIR/mcp-servers/global.json"
assert_file_exists "project.json exists" "$SCRIPT_DIR/mcp-servers/project.json"
assert_file_exists "linear-figma template exists" "$SCRIPT_DIR/templates/linear-figma.md"
assert_file_exists "Codex AGENTS.md template exists" "$SCRIPT_DIR/codex/AGENTS.md"
assert_file_exists "Codex config example exists" "$SCRIPT_DIR/codex/config/config.toml.example"
assert_file_exists "Codex Maverick skill exists" "$SCRIPT_DIR/codex/skills/maverick/SKILL.md"
assert_file_exists "Codex Maverick skill metadata exists" "$SCRIPT_DIR/codex/skills/maverick/agents/openai.yaml"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}3. JSON validity${NC}"

for f in "$SCRIPT_DIR/mcp-servers/global.json" "$SCRIPT_DIR/mcp-servers/project.json"; do
    fname=$(basename "$f")
    TOTAL=$((TOTAL + 1))
    if jq empty "$f" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} $fname is valid JSON"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} $fname is valid JSON"
        FAILED=$((FAILED + 1))
    fi
done

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}4. Global MCP structure${NC}"

GLOBAL="$SCRIPT_DIR/mcp-servers/global.json"
assert_json_key "global.json has mcpServers key" "$GLOBAL" ".mcpServers"
assert_json_key "serena server is defined" "$GLOBAL" ".mcpServers.serena"
assert_json_key "serena has command field" "$GLOBAL" ".mcpServers.serena.command"
assert_json_key "serena has args field" "$GLOBAL" ".mcpServers.serena.args"

SERENA_CMD=$(jq -r '.mcpServers.serena.command' "$GLOBAL")
assert_eq "serena command is 'uvx'" "uvx" "$SERENA_CMD"

assert_json_key "figma server is defined" "$GLOBAL" ".mcpServers.figma"
assert_json_key "figma has url field" "$GLOBAL" ".mcpServers.figma.url"

FIGMA_TYPE=$(jq -r '.mcpServers.figma.type' "$GLOBAL")
assert_eq "figma type is 'http'" "http" "$FIGMA_TYPE"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}5. Project MCP structure${NC}"

PROJECT="$SCRIPT_DIR/mcp-servers/project.json"
assert_json_key "project.json has mcpServers key" "$PROJECT" ".mcpServers"

for server in figma linear github chrome-devtools basic-memory; do
    assert_json_key "$server server is defined" "$PROJECT" ".mcpServers.\"$server\""
done

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}6. Setup dry-run (isolated environment)${NC}"

# Create isolated HOME to test setup without touching real settings
FAKE_HOME="$TEST_DIR/home"
mkdir -p "$FAKE_HOME/.claude"

# Run Claude setup with overridden HOME
OUTPUT=$(HOME="$FAKE_HOME" bash "$SETUP_SCRIPT" claude 2>&1) || true
SETUP_EXIT=$?

SETTINGS_RESULT="$FAKE_HOME/.claude/settings.json"

if [ -f "$SETTINGS_RESULT" ]; then
    assert_file_exists "settings.json was created" "$SETTINGS_RESULT"
    assert_json_key "settings.json has mcpServers" "$SETTINGS_RESULT" ".mcpServers"
    assert_json_key "serena was added to settings" "$SETTINGS_RESULT" ".mcpServers.serena"

    RESULT_CMD=$(jq -r '.mcpServers.serena.command' "$SETTINGS_RESULT")
    assert_eq "serena command in settings is 'uvx'" "uvx" "$RESULT_CMD"

    assert_json_key "figma was added to settings" "$SETTINGS_RESULT" ".mcpServers.figma"
else
    TOTAL=$((TOTAL + 1))
    echo -e "  ${RED}✗${NC} setup.sh did not create settings.json (exit code: $SETUP_EXIT)"
    echo -e "    output: $(echo "$OUTPUT" | tail -5)"
    FAILED=$((FAILED + 1))
fi

assert_contains "setup output shows completion" "$OUTPUT" "Setup complete"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}7. Setup with existing settings (merge test)${NC}"

MERGE_HOME="$TEST_DIR/merge_home"
mkdir -p "$MERGE_HOME/.claude"

# Create a pre-existing settings.json with custom content
cat > "$MERGE_HOME/.claude/settings.json" <<'EOF'
{
  "mcpServers": {
    "my-custom-server": {
      "type": "stdio",
      "command": "my-tool"
    }
  },
  "enabledPlugins": {
    "some-plugin": true
  }
}
EOF

OUTPUT=$(HOME="$MERGE_HOME" bash "$SETUP_SCRIPT" claude 2>&1) || true

MERGED="$MERGE_HOME/.claude/settings.json"

if [ -f "$MERGED" ]; then
    assert_json_key "existing custom server preserved" "$MERGED" ".mcpServers.\"my-custom-server\""
    assert_json_key "serena was merged in" "$MERGED" ".mcpServers.serena"
    assert_json_key "enabledPlugins preserved" "$MERGED" ".enabledPlugins"

    assert_file_exists "backup was created" "$MERGE_HOME/.claude/settings.json.backup"
else
    TOTAL=$((TOTAL + 1))
    echo -e "  ${RED}✗${NC} merge test: settings.json missing after setup"
    FAILED=$((FAILED + 1))
fi

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}8. Codex setup dry-run (isolated environment)${NC}"

CODEX_HOME="$TEST_DIR/codex_home"
mkdir -p "$CODEX_HOME/bin"

cat > "$CODEX_HOME/bin/codex" <<'EOF'
#!/bin/sh
echo "codex mock"
EOF
chmod +x "$CODEX_HOME/bin/codex"

OUTPUT=$(HOME="$CODEX_HOME" PATH="$CODEX_HOME/bin:$PATH" bash "$SETUP_SCRIPT" codex 2>&1) || true
CODEX_EXIT=$?

if [ "$CODEX_EXIT" -eq 0 ]; then
    assert_file_exists "Codex config.toml was created" "$CODEX_HOME/.codex/config.toml"
    assert_file_exists "Maverick Codex skill was installed" "$CODEX_HOME/.codex/skills/maverick/SKILL.md"
    assert_contains "Codex setup output shows completion" "$OUTPUT" "Setup complete"
else
    TOTAL=$((TOTAL + 1))
    echo -e "  ${RED}✗${NC} setup.sh codex failed in isolated environment (exit code: $CODEX_EXIT)"
    echo -e "    output: $(echo "$OUTPUT" | tail -5)"
    FAILED=$((FAILED + 1))
fi

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}9. Interactive selector dry-run${NC}"

INTERACTIVE_HOME="$TEST_DIR/interactive_home"
mkdir -p "$INTERACTIVE_HOME/bin"

cat > "$INTERACTIVE_HOME/bin/codex" <<'EOF'
#!/bin/sh
echo "codex mock"
EOF
chmod +x "$INTERACTIVE_HOME/bin/codex"

OUTPUT=$(printf "2\n" | HOME="$INTERACTIVE_HOME" PATH="$INTERACTIVE_HOME/bin:$PATH" bash "$SETUP_SCRIPT" 2>&1) || true
INTERACTIVE_EXIT=$?

if [ "$INTERACTIVE_EXIT" -eq 0 ]; then
    assert_file_exists "Interactive Codex config.toml was created" "$INTERACTIVE_HOME/.codex/config.toml"
    assert_file_exists "Interactive Maverick Codex skill was installed" "$INTERACTIVE_HOME/.codex/skills/maverick/SKILL.md"
else
    TOTAL=$((TOTAL + 1))
    echo -e "  ${RED}✗${NC} setup.sh interactive selector failed (exit code: $INTERACTIVE_EXIT)"
    echo -e "    output: $(echo "$OUTPUT" | tail -5)"
    FAILED=$((FAILED + 1))
fi

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}10. Commands directory${NC}"

COMMANDS_DIR="$SCRIPT_DIR/commands"
TOTAL=$((TOTAL + 1))
if [ -d "$COMMANDS_DIR" ]; then
    echo -e "  ${GREEN}✓${NC} commands/ directory exists"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} commands/ directory exists"
    FAILED=$((FAILED + 1))
fi

for cmd in maverick.md senior-architect.md senior-backend.md senior-frontend.md senior-security.md senior-qa.md review-resolver.md maverick-single.md; do
    assert_file_exists "command: $cmd" "$COMMANDS_DIR/$cmd"
done

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}11. Current settings.json sync check${NC}"

REAL_SETTINGS="$HOME/.claude/settings.json"
if [ -f "$REAL_SETTINGS" ]; then
    TOTAL=$((TOTAL + 1))
    if jq -e '.mcpServers.serena' "$REAL_SETTINGS" &>/dev/null; then
        echo -e "  ${GREEN}✓${NC} serena is in your active settings.json"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} serena is NOT in your active ~/.claude/settings.json (run setup.sh to fix)"
        FAILED=$((FAILED + 1))
    fi
else
    TOTAL=$((TOTAL + 1))
    echo -e "  ${RED}✗${NC} ~/.claude/settings.json does not exist (run setup.sh to create)"
    FAILED=$((FAILED + 1))
fi

# ============================================================
echo ""
echo "=============================="
if [ "$FAILED" -eq 0 ]; then
    echo -e " ${GREEN}All $TOTAL tests passed${NC}"
else
    echo -e " ${GREEN}$PASSED passed${NC}, ${RED}$FAILED failed${NC} (of $TOTAL)"
fi
echo "=============================="
echo ""

exit $FAILED
