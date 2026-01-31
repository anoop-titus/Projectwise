#!/usr/bin/env bash
# Create new Claude projects

source "$(dirname "${BASH_SOURCE[0]}")/core.sh"
source "$(dirname "${BASH_SOURCE[0]}")/registry.sh"

# Create from existing codebase path
create_from_existing() {
  local codebase_path="$1"
  local project_name="${2:-$(basename "$codebase_path")}"

  [[ -z "$codebase_path" || ! -d "$codebase_path" ]] && die "Invalid path: $codebase_path"

  local folder_name="${project_name}_$(date +%s)"
  mkdir -p "${CPM_HOME}/${folder_name}"

  # Symlink to codebase
  ln -sf "$(cd "$codebase_path" && pwd)" "${CPM_HOME}/${folder_name}/codebase"

  registry_add "$folder_name" "$project_name" "Linked to $codebase_path" "Development"

  print_success "Created project: $project_name (linked to $codebase_path)"
  echo "$folder_name"
}

# Create fresh project
create_fresh() {
  local project_name="$1"
  [[ -z "$project_name" ]] && die "Project name required"

  local folder_name="${project_name}_$(date +%s)"
  local project_path="${CPM_HOME}/${folder_name}"

  mkdir -p "$project_path"/{.claude,.planning,docs}

  # Create .gitignore
  cat > "$project_path/.gitignore" << 'GI'
.DS_Store
*.swp
*~
node_modules/
dist/
build/
.env
GI

  registry_add "$folder_name" "$project_name" "Project" "Research"

  print_success "Created project: $project_name at $project_path"
  echo "$folder_name"
}

# Interactive create flow
create_interactive() {
  local choice
  choice=$(prompt_choose "New project type:" "Existing codebase" "Fresh project")

  case "$choice" in
    "Existing"*)
      local path
      path=$(prompt_input "Path to existing codebase")
      [[ -z "$path" ]] && return 1
      # Expand ~ and resolve
      path="${path/#\~/$HOME}"
      local name
      name=$(prompt_input "Project name" "$(basename "$path")")
      create_from_existing "$path" "$name"
      ;;
    "Fresh"*)
      local name
      name=$(prompt_input "Project name")
      [[ -z "$name" ]] && return 1
      create_fresh "$name"
      ;;
    *)
      return 1
      ;;
  esac
}

create_main() {
  local subcommand="${1:-interactive}"
  case "$subcommand" in
    interactive)   create_interactive ;;
    existing)      create_from_existing "$2" "$3" ;;
    fresh)         create_fresh "$2" ;;
    help|--help|-h)
      cat << 'HELP'
Create commands:
  interactive              Guided project creation
  existing <path> [name]   Link existing codebase
  fresh <name>             Create new empty project
HELP
      ;;
    *) die "Unknown create command: $subcommand" ;;
  esac
}
