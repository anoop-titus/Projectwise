#!/usr/bin/env bash
set -euo pipefail
BIN_DIR="${1:-$HOME/.local/bin}"
rm -f "$BIN_DIR/cpm"
echo "Removed cpm. Remove 'eval \"\$(cpm shell-init)\"' from your shell profile."
