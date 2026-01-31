# Usage Guide

## Registry Commands

### Initialize Registry

Initialize a new project registry:

```bash
claude-pm registry init
```

This creates `~/.claude/registry.json` with the initial structure.

### Add Project to Registry

Add an existing project:

```bash
claude-pm registry add my-project ~/path/to/my-project
```

Multiple projects:

```bash
claude-pm registry add project1 ~/projects/project1
claude-pm registry add project2 ~/projects/project2
claude-pm registry add project3 ~/projects/project3
```

### Remove Project from Registry

```bash
claude-pm registry remove my-project
```

### List All Projects

```bash
claude-pm registry list
```

Or with full details:

```bash
claude-pm list
```

### Update Registry Metadata

```bash
claude-pm registry update
```

## Navigation Commands

### Interactive Project Selection

Select a project interactively with fzf:

```bash
claude-pm select
```

This displays a list of projects and lets you choose with arrow keys.

### Navigate to Project

Navigate directly to a project:

```bash
claude-pm select my-project
```

### List Projects

Show all projects in a formatted table:

```bash
claude-pm list
```

## Project Information

### View Project Details

```bash
claude-pm info my-project
```

Shows:
- Project name
- Full path
- Added date
- Last accessed date

### Preview Project

```bash
claude-pm preview my-project
```

Shows:
- Basic information
- Git status
- File/directory counts
- Local registry info

## Project Management

### Create New Project

Create and initialize a new project:

```bash
claude-pm create my-new-project
```

Or specify a custom path:

```bash
claude-pm create my-project ~/custom/path/my-project
```

The command:
1. Creates the directory structure
2. Initializes git repository
3. Creates .claude directory
4. Creates .planning directory
5. Adds project to registry

### Delete Project

Remove a project from the registry:

```bash
claude-pm delete my-project
```

With confirmation (default):
- Prompts before removing
- Asks whether to delete directory

Force deletion:

```bash
claude-pm delete my-project --force
```

## Symlink Management

### Browse Symlinks

View all project symlinks:

```bash
claude-pm symlink browse
```

Shows:
- Symlink name
- Target path
- Status (active/missing)

### Organize Symlinks

Create symlinks for all projects in a directory:

```bash
claude-pm symlink organize ~/projects-links
```

This creates symlinks like:
```
~/projects-links/my-project -> /full/path/to/my-project
~/projects-links/other-project -> /full/path/to/other-project
```

## Shell Integration

### Quick Aliases

Add to your shell configuration:

```bash
alias cpm="claude-pm"           # Short form
alias cpp="claude-pm select"    # Quick select
alias cpi="claude-pm info"      # Info shortcut
```

### Project Change Function

Add to your shell to cd into a project:

```bash
cpc() {
  local project_path
  project_path=$(claude-pm select "$@")
  if [[ -n "${project_path}" ]]; then
    cd "${project_path}"
  fi
}
```

Usage:

```bash
cpc              # Interactive selection, then cd
cpc my-project   # Direct cd to project
```

### Full Shell Integration

Complete shell setup in `~/.zshrc` or `~/.bashrc`:

```bash
# Claude Project Manager
export CLAUDE_REGISTRY_PATH="$HOME/.claude/registry.json"

# Aliases
alias cpm="claude-pm"
alias cpp="claude-pm select"
alias cpi="claude-pm info"

# Functions
cpc() {
  local project_path
  project_path=$(claude-pm select "$@")
  if [[ -n "${project_path}" ]]; then
    cd "${project_path}"
  fi
}

# Optional: Tab completion (if implemented)
# _claude_pm_completion() { ... }
# complete -o default -o nospace -F _claude_pm_completion claude-pm
```

## Common Workflows

### Workflow 1: Quick Navigation

```bash
# List projects
claude-pm list

# Jump to a project
cpc my-project

# View project info
claude-pm info my-project
```

### Workflow 2: Project Creation

```bash
# Create new project
claude-pm create my-app ~/projects/my-app

# Navigate to it
cd ~/projects/my-app

# Add local registry
claude-pm registry init

# Start development
```

### Workflow 3: Project Migration

```bash
# Initialize registry
claude-pm registry init

# Add existing projects
claude-pm registry add project1 ~/old/project1
claude-pm registry add project2 ~/old/project2

# Verify
claude-pm list

# Set up symlinks
claude-pm symlink organize ~/projects-shortcuts
```

### Workflow 4: Batch Operations

```bash
# List all projects
for project in $(claude-pm registry list); do
  echo "Processing: $project"
  claude-pm info "$project"
done

# Update all projects
for project in $(claude-pm registry list); do
  cd "$(claude-pm select "$project")"
  git pull
done
```

## Advanced Usage

### Custom Registry Location

```bash
export CLAUDE_REGISTRY_PATH="/custom/path/registry.json"
claude-pm registry init
```

### Registry Backup

```bash
cp ~/.claude/registry.json ~/.claude/registry.json.backup
```

### Registry Export

```bash
# JSON format
claude-pm registry list | jq -R 'split("\n")[:-1]' > projects.json

# CSV format
echo "Project,Path" > projects.csv
jq -r '.projects | to_entries | .[] | "\(.key),\(.value.path)"' \
  ~/.claude/registry.json >> projects.csv
```

### Bulk Add Projects

```bash
# Script to add multiple projects
for dir in ~/projects/*/; do
  project_name=$(basename "$dir")
  claude-pm registry add "$project_name" "$dir"
done
```

## Tips and Tricks

1. **Use aliases** - Set up `cpc` for quick navigation
2. **Regular backups** - Backup registry periodically
3. **Organize symlinks** - Create quick-access shortcuts
4. **Shell functions** - Create custom workflows in shell
5. **Tab completion** - Set up completion for faster input

## Getting Help

```bash
# General help
claude-pm help

# Command-specific help
claude-pm registry help
claude-pm symlink help

# Version information
claude-pm version
```

See TROUBLESHOOTING.md for common issues and solutions.
