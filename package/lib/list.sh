#!/usr/bin/env bash
# List projects in table format

source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"

list_projects_table() {
  local mode="${1:-quick}"
  local registry_path
  registry_path=$(get_registry_path)
  ensure_registry

  echo -e "${CPM_CYAN}Claude Project Manager — Projects${CPM_NC}\n"

  printf "%-3s %-35s %-12s %-8s %8s %12s\n" "" "Name" "Category" "Status" "Sessions" "Last Active"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  registry_get_sorted "$mode" | jq -r \
    '[.favorite, .display_name, .category, .status, .session_count, .last_accessed] | @tsv' | \
  while IFS=$'\t' read -r fav name cat status sessions last_accessed; do
    local fav_icon="  "
    [[ "$fav" == "true" ]] && fav_icon="* "

    local rel
    rel=$(relative_time "$last_accessed")

    printf "%-3s %-35s %-12s %-8s %8s %12s\n" \
      "$fav_icon" "${name:0:35}" "${cat:0:12}" "$status" "$sessions" "$rel"
  done

  echo ""
  local total
  total=$(jq '.projects | length' "$registry_path")
  local active
  active=$(jq '[.projects[] | select(.status == "active")] | length' "$registry_path")
  echo -e "Total: ${CPM_CYAN}${total}${CPM_NC} projects (${active} active)"
}

list_main() {
  list_projects_table "${1:-quick}"
}
