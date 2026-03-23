#!/usr/bin/env bash
# Core utilities for Claude Project Manager
# Note: Callers should set 'set -euo pipefail' if desired.
# We don't set it here to avoid affecting sourced scripts unexpectedly.

# Guard against double-sourcing (prevents readonly re-declaration errors)
if [[ -n "${CLAUDE_PM_CORE_SOURCED:-}" ]]; then
  return 0
fi
export CLAUDE_PM_CORE_SOURCED=1

# Color codes
readonly CPM_RED='\033[0;31m'
readonly CPM_GREEN='\033[0;32m'
readonly CPM_YELLOW='\033[1;33m'
readonly CPM_BLUE='\033[0;34m'
readonly CPM_CYAN='\033[0;36m'
readonly CPM_GRAY='\033[0;90m'
readonly CPM_BOLD='\033[1m'
readonly CPM_NC='\033[0m'

# Resolve CPM_HOME: where projects live
# Priority: CLAUDE_PROJECTS_DIR env > ~/.claude/projects
CPM_HOME="${CLAUDE_PROJECTS_DIR:-$HOME/.claude/projects}"
CPM_ARCHIVE_DIR="${CLAUDE_ARCHIVE_DIR:-$HOME/.claude/archive}"

# Registry path (always inside CPM_HOME)
get_registry_path() {
  echo "${CPM_HOME}/.registry.json"
}

# Print helpers
print_info()    { echo -e "${CPM_BLUE}i${CPM_NC} $*"; }
print_success() { echo -e "${CPM_GREEN}+${CPM_NC} $*"; }
print_error()   { echo -e "${CPM_RED}x${CPM_NC} $*" >&2; }
print_warning() { echo -e "${CPM_YELLOW}!${CPM_NC} $*"; }

die() { print_error "$@"; exit 1; }

command_exists() { command -v "$1" &>/dev/null; }

# Check if gum is available; if not, use fallback prompts
has_gum() { command_exists gum; }

# Prompt for input — uses gum if available, else read
prompt_input() {
  local placeholder="$1"
  local default="${2:-}"
  if has_gum; then
    gum input --placeholder "$placeholder" --value "$default"
  else
    local input
    if [[ -n "$default" ]]; then
      read -r -p "$placeholder [$default]: " input
      echo "${input:-$default}"
    else
      read -r -p "$placeholder: " input
      echo "$input"
    fi
  fi
}

# Prompt for confirmation — uses gum if available, else read
prompt_confirm() {
  local message="$1"
  if has_gum; then
    gum confirm "$message"
  else
    local answer
    read -r -p "$message [y/N]: " answer
    [[ "$answer" =~ ^[Yy] ]]
  fi
}

# Prompt for choice — uses gum if available, else numbered menu
# Validates input and returns empty string on invalid choice
prompt_choose() {
  local header="$1"
  shift
  local num_opts=$#

  if has_gum; then
    printf '%s\n' "$@" | gum choose --header "$header"
  else
    echo "$header" >&2
    local i=1
    for opt in "$@"; do
      echo "  $i) $opt" >&2
      ((i++))
    done
    local choice
    read -r -p "Choice [1-$num_opts]: " choice

    # Validate choice is numeric and in range
    if ! [[ "$choice" =~ ^[0-9]+$ ]] || ((choice < 1 || choice > num_opts)); then
      echo "Error: Invalid choice '$choice'. Expected 1-$num_opts" >&2
      return 1
    fi

    local idx=$((choice - 1))
    local arr=("$@")
    echo "${arr[$idx]}"
  fi
}

# Ensure registry exists
ensure_registry() {
  local registry_path
  registry_path=$(get_registry_path)
  if [[ ! -f "$registry_path" ]]; then
    die "Registry not found at $registry_path. Run 'claude-pm setup' first."
  fi
}

# Validate JSON file
validate_json() {
  jq empty "$1" 2>/dev/null
}

# Atomic registry update: apply jq expression + optional --arg/--argjson flags, validate, backup, swap.
# Usage: registry_atomic_update [--arg key val] ... 'jq_filter'
# Keeps last 10 backups.
registry_atomic_update() {
  local registry_path
  registry_path=$(get_registry_path)

  local temp_file
  temp_file=$(mktemp) || { print_error "Failed to create temp file"; return 1; }

  if jq "$@" "$registry_path" > "$temp_file" && validate_json "$temp_file"; then
    local backup_dir
    backup_dir="$(dirname "$registry_path")/.backups"
    mkdir -p "$backup_dir"
    cp "$registry_path" "${backup_dir}/registry.$(date +%s).backup"
    (cd "$backup_dir" && ls -t registry.*.backup 2>/dev/null | tail -n +11 | xargs rm -f 2>/dev/null || true)
    mv "$temp_file" "$registry_path"
    return 0
  else
    rm -f "$temp_file"
    print_error "Registry update failed"
    return 1
  fi
}

# Convert ISO timestamp to relative time (Linux-first, macOS fallback)
relative_time() {
  local ts="$1"
  [[ -z "$ts" || "$ts" == "null" ]] && echo "—" && return

  local unix_ts now diff
  unix_ts=$(
    date -d "$ts" +%s 2>/dev/null ||
    date -j -f "%Y-%m-%dT%H:%M:%SZ" "$ts" +%s 2>/dev/null ||
    echo 0
  )
  [[ $unix_ts -eq 0 ]] && echo "—" && return

  now=$(date +%s)
  diff=$((now - unix_ts))

  if   ((diff < 60));     then echo "just now"
  elif ((diff < 3600));   then echo "$((diff / 60))m ago"
  elif ((diff < 86400));  then echo "$((diff / 3600))h ago"
  elif ((diff < 604800)); then echo "$((diff / 86400))d ago"
  else
    date -d "$ts" "+%b %d" 2>/dev/null ||
    date -j -f "%Y-%m-%dT%H:%M:%SZ" "$ts" "+%b %d" 2>/dev/null ||
    echo "—"
  fi
}
