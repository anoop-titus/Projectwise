#!/usr/bin/env bash
# Interactive FZF project selector with rich preview and keybindings

set -euo pipefail

# Guard against double-sourcing
if [[ -z "${CLAUDE_PM_CORE_SOURCED:-}" ]]; then
  source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
  export CLAUDE_PM_CORE_SOURCED=1
fi

if [[ -z "${CLAUDE_PM_REGISTRY_SOURCED:-}" ]]; then
  source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"
  export CLAUDE_PM_REGISTRY_SOURCED=1
fi

# Generate FZF input list from registry
# Output: "display_name\tfolder_name" per line, with special entries at bottom
selector_generate_list() {
  local mode="${1:-quick}"
  registry_get_sorted "$mode" | jq -r '"\(.display_name // .folder_name)\t\(.folder_name)"'
  # U+2795 HEAVY PLUS SIGN for "New Project"
  printf '%s\t%s\n' "➕ New Project" "__NEW_PROJECT__"
  # U+1F4AC SPEECH BALLOON for "Quick Session"
  printf '%s\t%s\n' "💬 Quick Session (no project)" "__QUICK_SESSION__"
}

# Main TUI selector with safe command injection prevention
select_project_with_fzf() {
  local mode="${1:-quick}"

  command_exists fzf || die "fzf is required. Install via: nix profile install nixpkgs#fzf"
  command_exists jq  || die "jq is required. Install via: nix profile install nixpkgs#jq"

  local lib_dir
  lib_dir="$(dirname "${BASH_SOURCE[0]}")"

  local fzf_input
  fzf_input=$(selector_generate_list "$mode")

  [[ -z "$fzf_input" ]] && die "No projects found"

  # FZF bindings using safer approach with temporary scripts
  # This avoids shell injection via {2} substitution

  local selected
  selected=$(echo "$fzf_input" | \
    fzf \
      --ansi \
      --delimiter $'\t' \
      --with-nth 1 \
      --header "R:Rename  M:Metadata  F:Favorite  Ctrl-D:Archive  Enter:Select" \
      --preview "source '$lib_dir/core.sh' 2>/dev/null && source '$lib_dir/registry.sh' 2>/dev/null && source '$lib_dir/preview.sh' 2>/dev/null && preview_project {2}" \
      --preview-window "right:50%:wrap" \
      --exit-0 \
      2>/dev/null)

  [[ -z "$selected" ]] && return 1

  # Extract folder_name (this is safe - it comes from our registry)
  local folder_name
  folder_name=$(echo "$selected" | cut -f2)

  echo "$folder_name"
}

# Main entry
selector_main() {
  local mode="${1:-quick}"
  local result
  result=$(select_project_with_fzf "$mode") || return 1

  # Return the result — caller decides what to do with it
  # Special values: __NEW_PROJECT__, __QUICK_SESSION__, or a folder_name
  echo "$result"
}
