#!/usr/bin/env bash
# BATS test helper

TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${TEST_DIR}/.." && pwd)"

setup_test_registry() {
  export TEST_PROJECTS_DIR
  TEST_PROJECTS_DIR=$(mktemp -d)
  export CLAUDE_PROJECTS_DIR="$TEST_PROJECTS_DIR"
  export CPM_HOME="$TEST_PROJECTS_DIR"

  local ts
  ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  cat > "$TEST_PROJECTS_DIR/.registry.json" << EOF
{
  "version": "2.0.0",
  "projects": [],
  "metadata": {
    "created": "$ts",
    "updated": "$ts"
  }
}
EOF
}

teardown_test_registry() {
  [[ -n "$TEST_PROJECTS_DIR" && -d "$TEST_PROJECTS_DIR" ]] && rm -rf "$TEST_PROJECTS_DIR"
}

source "$PROJECT_DIR/lib/core.sh"
source "$PROJECT_DIR/lib/registry.sh"
