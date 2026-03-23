#!/usr/bin/env bash
# Claude Project Manager (cpm) — Rust binary installer
set -euo pipefail

GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}Claude Project Manager — Installer${NC}"
echo ""

# Check dependencies
if ! command -v cargo &>/dev/null; then
  echo "Error: Rust toolchain required. Install from https://rustup.rs"; exit 1
fi
if ! command -v fzf &>/dev/null; then
  echo "Warning: fzf not found. Install it for project selection (brew install fzf)"
fi

# Build
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN_DIR="${1:-$HOME/.local/bin}"
echo "Building cpm (release)..."
cd "$SCRIPT_DIR" && cargo build --release
mkdir -p "$BIN_DIR" && cp target/release/cpm "$BIN_DIR/cpm"
echo -e "${GREEN}+ Binary installed to $BIN_DIR/cpm${NC}"

# Ensure BIN_DIR is in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
  echo "Note: $BIN_DIR is not in your PATH"
fi

# Initialize registry if needed
PROJECTS_DIR="${CLAUDE_PROJECTS_DIR:-$HOME/.claude/projects}"
REGISTRY="$PROJECTS_DIR/.registry.json"
if [[ ! -f "$REGISTRY" ]]; then
  mkdir -p "$PROJECTS_DIR"
  "$BIN_DIR/cpm" registry init
  echo -e "${GREEN}+ Registry initialized${NC}"
elif ! python3 -c "import json; r=json.load(open('$REGISTRY')); assert 'metadata' in r" 2>/dev/null; then
  # Migrate old registry format
  python3 -c "
import json, datetime
with open('$REGISTRY') as f: reg = json.load(f)
now = datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
if 'metadata' not in reg: reg['metadata'] = {'created': now, 'updated': now}
reg['version'] = '3.1.0'
for p in reg.get('projects', []):
    if 'archive_path' not in p: p['archive_path'] = p.pop('codebase_path', None)
    if 'tags' not in p: p['tags'] = []
    if 'session_count' not in p: p['session_count'] = 0
    if 'favorite' not in p: p['favorite'] = False
with open('$REGISTRY', 'w') as f: json.dump(reg, f, indent=2)
print('Migrated registry to v3.1.0')
"
  echo -e "${GREEN}+ Registry migrated${NC}"
else
  echo -e "${GREEN}+ Registry exists${NC}"
fi

# Shell integration
SHELL_INIT='eval "$(cpm shell-init)"'
SHELL_RC=""
if [[ -n "${ZSH_VERSION:-}" ]] || [[ "$SHELL" == */zsh ]]; then
  SHELL_RC="$HOME/.zshrc"
elif [[ -n "${BASH_VERSION:-}" ]] || [[ "$SHELL" == */bash ]]; then
  SHELL_RC="$HOME/.bashrc"
fi

if [[ -n "$SHELL_RC" ]]; then
  if grep -qF 'cpm shell-init' "$SHELL_RC" 2>/dev/null; then
    echo -e "${GREEN}+ Shell integration already in $SHELL_RC${NC}"
  else
    echo "" >> "$SHELL_RC"
    echo '# Claude Project Manager' >> "$SHELL_RC"
    echo "$SHELL_INIT" >> "$SHELL_RC"
    echo -e "${GREEN}+ Added shell integration to $SHELL_RC${NC}"
  fi
fi

echo ""
echo -e "${GREEN}Done!${NC} Restart your shell or run: source ${SHELL_RC:-~/.zshrc}"
echo "Then type 'claude' to launch the project picker."
