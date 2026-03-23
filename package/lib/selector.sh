#!/usr/bin/env bash
# Interactive FZF project selector with keybindings

set -euo pipefail

if [[ -z "${CLAUDE_PM_CORE_SOURCED:-}" ]]; then
  source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
fi

if [[ -z "${CLAUDE_PM_REGISTRY_SOURCED:-}" ]]; then
  source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"
fi

# Generate FZF input: "display_name\tfolder_name" per line, specials at bottom
selector_generate_list() {
  local mode="${1:-quick}"
  registry_get_sorted "$mode" | jq -r '"\(.display_name // .folder_name)\t\(.folder_name)"'
  printf '%s\t%s\n' "➕ New Project" "__NEW_PROJECT__"
  printf '%s\t%s\n' "💬 Quick Session (no project)" "__QUICK_SESSION__"
}

# Main TUI selector
select_project_with_fzf() {
  local mode="${1:-quick}"

  command_exists fzf || die "fzf is required. Install via: nix profile install nixpkgs#fzf"
  command_exists jq  || die "jq is required. Install via: nix profile install nixpkgs#jq"

  local lib_dir
  lib_dir="$(dirname "${BASH_SOURCE[0]}")"

  local fzf_input
  fzf_input=$(selector_generate_list "$mode")
  [[ -z "$fzf_input" ]] && die "No projects found"

  # Reload command re-generates the list after mutations
  local reload_cmd="source '$lib_dir/core.sh' && source '$lib_dir/registry.sh' && source '$lib_dir/selector.sh' && selector_generate_list '$mode'"

  local selected
  selected=$(echo "$fzf_input" | \
    fzf \
      --ansi \
      --delimiter $'\t' \
      --with-nth 1 \
      --header "R:Rename  F:Favorite  Ctrl-D:Archive  Enter:Select" \
      --preview "source '$lib_dir/core.sh' && source '$lib_dir/registry.sh' && source '$lib_dir/preview.sh' && preview_project {2}" \
      --preview-window "right:50%:wrap" \
      --bind "r:execute-silent(
          fn=\$(echo {2})
          [[ \"\$fn\" == __* ]] && exit 0
          new_name=\$(bash -c \"source '$lib_dir/core.sh' && prompt_input 'New display name'\")
          [[ -z \"\$new_name\" ]] && exit 0
          source '$lib_dir/core.sh'
          source '$lib_dir/registry.sh'
          registry_set_display_name \"\$fn\" \"\$new_name\"
        )+reload($reload_cmd)" \
      --bind "f:execute-silent(
          fn=\$(echo {2})
          [[ \"\$fn\" == __* ]] && exit 0
          source '$lib_dir/core.sh'
          source '$lib_dir/registry.sh'
          registry_toggle_favorite \"\$fn\"
        )+reload($reload_cmd)" \
      --bind "ctrl-d:execute-silent(
          fn=\$(echo {2})
          [[ \"\$fn\" == __* ]] && exit 0
          source '$lib_dir/core.sh'
          source '$lib_dir/registry.sh'
          registry_set_status \"\$fn\" archived
        )+reload($reload_cmd)" \
      --exit-0 \
      2>/dev/null)

  [[ -z "$selected" ]] && return 1

  local folder_name
  folder_name=$(echo "$selected" | cut -f2)
  echo "$folder_name"
}

# Main entry — returns a folder_name or a __SPECIAL__ value
selector_main() {
  local mode="${1:-quick}"
  local result
  result=$(select_project_with_fzf "$mode") || return 1
  echo "$result"
}
