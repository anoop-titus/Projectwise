#!/usr/bin/env bash
# Project archive/restore/delete

source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"

# Archive: move to archive dir, mark as archived
archive_project() {
  local folder_name="$1"
  [[ -z "$folder_name" ]] && die "Usage: claude-pm archive <folder_name>"

  prompt_confirm "Archive project '$folder_name'?" || { echo "Cancelled."; return 1; }

  mkdir -p "$CPM_ARCHIVE_DIR"

  local src="${CPM_HOME}/${folder_name}"
  if [[ -d "$src" ]]; then
    mv "$src" "${CPM_ARCHIVE_DIR}/${folder_name}"
  fi

  registry_set_status "$folder_name" "archived"

  # Store archive path
  local archive_path="${CPM_ARCHIVE_DIR}/${folder_name}"
  ensure_registry
  registry_atomic_update_args \
    --arg fn "$folder_name" \
    --arg ap "$archive_path" \
    '(.projects[] | select(.folder_name == $fn)) |= (.archive_path = $ap)'

  print_success "Archived: $folder_name -> $archive_path"
}

# Restore from archive
restore_project() {
  local folder_name="$1"
  [[ -z "$folder_name" ]] && die "Usage: claude-pm restore <folder_name>"

  local archive_path
  archive_path=$(jq -r ".projects[] | select(.folder_name == \"$folder_name\") | .archive_path // empty" "$(get_registry_path)")
  [[ -z "$archive_path" ]] && archive_path="${CPM_ARCHIVE_DIR}/${folder_name}"

  [[ ! -d "$archive_path" ]] && die "Archive not found at $archive_path"

  mv "$archive_path" "${CPM_HOME}/${folder_name}"

  registry_set_status "$folder_name" "active"
  registry_atomic_update \
    "(.projects[] | select(.folder_name == \"$folder_name\")) |= del(.archive_path)"

  print_success "Restored: $folder_name"
}

# Permanent delete (removes from registry and optionally filesystem)
delete_project() {
  local folder_name="$1"
  [[ -z "$folder_name" ]] && die "Usage: claude-pm delete <folder_name>"

  prompt_confirm "PERMANENTLY delete '$folder_name' from registry?" || return 1

  local project_path="${CPM_HOME}/${folder_name}"
  registry_remove "$folder_name"

  if [[ -d "$project_path" ]]; then
    if prompt_confirm "Also delete the directory at $project_path?"; then
      rm -rf "$project_path"
      print_success "Directory removed"
    else
      print_info "Directory preserved at $project_path"
    fi
  fi
}

delete_main() {
  local subcommand="${1:-help}"
  case "$subcommand" in
    archive)  archive_project "$2" ;;
    restore)  restore_project "$2" ;;
    remove)   delete_project "$2" ;;
    help|--help|-h)
      cat << 'HELP'
Delete/Archive commands:
  archive <folder>    Move project to archive, mark archived
  restore <folder>    Restore from archive
  remove <folder>     Permanently delete from registry
HELP
      ;;
    *) die "Unknown delete command: $subcommand" ;;
  esac
}
