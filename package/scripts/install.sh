#!/usr/bin/env bash
# Claude Project Manager (cpm) — Rust binary installer
set -euo pipefail
echo "Claude Project Manager — Installer"
if ! command -v cargo &>/dev/null; then
  echo "Error: Rust toolchain required. Install from https://rustup.rs"; exit 1
fi
if ! command -v fzf &>/dev/null; then
  echo "Warning: fzf not found. Install for project selection."
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN_DIR="${1:-$HOME/.local/bin}"
echo "Building cpm (release)..."
cd "$SCRIPT_DIR" && cargo build --release
mkdir -p "$BIN_DIR" && cp target/release/cpm "$BIN_DIR/cpm"
echo "Installed to $BIN_DIR/cpm"
echo 'Add to shell: eval "$(cpm shell-init)"'
