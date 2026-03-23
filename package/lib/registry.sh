#!/usr/bin/env bash
# Registry management for Claude Project Manager
# Registry format: array-based with rich metadata per project

# Guard against double-sourcing
if [[ -n "${CLAUDE_PM_REGISTRY_SOURCED:-}" ]]; then
  return 0
fi
export CLAUDE_PM_REGISTRY_SOURCED=1

if [[ -z "${CLAUDE_PM_CORE_SOURCED:-}" ]]; then
  source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
fi

# Initialize registry
registry_init() {
  local registry_path
  registry_path=$(get_registry_path)

  mkdir -p "$(dirname "$registry_path")"

  if [[ -f "$registry_path" ]]; then
    print_warning "Registry already exists at $registry_path"
    return 0
  fi

  local timestamp
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  cat > "$registry_path" << EOF
{
  "version": "2.0.0",
  "projects": [],
  "metadata": {
    "created": "$timestamp",
    "updated": "$timestamp"
  }
}
EOF

  print_success "Registry initialized at $registry_path"
}

# Add project to registry
registry_add() {
  local folder_name="$1"
  local display_name="${2:-$folder_name}"
  local description="${3:-Project}"
  local category="${4:-Research}"

  [[ -z "$folder_name" ]] && die "Usage: registry_add <folder_name> [display_name] [description] [category]"
  ensure_registry

  local timestamp
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  registry_atomic_update \
    --arg fn "$folder_name" \
    --arg dn "$display_name" \
    --arg desc "$description" \
    --arg cat "$category" \
    --arg ts "$timestamp" \
    '.projects += [{
      "id": $fn,
      "folder_name": $fn,
      "display_name": $dn,
      "description": $desc,
      "tags": [],
      "category": $cat,
      "status": "active",
      "created": $ts,
      "last_accessed": $ts,
      "session_count": 0,
      "git_link": null,
      "favorite": false
    }]'

  print_success "Added project: $display_name"
}

# Remove project from registry
registry_remove() {
  local folder_name="$1"
  [[ -z "$folder_name" ]] && die "Usage: registry_remove <folder_name>"
  ensure_registry

  registry_atomic_update \
    --arg fn "$folder_name" \
    '(.projects) |= map(select(.folder_name != $fn))'

  print_success "Removed project: $folder_name"
}

# Get project JSON by folder_name
registry_get_project() {
  local folder_name="$1"
  [[ -z "$folder_name" ]] && return 1
  ensure_registry

  jq --arg fn "$folder_name" \
    '.projects[] | select(.folder_name == $fn)' "$(get_registry_path)"
}

# List all project folder_names
registry_list_names() {
  ensure_registry
  jq -r '.projects[].folder_name' "$(get_registry_path)"
}

# Update a single string field on a project (safe via --arg)
registry_set_field() {
  local folder_name="$1"
  local field="$2"
  local value="$3"
  ensure_registry

  registry_atomic_update \
    --arg fn "$folder_name" \
    --arg val "$value" \
    --arg field "$field" \
    '(.projects[] | select(.folder_name == $fn) | .[$field]) |= $val'
}

# Update last_accessed timestamp
registry_touch() {
  local folder_name="$1"
  local timestamp
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  registry_set_field "$folder_name" "last_accessed" "$timestamp"
}

# Increment session count
registry_increment_sessions() {
  local folder_name="$1"
  ensure_registry
  registry_atomic_update \
    --arg fn "$folder_name" \
    '(.projects[] | select(.folder_name == $fn) | .session_count) |= (. + 1)'
}

# Toggle favorite
registry_toggle_favorite() {
  local folder_name="$1"
  ensure_registry
  registry_atomic_update \
    --arg fn "$folder_name" \
    '(.projects[] | select(.folder_name == $fn) | .favorite) |= not'
}

registry_set_display_name() { registry_set_field "$1" "display_name" "$2"; }
registry_set_category()     { registry_set_field "$1" "category" "$2"; }
registry_set_description()  { registry_set_field "$1" "description" "$2"; }

# Set status (validates allowed values)
registry_set_status() {
  local folder_name="$1"
  local status="$2"
  [[ "$status" =~ ^(active|paused|archived)$ ]] || die "Invalid status: $status"
  registry_set_field "$folder_name" "status" "$status"
}

# Set tags (comma-separated string → JSON array)
registry_set_tags() {
  local folder_name="$1"
  local tags_str="$2"
  ensure_registry

  local tags_json
  tags_json=$(echo "$tags_str" | jq -R 'split(",") | map(gsub("^\\s+|\\s+$";""))')

  registry_atomic_update \
    --argjson tags "$tags_json" \
    --arg fn "$folder_name" \
    '(.projects[] | select(.folder_name == $fn) | .tags) |= $tags'
}

# Set git link (pass empty string to clear)
registry_set_git_link() {
  local folder_name="$1"
  local url="$2"
  ensure_registry
  if [[ -z "$url" ]]; then
    registry_atomic_update \
      --arg fn "$folder_name" \
      '(.projects[] | select(.folder_name == $fn) | .git_link) |= null'
  else
    registry_set_field "$folder_name" "git_link" "$url"
  fi
}

# Get projects sorted by last_accessed desc, optionally filtered by mode
registry_get_sorted() {
  local mode="${1:-quick}"
  local registry_path
  registry_path=$(get_registry_path)
  ensure_registry

  case "$mode" in
    favorite)
      jq '[.projects[] | select(.favorite == true)] | sort_by(.last_accessed) | reverse | .[]' "$registry_path"
      ;;
    all)
      jq '[.projects[]] | sort_by(.last_accessed) | reverse | .[]' "$registry_path"
      ;;
    *)
      jq '[.projects[] | select(.status == "active")] | sort_by(.last_accessed) | reverse | .[]' "$registry_path"
      ;;
  esac
}

# CLI dispatcher
registry_main() {
  local subcommand="${1:-help}"
  case "$subcommand" in
    init)            registry_init ;;
    add)             registry_add "$2" "$3" "$4" "$5" ;;
    remove)          registry_remove "$2" ;;
    list)            registry_list_names ;;
    get)             registry_get_project "$2" ;;
    touch)           registry_touch "$2" ;;
    set-name)        registry_set_display_name "$2" "$3" ;;
    set-status)      registry_set_status "$2" "$3" ;;
    set-category)    registry_set_category "$2" "$3" ;;
    set-description) registry_set_description "$2" "$3" ;;
    set-tags)        registry_set_tags "$2" "$3" ;;
    set-git)         registry_set_git_link "$2" "$3" ;;
    toggle-fav)      registry_toggle_favorite "$2" ;;
    help|--help|-h)
      cat << 'HELP'
Registry commands:
  init                           Initialize a new registry
  add <folder> [name] [desc]     Add a project
  remove <folder>                Remove a project
  list                           List project folder names
  get <folder>                   Show project JSON
  touch <folder>                 Update last_accessed
  set-name <folder> <name>       Rename display name
  set-status <folder> <status>   Set status (active|paused|archived)
  set-category <folder> <cat>    Set category
  set-description <folder> <d>   Set description
  set-tags <folder> <csv-tags>   Set tags (comma-separated)
  set-git <folder> <url>         Set git link (empty to clear)
  toggle-fav <folder>            Toggle favorite flag
HELP
      ;;
    *) die "Unknown registry command: $subcommand" ;;
  esac
}
