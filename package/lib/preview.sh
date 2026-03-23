#!/usr/bin/env bash
# Project preview for FZF pane and standalone use

source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"

preview_project() {
  local folder_name="$1"

  [[ -z "$folder_name" || "$folder_name" == "__NEW_PROJECT__" || "$folder_name" == "__QUICK_SESSION__" ]] && return 0

  local registry_path
  registry_path=$(get_registry_path)

  # Extract all fields in a single jq call
  local fields
  fields=$(jq -r --arg fn "$folder_name" '
    .projects[] | select(.folder_name == $fn or .id == $fn) |
    [
      (.display_name // "—"),
      (.description // "—"),
      ([ .tags[]? ] | join(",")),
      (.category // "—"),
      (.status // "—"),
      (.created // "—"),
      (.last_accessed // "—"),
      ((.session_count // 0) | tostring),
      (.git_link // "—"),
      ((.favorite // false) | tostring)
    ] | join("\u0000")
  ' "$registry_path" 2>/dev/null)

  [[ -z "$fields" ]] && echo "Project not found: $folder_name" && return 0

  IFS=$'\0' read -r display_name description tags category status \
    created last_accessed session_count git_link is_favorite <<< "$fields"

  [[ -z "$tags" ]] && tags="—"

  local created_short="${created%%T*}"
  local accessed_short="${last_accessed%%T*}"
  local accessed_rel
  accessed_rel=$(relative_time "$last_accessed")

  # Folder stats
  local project_folder="${CPM_HOME}/${folder_name}"
  local file_count="—" size="—"
  if [[ -d "$project_folder" ]]; then
    file_count=$(find "$project_folder" -type f 2>/dev/null | wc -l | tr -d ' ')
    size=$(du -sh "$project_folder" 2>/dev/null | awk '{print $1}')
  fi

  # Render with gum if available, else plain text
  local fav_icon=""
  [[ "$is_favorite" == "true" ]] && fav_icon=" *"

  if has_gum; then
    gum style --padding "1 2" --border "double" --align "left" \
      "$(gum style --bold --foreground "39" "$display_name$fav_icon")"

    [[ "$description" != "—" && "$description" != "Project" ]] && \
      gum style --padding "0 2" --foreground "248" "$description"

    echo
    gum style --padding "0 2" \
      "$(gum style --bold 'Category:'     ) $category" \
      "$(gum style --bold 'Status:'       ) $status" \
      "$(gum style --bold 'Tags:'         ) $tags" \
      "$(gum style --bold 'Created:'      ) $created_short" \
      "$(gum style --bold 'Last Active:'  ) $accessed_rel" \
      "$(gum style --bold 'Sessions:'     ) $session_count" \
      "$(gum style --bold 'Files:'        ) $file_count" \
      "$(gum style --bold 'Size:'         ) $size"

    # Size warning
    if [[ -d "$project_folder" ]]; then
      local size_kb
      size_kb=$(du -sk "$project_folder" 2>/dev/null | awk '{print $1}')
      if ((size_kb > 10240)); then
        echo
        gum style --padding "0 2" --foreground "196" --bold "! Large project ($size) — consider archiving or pruning"
      elif ((size_kb > 5120)); then
        echo
        gum style --padding "0 2" --foreground "208" "! Project size: $size — consider cleanup"
      fi
    fi

    [[ "$git_link" != "—" && "$git_link" != "null" ]] && echo && \
      gum style --padding "0 2" --foreground "35" "$(gum style --bold 'Git:') $git_link"
  else
    # Plain text fallback
    echo "=== $display_name$fav_icon ==="
    [[ "$description" != "—" && "$description" != "Project" ]] && echo "$description"
    echo ""
    printf "  %-15s %s\n" "Category:" "$category"
    printf "  %-15s %s\n" "Status:" "$status"
    printf "  %-15s %s\n" "Tags:" "$tags"
    printf "  %-15s %s\n" "Created:" "$created_short"
    printf "  %-15s %s\n" "Last Active:" "$accessed_rel"
    printf "  %-15s %s\n" "Sessions:" "$session_count"
    printf "  %-15s %s\n" "Files:" "$file_count"
    printf "  %-15s %s\n" "Size:" "$size"
    [[ "$git_link" != "—" && "$git_link" != "null" ]] && printf "  %-15s %s\n" "Git:" "$git_link"
  fi
}

preview_main() {
  preview_project "$@"
}
