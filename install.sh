#!/usr/bin/env bash
# Maverick one-step installer.
#
#   curl -fsSL https://raw.githubusercontent.com/igorvieira/maverick/main/install.sh | bash
#
# Clones (or updates) Maverick into ~/.local/share/maverick, installs a `maverick-setup` command into
# ~/.local/bin, and configures every supported CLI found on this machine (Claude Code,
# Codex). Safe to re-run: it updates in place and never overwrites project adapters.
#
# Environment:
#   MAVERICK_HOME   install location (default: ~/.local/share/maverick)
#   MAVERICK_REF    branch or tag to install (default: main)
#   MAVERICK_REPO   git URL (default: https://github.com/igorvieira/maverick.git)

set -euo pipefail

MAVERICK_HOME="${MAVERICK_HOME:-$HOME/.local/share/maverick}"
MAVERICK_REF="${MAVERICK_REF:-main}"
MAVERICK_REPO="${MAVERICK_REPO:-https://github.com/igorvieira/maverick.git}"
LOCAL_BIN="$HOME/.local/bin"

say() { printf '%s\n' "$*"; }
fail() { printf 'maverick: %s\n' "$*" >&2; exit 1; }

command -v git &> /dev/null || fail "git is required. Install git and run this again."

if [ -d "$MAVERICK_HOME/.git" ]; then
    say "Updating Maverick in $MAVERICK_HOME"
    [ -z "$(git -C "$MAVERICK_HOME" status --porcelain)" ] \
        || fail "local changes in $MAVERICK_HOME block the update; commit or discard them."
    git -C "$MAVERICK_HOME" fetch --quiet --depth 1 origin "$MAVERICK_REF"
    git -C "$MAVERICK_HOME" checkout --quiet -B "$MAVERICK_REF" FETCH_HEAD
elif [ -e "$MAVERICK_HOME" ]; then
    fail "$MAVERICK_HOME exists and is not a Maverick checkout; set MAVERICK_HOME elsewhere."
else
    say "Installing Maverick into $MAVERICK_HOME"
    git clone --quiet --depth 1 --branch "$MAVERICK_REF" "$MAVERICK_REPO" "$MAVERICK_HOME"
fi

mkdir -p "$LOCAL_BIN"
cat > "$LOCAL_BIN/maverick-setup" <<WRAPPER
#!/usr/bin/env bash
exec "$MAVERICK_HOME/setup.sh" "\$@"
WRAPPER
chmod 755 "$LOCAL_BIN/maverick-setup"
say "Command installed: $LOCAL_BIN/maverick-setup"

configured=0
if command -v claude &> /dev/null; then
    bash "$MAVERICK_HOME/setup.sh" claude && configured=1
else
    say "Claude Code not found; skipping (install it, then run: maverick-setup claude)"
fi
if command -v codex &> /dev/null; then
    bash "$MAVERICK_HOME/setup.sh" codex && configured=1
else
    say "Codex not found; skipping (install it, then run: maverick-setup codex)"
fi
[ "$configured" -eq 1 ] || say "No supported CLI configured yet; packs can still be installed per repo."

case ":$PATH:" in
    *":$LOCAL_BIN:"*) ;;
    *) say ""; say "Add $LOCAL_BIN to your PATH, e.g.: echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc" ;;
esac

say ""
say "Done. In any repository:"
say "  cd /path/to/repo && maverick-setup project     # packs detected from the repo"
say "  maverick-setup project . --pack typescript       # or choose packs explicitly"
