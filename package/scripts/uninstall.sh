#!/usr/bin/env bash
# Uninstallation script for Claude Project Manager

set -euo pipefail

# Colors
readonly COLOR_RED='\033[0;31m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_NC='\033[0m'

echo -e "${COLOR_BLUE}Claude Project Manager Uninstallation${COLOR_NC}\n"

INSTALL_PREFIX="${1:-/usr/local}"
BIN_DIR="${INSTALL_PREFIX}/bin"
LIB_DIR="${INSTALL_PREFIX}/lib/claude-pm"

echo "Removing from:"
echo "  ${BIN_DIR}"
echo "  ${LIB_DIR}"

# Remove files
rm -f "${BIN_DIR}/claude-pm"
rm -f "${BIN_DIR}/cpm"
rm -rf "${LIB_DIR}"

echo -e "${COLOR_RED}✓ Uninstallation complete${COLOR_NC}"
echo ""
echo "Note: Registry at ~/.claude/registry.json was NOT removed"
echo "Remove manually if needed: rm ~/.claude/registry.json"
