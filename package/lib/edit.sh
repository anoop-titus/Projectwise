#!/usr/bin/env bash
# Interactive metadata editor

source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"

edit_interactive() {
  local folder_name="$1"
  [[ -z "$folder_name" ]] && die "Usage: claude-pm edit <folder_name>"

  local project
  project=$(registry_get_project "$folder_name")
  [[ -z "$project" ]] && die "Project not found: $folder_name"

  local display_name category
  display_name=$(echo "$project" | jq -r '.display_name')
  category=$(echo "$project" | jq -r '.category')

  while true; do
    local choice
    choice=$(prompt_choose "Edit: $display_name" \
      "1. Rename display name" \
      "2. Edit description" \
      "3. Edit category" \
      "4. Edit tags" \
      "5. Edit git link" \
      "6. Toggle favorite" \
      "7. Change status" \
      "8. Done")

    case "$choice" in
      "1."*|"1")
        local current
        current=$(echo "$project" | jq -r '.display_name')
        local new_name
        new_name=$(prompt_input "Display name" "$current")
        [[ -n "$new_name" && "$new_name" != "$current" ]] && \
          registry_set_display_name "$folder_name" "$new_name" && \
          print_success "Renamed to: $new_name"
        ;;
      "2."*|"2")
        local current
        current=$(echo "$project" | jq -r '.description')
        local new_desc
        new_desc=$(prompt_input "Description" "$current")
        [[ -n "$new_desc" ]] && registry_set_description "$folder_name" "$new_desc"
        ;;
      "3."*|"3")
        local new_cat
        new_cat=$(prompt_choose "Category" "Research" "Development" "Testing" "Personal" "Archive")
        [[ -n "$new_cat" ]] && registry_set_category "$folder_name" "$new_cat"
        ;;
      "4."*|"4")
        local current_tags
        current_tags=$(echo "$project" | jq -r '.tags | join(", ")')
        local new_tags
        new_tags=$(prompt_input "Tags (comma-separated)" "$current_tags")
        registry_set_tags "$folder_name" "$new_tags"
        ;;
      "5."*|"5")
        local new_git
        new_git=$(prompt_input "Git URL" "")
        registry_set_git_link "$folder_name" "$new_git"
        ;;
      "6."*|"6")
        registry_toggle_favorite "$folder_name"
        print_success "Favorite toggled"
        ;;
      "7."*|"7")
        local new_status
        new_status=$(prompt_choose "Status" "active" "paused" "archived")
        [[ -n "$new_status" ]] && registry_set_status "$folder_name" "$new_status"
        ;;
      "8."*|"8"|"")
        return 0
        ;;
    esac

    # Refresh project data
    project=$(registry_get_project "$folder_name")
    display_name=$(echo "$project" | jq -r '.display_name')
  done
}

edit_main() {
  edit_interactive "$@"
}
