#!/usr/bin/env bash
# Interactive installer for Claude Project Manager
# Usage: ./install.sh [prefix]

set -euo pipefail

COLOR_GREEN='\033[0;32m'
COLOR_CYAN='\033[0;36m'
COLOR_YELLOW='\033[1;33m'
COLOR_RED='\033[0;31m'
COLOR_NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo -e "${COLOR_CYAN}Claude Project Manager — Installer${COLOR_NC}\n"

# Check dependencies
echo "Checking dependencies..."
missing=()
for dep in bash jq fzf; do
  if ! command -v "$dep" &>/dev/null; then
    missing+=("$dep")
  fi
done

if [[ ${#missing[@]} -gt 0 ]]; then
  echo -e "${COLOR_YELLOW}Missing required dependencies: ${missing[*]}${COLOR_NC}"
  echo ""
  if command -v nix &>/dev/null; then
    echo "Install via Nix:"
    for dep in "${missing[@]}"; do
      echo "  nix profile install nixpkgs#$dep"
    done
  else
    echo "Install Nix first: https://nixos.org/download"
    echo "Then: nix profile install nixpkgs#jq nixpkgs#fzf nixpkgs#gum"
  fi
  echo ""
  echo -e "${COLOR_RED}Install dependencies first, then re-run this script.${COLOR_NC}"
  exit 1
fi

# Optional: gum
if ! command -v gum &>/dev/null; then
  echo -e "${COLOR_YELLOW}Optional: 'gum' not found. Install for enhanced TUI:${COLOR_NC}"
  echo "  nix profile install nixpkgs#gum"
  echo "(Continuing with plain-text fallbacks)"
  echo ""
fi

# Installation prefix
INSTALL_PREFIX="${1:-$HOME/.local}"
BIN_DIR="${INSTALL_PREFIX}/bin"
LIB_DIR="${INSTALL_PREFIX}/lib/claude-pm"

echo "Install to: $BIN_DIR (bin), $LIB_DIR (lib)"
read -r -p "Continue? [Y/n]: " confirm
[[ "$confirm" =~ ^[Nn] ]] && { echo "Cancelled."; exit 0; }

mkdir -p "$BIN_DIR" "$LIB_DIR" "$LIB_DIR/templates"

# Copy files
echo -e "\n${COLOR_CYAN}Installing files...${COLOR_NC}"
cp "$SCRIPT_DIR/lib/"*.sh "$LIB_DIR/"
cp "$SCRIPT_DIR/templates/"* "$LIB_DIR/templates/"
cp "$SCRIPT_DIR/VERSION" "$LIB_DIR/"

# Install main binary with patched LIB_DIR
sed "s|SCRIPT_DIR=\"\$(cd \"\$(dirname \"\${BASH_SOURCE\[0\]}\")\/\.\.\" && pwd)\"|SCRIPT_DIR=\"${INSTALL_PREFIX}\"|" \
  "$SCRIPT_DIR/bin/claude-pm" > "$BIN_DIR/claude-pm"
chmod +x "$BIN_DIR/claude-pm"

# Install cpm alias
cp "$SCRIPT_DIR/bin/cpm" "$BIN_DIR/cpm"
chmod +x "$BIN_DIR/cpm"

echo -e "${COLOR_GREEN}+ Files installed${COLOR_NC}"

# Projects directory setup
echo ""
echo -e "${COLOR_CYAN}Projects Directory Setup${COLOR_NC}"
echo ""
echo "Where should Claude projects be stored?"
echo "  1) Default (~/.claude/projects)"
echo "  2) Enter custom path"
echo "  3) Use current directory ($(pwd))"
read -r -p "Choice [1]: " dir_choice

case "${dir_choice:-1}" in
  2)
    read -r -p "Path: " projects_dir
    projects_dir="${projects_dir/#\~/$HOME}"
    ;;
  3)
    projects_dir="$(pwd)"
    ;;
  *)
    projects_dir="$HOME/.claude/projects"
    ;;
esac

mkdir -p "$projects_dir"

# Initialize registry if needed
local_registry="$projects_dir/.registry.json"
if [[ ! -f "$local_registry" ]]; then
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  cat > "$local_registry" << EOF
{
  "version": "2.0.0",
  "projects": [],
  "metadata": {
    "created": "$timestamp",
    "updated": "$timestamp"
  }
}
EOF
  echo -e "${COLOR_GREEN}+ Registry initialized at $local_registry${COLOR_NC}"
fi

# Shell integration
echo ""
echo -e "${COLOR_GREEN}+ Installation complete!${COLOR_NC}"
echo ""
echo "Add to your shell profile (~/.zshrc or ~/.bashrc):"
echo ""
echo "  export PATH=\"$BIN_DIR:\$PATH\""
echo "  export CLAUDE_PROJECTS_DIR=\"$projects_dir\""
echo "  eval \"\$(claude-pm shell-init)\""
echo ""
echo "Then restart your shell or run: source ~/.zshrc"
