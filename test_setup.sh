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

create_stub_bin() {
    local dir="$1"
    local name="$2"
    mkdir -p "$dir"
    cat > "$dir/$name" <<EOF
#!/bin/sh
echo "$name mock"
EOF
    chmod +x "$dir/$name"
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

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}2. Required files${NC}"

assert_file_exists "global.json exists" "$SCRIPT_DIR/claude/mcp-servers/global.json"
assert_file_exists "project.json exists" "$SCRIPT_DIR/claude/mcp-servers/project.json"
assert_file_exists "linear-figma template exists" "$SCRIPT_DIR/claude/templates/linear-figma.md"
assert_file_exists "maverick project adapter template exists" "$SCRIPT_DIR/claude/templates/maverick-project.md"
assert_file_exists "maverick skill exists" "$SCRIPT_DIR/claude/skills/maverick/SKILL.md"
assert_file_exists "go pack manifest exists" "$SCRIPT_DIR/claude/agents/go/pack.md"
assert_file_exists "go pack json manifest exists" "$SCRIPT_DIR/claude/agents/go/pack.json"
assert_file_exists "typescript pack json manifest exists" "$SCRIPT_DIR/claude/agents/typescript/pack.json"
assert_file_exists "go pack implementer agent exists" "$SCRIPT_DIR/claude/agents/go/go-implementer.md"
for pack in rust python rails elixir; do
    assert_file_exists "$pack pack json manifest exists" "$SCRIPT_DIR/claude/agents/$pack/pack.json"
done
for principle in README domain application-flow adapters eventing architecture red-team; do
    assert_file_exists "shared principle exists: $principle" "$SCRIPT_DIR/claude/principles/$principle.md"
done
assert_file_exists "one-step installer exists" "$SCRIPT_DIR/install.sh"
assert_file_exists "Codex AGENTS.md template exists" "$SCRIPT_DIR/codex/AGENTS.md"
assert_file_exists "Codex config example exists" "$SCRIPT_DIR/codex/config/config.toml.example"
assert_file_exists "Codex Maverick skill exists" "$SCRIPT_DIR/codex/skills/maverick/SKILL.md"
assert_file_exists "Codex Maverick skill metadata exists" "$SCRIPT_DIR/codex/skills/maverick/agents/openai.yaml"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}3. JSON validity${NC}"

for f in "$SCRIPT_DIR/claude/mcp-servers/global.json" "$SCRIPT_DIR/claude/mcp-servers/project.json" \
         "$SCRIPT_DIR"/claude/agents/*/pack.json; do
    fname=${f#"$SCRIPT_DIR"/}
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

GLOBAL="$SCRIPT_DIR/claude/mcp-servers/global.json"
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

PROJECT="$SCRIPT_DIR/claude/mcp-servers/project.json"
assert_json_key "project.json has mcpServers key" "$PROJECT" ".mcpServers"

for server in figma linear github chrome-devtools basic-memory; do
    assert_json_key "$server server is defined" "$PROJECT" ".mcpServers.\"$server\""
done

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}6. Setup dry-run (isolated environment)${NC}"

# claude stub that records every call; `mcp get` succeeds only for names in $HOME/claude-existing.
create_claude_stub() {
    local dir="$1"
    mkdir -p "$dir"
    cat > "$dir/claude" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$HOME/claude-calls.log"
if [ "$1 $2" = "mcp get" ]; then
    grep -qx "$3" "$HOME/claude-existing" 2>/dev/null
    exit $?
fi
exit 0
EOF
    chmod +x "$dir/claude"
}

# Create isolated HOME to test setup without touching real settings
FAKE_HOME="$TEST_DIR/home"
mkdir -p "$FAKE_HOME/.claude"
create_claude_stub "$FAKE_HOME/bin"
create_stub_bin "$FAKE_HOME/bin" "uvx"

# Run Claude setup with overridden HOME
OUTPUT=$(HOME="$FAKE_HOME" PATH="$FAKE_HOME/bin:$PATH" bash "$SETUP_SCRIPT" claude 2>&1) || true
CALLS=$(cat "$FAKE_HOME/claude-calls.log" 2>/dev/null)

assert_contains "serena added via claude mcp add-json (user scope)" "$CALLS" "mcp add-json --scope user serena"
assert_contains "figma added via claude mcp add-json (user scope)" "$CALLS" "mcp add-json --scope user figma"
assert_contains "serena JSON carries its uvx command" "$CALLS" '"command":"uvx"'

MCP_IN_SETTINGS=$(jq -r 'has("mcpServers")' "$FAKE_HOME/.claude/settings.json" 2>/dev/null || echo "false")
assert_eq "no mcpServers written to settings.json" "false" "$MCP_IN_SETTINGS"

assert_contains "setup output shows completion" "$OUTPUT" "Setup complete"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}7. Re-run with legacy settings.json MCPs (migration test)${NC}"

MERGE_HOME="$TEST_DIR/merge_home"
mkdir -p "$MERGE_HOME/.claude"
create_claude_stub "$MERGE_HOME/bin"
create_stub_bin "$MERGE_HOME/bin" "uvx"
echo "serena" > "$MERGE_HOME/claude-existing"

# settings.json as written by the old setup.sh, plus a user's own entries
cat > "$MERGE_HOME/.claude/settings.json" <<'EOF'
{
  "mcpServers": {
    "my-custom-server": {
      "type": "stdio",
      "command": "my-tool"
    },
    "figma": {
      "type": "http",
      "url": "https://mcp.figma.com/mcp"
    }
  },
  "enabledPlugins": {
    "some-plugin": true
  }
}
EOF

OUTPUT=$(HOME="$MERGE_HOME" PATH="$MERGE_HOME/bin:$PATH" bash "$SETUP_SCRIPT" claude 2>&1) || true
CALLS=$(cat "$MERGE_HOME/claude-calls.log" 2>/dev/null)
MERGED="$MERGE_HOME/.claude/settings.json"

assert_contains "already configured serena is skipped" "$OUTPUT" "serena already configured"
SERENA_ADDS=$(grep -c "add-json --scope user serena" "$MERGE_HOME/claude-calls.log" 2>/dev/null || true)
assert_eq "serena is not re-added" "0" "$SERENA_ADDS"
assert_contains "figma is registered with claude" "$CALLS" "mcp add-json --scope user figma"

FIGMA_LEFT=$(jq -r '.mcpServers | has("figma")' "$MERGED")
assert_eq "stale figma removed from settings.json" "false" "$FIGMA_LEFT"
assert_json_key "user's own server preserved" "$MERGED" ".mcpServers.\"my-custom-server\""
assert_json_key "enabledPlugins preserved" "$MERGED" ".enabledPlugins"
assert_contains "leftover ignored MCPs are reported" "$OUTPUT" "my-custom-server"
assert_file_exists "backup was created" "$MERGE_HOME/.claude/settings.json.backup"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}8. Codex setup dry-run (isolated environment)${NC}"

CODEX_HOME="$TEST_DIR/codex_home"
create_stub_bin "$CODEX_HOME/bin" "codex"

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

DEFAULT_HOME="$TEST_DIR/default_home"
mkdir -p "$DEFAULT_HOME/.claude"
create_claude_stub "$DEFAULT_HOME/bin"
create_stub_bin "$DEFAULT_HOME/bin" "uvx"

OUTPUT=$(HOME="$DEFAULT_HOME" PATH="$DEFAULT_HOME/bin:$PATH" bash "$SETUP_SCRIPT" < /dev/null 2>&1) || true
DEFAULT_EXIT=$?

if [ "$DEFAULT_EXIT" -eq 0 ]; then
    assert_contains "Non-interactive default installed serena" "$(cat "$DEFAULT_HOME/claude-calls.log" 2>/dev/null)" "mcp add-json --scope user serena"
else
    TOTAL=$((TOTAL + 1))
    echo -e "  ${RED}✗${NC} setup.sh non-interactive default failed (exit code: $DEFAULT_EXIT)"
    echo -e "    output: $(echo "$OUTPUT" | tail -5)"
    FAILED=$((FAILED + 1))
fi

INTERACTIVE_HOME="$TEST_DIR/interactive_home"
create_stub_bin "$INTERACTIVE_HOME/bin" "codex"

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

COMMANDS_DIR="$SCRIPT_DIR/claude/commands"
TOTAL=$((TOTAL + 1))
if [ -d "$COMMANDS_DIR" ]; then
    echo -e "  ${GREEN}✓${NC} claude/commands/ directory exists"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} claude/commands/ directory exists"
    FAILED=$((FAILED + 1))
fi

for cmd in maverick.md senior-architect.md senior-frontend.md senior-security.md senior-qa.md review-resolver.md maverick-single.md; do
    assert_file_exists "command: $cmd" "$COMMANDS_DIR/$cmd"
done

TOTAL=$((TOTAL + 1))
if [ ! -f "$COMMANDS_DIR/senior-backend.md" ]; then
    echo -e "  ${GREEN}✓${NC} senior-backend.md removed (replaced by the go pack)"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} senior-backend.md should not exist (replaced by the go pack)"
    FAILED=$((FAILED + 1))
fi

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}11. Project install (isolated environment)${NC}"

PROJECT_TARGET="$TEST_DIR/sample-repo"
mkdir -p "$PROJECT_TARGET"

OUTPUT=$(bash "$SETUP_SCRIPT" project "$PROJECT_TARGET" --pack go 2>&1) || true
PROJECT_EXIT=$?

if [ "$PROJECT_EXIT" -eq 0 ]; then
    assert_file_exists "maverick skill installed in project" "$PROJECT_TARGET/.claude/skills/maverick/SKILL.md"
    assert_file_exists "maverick command installed in project" "$PROJECT_TARGET/.claude/commands/maverick.md"
    assert_file_exists "go agent installed flat in project agents/" "$PROJECT_TARGET/.claude/agents/go-implementer.md"
    assert_file_exists "go pack manifest installed" "$PROJECT_TARGET/.claude/maverick/packs/go.md"
    assert_file_exists "go structured pack manifest installed" "$PROJECT_TARGET/.claude/maverick/packs/go.json"
    assert_file_exists "adapter template installed" "$PROJECT_TARGET/.claude/maverick/project.md"

    TOTAL=$((TOTAL + 1))
    if [ ! -f "$PROJECT_TARGET/.claude/agents/pack.md" ]; then
        echo -e "  ${GREEN}✓${NC} pack.md not copied into agents/ (manifest only in maverick/packs/)"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} pack.md must not be copied into agents/"
        FAILED=$((FAILED + 1))
    fi

    # Re-run must preserve an edited adapter
    echo "customized adapter" > "$PROJECT_TARGET/.claude/maverick/project.md"
    OUTPUT=$(bash "$SETUP_SCRIPT" project "$PROJECT_TARGET" --pack go 2>&1) || true
    ADAPTER_CONTENT=$(cat "$PROJECT_TARGET/.claude/maverick/project.md")
    assert_eq "re-run preserves existing adapter" "customized adapter" "$ADAPTER_CONTENT"
else
    TOTAL=$((TOTAL + 1))
    echo -e "  ${RED}✗${NC} setup.sh project failed (exit code: $PROJECT_EXIT)"
    echo -e "    output: $(echo "$OUTPUT" | tail -5)"
    FAILED=$((FAILED + 1))
fi

# Personal knowledge is linked (never copied) when an index exists
KNOWLEDGE_DIR="$TEST_DIR/knowledge"
mkdir -p "$KNOWLEDGE_DIR"
printf '%s\n' '{"schema_version":1,"entries":[]}' > "$KNOWLEDGE_DIR/index.json"
KNOWLEDGE_TARGET="$TEST_DIR/knowledge-repo"
mkdir -p "$KNOWLEDGE_TARGET"
MAVERICK_KNOWLEDGE="$KNOWLEDGE_DIR" bash "$SETUP_SCRIPT" project "$KNOWLEDGE_TARGET" >/dev/null 2>&1 || true
TOTAL=$((TOTAL + 1))
if [ -L "$KNOWLEDGE_TARGET/.claude/maverick/knowledge" ] && [ "$(readlink "$KNOWLEDGE_TARGET/.claude/maverick/knowledge")" = "$KNOWLEDGE_DIR" ]; then
    echo -e "  ${GREEN}✓${NC} knowledge directory linked into project"
    PASSED=$((PASSED + 1))
else
    echo -e "  ${RED}✗${NC} knowledge directory not linked"
    FAILED=$((FAILED + 1))
fi
if [ ! -f "$SCRIPT_DIR/.knowledge/index.json" ]; then
    TOTAL=$((TOTAL + 1))
    if [ ! -e "$PROJECT_TARGET/.claude/maverick/knowledge" ]; then
        echo -e "  ${GREEN}✓${NC} no knowledge link without an index"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}✗${NC} knowledge linked without an index"
        FAILED=$((FAILED + 1))
    fi
fi

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}12. Structured packs and multi-pack installation${NC}"

MULTI_TARGET="$TEST_DIR/multi-pack-repo"
mkdir -p "$MULTI_TARGET/.claude/maverick"
printf '%s\n' '{"schema_version":1,"packs":[],"commands":{}}' > "$MULTI_TARGET/.claude/maverick/project.json"
STRUCTURED_BEFORE=$(cat "$MULTI_TARGET/.claude/maverick/project.json")
if OUTPUT=$(bash "$SETUP_SCRIPT" project "$MULTI_TARGET" --pack go --pack typescript 2>&1); then
    assert_eq "multi-pack install succeeds" "0" "0"
else
    assert_eq "multi-pack install succeeds" "0" "1"
fi
for pack in go typescript; do
    MANIFEST="$MULTI_TARGET/.claude/maverick/packs/$pack.json"
    assert_file_exists "$pack structured manifest installed" "$MANIFEST"
    assert_json_key "$pack manifest uses schema version 1" "$MANIFEST" '.schema_version == 1'
    assert_file_exists "$pack compatibility pointer installed" "$MULTI_TARGET/.claude/maverick/packs/$pack.md"
    while IFS= read -r agent; do
        assert_file_exists "$pack referenced agent installed: $agent" "$MULTI_TARGET/.claude/agents/$agent.md"
    done < <(jq -r '[.agents.planner, .agents.implementer, .agents.red_team, .reviewers[].agent] | .[] | select(. != null)' "$MANIFEST")
done
assert_eq "structured adapter preserved" "$STRUCTURED_BEFORE" "$(cat "$MULTI_TARGET/.claude/maverick/project.json")"
assert_eq "pack.json not copied into agents" "false" "$(test -f "$MULTI_TARGET/.claude/agents/pack.json" && echo true || echo false)"
if bash "$SETUP_SCRIPT" project "$MULTI_TARGET" --pack ../go >/dev/null 2>&1; then
    assert_eq "pack traversal rejected" "rejected" "accepted"
else
    assert_eq "pack traversal rejected" "rejected" "rejected"
fi
for principle in domain application-flow adapters eventing architecture red-team; do
    assert_file_exists "principle installed: $principle" "$MULTI_TARGET/.claude/maverick/principles/$principle.md"
done

ALL_TARGET="$TEST_DIR/all-packs-repo"
mkdir -p "$ALL_TARGET"
ALL_ARGS=()
for manifest in "$SCRIPT_DIR"/claude/agents/*/pack.json; do
    ALL_ARGS+=(--pack "$(jq -r '.id' "$manifest")")
done
if bash "$SETUP_SCRIPT" project "$ALL_TARGET" "${ALL_ARGS[@]}" >/dev/null 2>&1; then
    assert_eq "every pack installs together" "0" "0"
else
    assert_eq "every pack installs together" "0" "1"
fi
for manifest in "$ALL_TARGET"/.claude/maverick/packs/*.json; do
    pack=$(basename "$manifest" .json)
    MISSING=0
    while IFS= read -r agent; do
        [ -f "$ALL_TARGET/.claude/agents/$agent.md" ] || MISSING=$((MISSING + 1))
    done < <(jq -r '[.agents.planner, .agents.implementer, .agents.red_team, .reviewers[].agent] | .[] | select(. != null)' "$manifest")
    assert_eq "$pack: every referenced agent installed" "0" "$MISSING"
done

DETECT_TARGET="$TEST_DIR/detect-repo"
mkdir -p "$DETECT_TARGET/config"
touch "$DETECT_TARGET/Gemfile" "$DETECT_TARGET/config/application.rb" "$DETECT_TARGET/mix.exs"
OUTPUT=$(cd "$DETECT_TARGET" && bash "$SETUP_SCRIPT" project 2>&1) || true
assert_contains "packs detected from repo files" "$OUTPUT" "Detected packs: elixir rails"
assert_file_exists "detected rails pack installed" "$DETECT_TARGET/.claude/maverick/packs/rails.json"
assert_file_exists "detected elixir pack installed" "$DETECT_TARGET/.claude/maverick/packs/elixir.json"
assert_eq "undetected go pack not installed" "false" "$(test -f "$DETECT_TARGET/.claude/maverick/packs/go.json" && echo true || echo false)"

EMPTY_TARGET="$TEST_DIR/empty-repo"
mkdir -p "$EMPTY_TARGET"
OUTPUT=$(bash "$SETUP_SCRIPT" project "$EMPTY_TARGET" 2>&1) || true
assert_contains "no pack detected message" "$OUTPUT" "No language pack detected"
assert_eq "no principles without packs" "false" "$(test -d "$EMPTY_TARGET/.claude/maverick/principles" && echo true || echo false)"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}12b. One-step installer${NC}"

if bash -n "$SCRIPT_DIR/install.sh"; then
    assert_eq "install.sh syntax is valid" "0" "0"
else
    assert_eq "install.sh syntax is valid" "0" "1"
fi
SRC_REPO="$TEST_DIR/src-repo"
git init --quiet -b main "$SRC_REPO"
cp -R "$SCRIPT_DIR/setup.sh" "$SCRIPT_DIR/claude" "$SRC_REPO/"
git -C "$SRC_REPO" add -A
git -C "$SRC_REPO" -c user.name=t -c user.email=t@example.com commit --quiet -m init
INSTALL_HOME="$TEST_DIR/install_home"
mkdir -p "$INSTALL_HOME"
run_installer() {
    env -i HOME="$INSTALL_HOME" PATH=/usr/bin:/bin MAVERICK_REPO="file://$SRC_REPO" \
        bash "$SCRIPT_DIR/install.sh" 2>&1
}
OUTPUT=$(run_installer) || true
assert_file_exists "installer clones into ~/.local/share/maverick" "$INSTALL_HOME/.local/share/maverick/setup.sh"
assert_file_exists "installer adds maverick-setup command" "$INSTALL_HOME/.local/bin/maverick-setup"
assert_contains "installer skips missing CLIs" "$OUTPUT" "Claude Code not found; skipping"
OUTPUT=$(run_installer) || true
assert_contains "installer re-run updates in place" "$OUTPUT" "Updating Maverick"
WRAP_TARGET="$TEST_DIR/wrap-repo"
mkdir -p "$WRAP_TARGET"
touch "$WRAP_TARGET/Cargo.toml"
OUTPUT=$(cd "$WRAP_TARGET" && "$INSTALL_HOME/.local/bin/maverick-setup" project 2>&1) || true
assert_file_exists "maverick-setup installs detected pack" "$WRAP_TARGET/.claude/maverick/packs/rust.json"

# ----------------------------------------------------------
echo ""
echo -e "${YELLOW}13. Hermes setup and unattended dispatch${NC}"

HERMES_HOME="$TEST_DIR/hermes_home"
create_stub_bin "$HERMES_HOME/bin" "hermes"
mkdir -p "$HERMES_HOME/.config/maverick"
echo '{"keep":"me"}' > "$HERMES_HOME/.config/maverick/dispatch.json"

if OUTPUT=$(HOME="$HERMES_HOME" PATH="$HERMES_HOME/bin:$PATH" bash "$SETUP_SCRIPT" hermes 2>&1); then
    assert_file_exists "Hermes skill installed" "$HERMES_HOME/.hermes/skills/autonomous-ai-agents/maverick-dispatch/SKILL.md"
    assert_file_exists "maverick-dispatch installed" "$HERMES_HOME/.local/bin/maverick-dispatch"
    assert_file_exists "maverick-report installed" "$HERMES_HOME/.local/bin/maverick-report"
    assert_eq "existing dispatch.json preserved" '{"keep":"me"}' "$(cat "$HERMES_HOME/.config/maverick/dispatch.json")"
    assert_contains "channel prompt snippet printed" "$OUTPUT" "channel_prompts"
else
    assert_eq "setup.sh hermes succeeds" "0" "1"
fi

assert_json_key "dispatch.json.example is valid with grok disabled" "$SCRIPT_DIR/hermes/dispatch.json.example" '.executors.grok.enabled == false'

REPORT_OUT=$(env -u MAVERICK_DISCORD_WEBHOOK MAVERICK_SECRETS=/nonexistent "$SCRIPT_DIR/hermes/bin/maverick-report" PLAN "hello" 2>&1; echo "exit=$?")
assert_contains "maverick-report without webhook is a silent no-op" "$REPORT_OUT" "exit=0"

# Stub executor: Phase A writes plan.md, Phase B writes result.json (or revises the plan on "refaz").
DISPATCH_DIR="$TEST_DIR/dispatch"
STUB="$DISPATCH_DIR/stub-exec"
mkdir -p "$DISPATCH_DIR/repo"
cat > "$STUB" <<'EOF'
#!/bin/sh
case "$1" in
  plan) printf '## Plan\n%s\n' "$2" > "$MAVERICK_STATE_DIR/plan.md"; echo "MAVERICK_PLAN_READY" ;;
  resume)
    case "$2" in
      *refaz*) sleep 1; printf '## Plan v2\n' > "$MAVERICK_STATE_DIR/plan.md" ;;
      *) printf '{"status":"completed","pr":"https://example/pr/1","branch":"%s","summary":"done"}\n' \
           "$(git rev-parse --abbrev-ref HEAD)" > "$MAVERICK_STATE_DIR/result.json" ;;
    esac ;;
esac
EOF
chmod +x "$STUB"
git -C "$DISPATCH_DIR/repo" init -q -b main
git -C "$DISPATCH_DIR/repo" -c user.email=t@t -c user.name=t commit -q --allow-empty -m init
cat > "$DISPATCH_DIR/dispatch.json" <<EOF
{
  "default_repo": "demo", "default_executor": "stub",
  "state_root": "$DISPATCH_DIR/state", "worktree_root": "$DISPATCH_DIR/wt",
  "repos": {"demo": {"path": "$DISPATCH_DIR/repo", "base": "main"}},
  "executors": {
    "stub": {"enabled": true, "session": "preassign",
             "plan_prompt": "/maverick --unattended=plan {task_args}", "resume_prompt": "{reply}",
             "plan": ["$STUB", "plan", "{prompt}"], "resume": ["$STUB", "resume", "{prompt}"]},
    "grok": {"enabled": false}
  }
}
EOF
DISPATCH="$SCRIPT_DIR/hermes/bin/maverick-dispatch"
export MAVERICK_DISPATCH_CONFIG="$DISPATCH_DIR/dispatch.json" MAVERICK_SECRETS=/nonexistent
PLAN_OUT=$(env -u MAVERICK_DISCORD_WEBHOOK "$DISPATCH" plan --thread T1 '[grok] demo: fix "login" bug' | tail -1)
assert_json_key "plan dispatch reports plan_ready" <(echo "$PLAN_OUT") '.status == "plan_ready"'
assert_json_key "disabled executor falls back without asking" <(echo "$PLAN_OUT") '.note == "grok disabled, using stub"'
assert_contains "task passed verbatim as --local" "$(cat "$(echo "$PLAN_OUT" | jq -r .plan)")" 'local "fix \\"login\\" bug"'
assert_eq "worktree created on maverick/ branch" "true" \
    "$(git -C "$DISPATCH_DIR/repo" branch --list 'maverick/*' | grep -q fix-login-bug && echo true || echo false)"

REDO_OUT=$(env -u MAVERICK_DISCORD_WEBHOOK "$DISPATCH" execute --thread T1 'refaz com X' | tail -1)
assert_json_key "rejecting reply returns a revised plan" <(echo "$REDO_OUT") '.status == "plan_ready"'
EXEC_OUT=$(env -u MAVERICK_DISCORD_WEBHOOK "$DISPATCH" execute --thread T1 'ok' | tail -1)
assert_json_key "execute dispatch reports completed with PR" <(echo "$EXEC_OUT") '.status == "completed" and .pr != null'
NONE_OUT=$(env -u MAVERICK_DISCORD_WEBHOOK "$DISPATCH" execute --thread T1 'ok' | tail -1)
assert_json_key "no second execute once the run finished" <(echo "$NONE_OUT") '.status == "error"'
unset MAVERICK_DISPATCH_CONFIG MAVERICK_SECRETS

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
