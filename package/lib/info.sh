#!/usr/bin/env bash
# Detailed project information display

source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"

show_project_info() {
  local folder_name="$1"
  [[ -z "$folder_name" ]] && die "Usage: claude-pm info <folder_name>"

  ensure_registry

  local project_data
  project_data=$(registry_get_project "$folder_name")
  [[ -z "$project_data" ]] && die "Project not found: $folder_name"

  echo -e "${CPM_CYAN}Project Information${CPM_NC}"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  echo "$project_data" | jq -r '
    "Name:          \(.display_name)",
    "Folder:        \(.folder_name)",
    "Description:   \(.description)",
    "Category:      \(.category)",
    "Status:        \(.status)",
    "Tags:          \(.tags | join(", "))",
    "Favorite:      \(.favorite)",
    "Created:       \(.created)",
    "Last Accessed: \(.last_accessed)",
    "Sessions:      \(.session_count)",
    "Git Link:      \(.git_link // "—")"
  '

  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

info_main() {
  show_project_info "$@"
}
