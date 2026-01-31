# Enhanced Claude Project Workflow - Complete User Guide

## 1. Introduction

### What Changed

The Claude project workflow has been **dramatically enhanced** from a simple numbered menu to an intelligent, searchable project discovery system.

**Before:**
```
1. projects-personal-finance
2. projects-research-papers
3. projects-coding-experiments
... (only 10 most recent)
```

**After:**
```
> claude
🔍 Fuzzy search, previews, categories, favorites, and smart organization
```

### Why These Changes

1. **Discoverability**: Fuzzy search lets you find any project by typing keywords, not remembering exact names
2. **Rich Context**: See descriptions, tags, categories, creation dates, and modification dates at a glance
3. **Favorites**: Star your most-used projects for lightning-fast access
4. **Categories**: Organize by type (Research, Medicine, Leisure, Productivity, Finance, Travel, Business, Stay)
5. **No Limits**: Search across ALL projects, not just the 10 most recent

### Who Benefits

- Users with 5+ projects (where naming matters)
- Users who switch between different project types
- Users who want to organize without renaming folders
- Anyone who sometimes forgets what a project is about

---

## 2. Quick Start (5 Minutes)

### Step 1: Launch the Selector

```bash
$ claude
```

You'll see a list of your projects with a search box at the top.

### Step 2: Search or Navigate

```
> claudef                          # Type to search (fuzzy)
>
│ claud-python-ml                  # Matches
│ claude-research-papers
│ claude-finance-dashboard
```

Use arrow keys to navigate up/down.

### Step 3: Select a Project

```
> claude
↓ 1. my-research-project
  2. claude-personal-finance
  3. coding-experiments

(Press Enter to select)
```

Press `Enter` to open the selected project. You'll automatically `cd` into it.

### Step 4: Try Helper Commands

**See project statistics:**
```bash
$ claude-status
```

**Mark a project as favorite:**
```bash
$ claude-favorite
```

**View project metadata:**
```bash
$ claude-info
```

---

## 3. Features Overview

### FZF Selector - Smart Project Search

**What it does:**
- Searches across all projects instantly
- Shows metadata in a preview pane (description, tags, category, status)
- Navigate with arrow keys, search with typing
- Fast, responsive, and intuitive

**Example:**
```
> machine learning
│ claude-ml-experiments
│ research-neural-networks
│ personal-data-science
```

### Browse Mode - All Folders

**What it does:**
- View ALL folders recursively (not just registered projects)
- Useful for quick access to nested directories
- Navigate with arrow keys, select with Enter

**Shortcut:** `Ctrl-B` in the selector

**Use case:** Finding unregistered folders or exploring your entire projects directory.

### Favorites - Quick Access

**What it does:**
- Star your most-used projects
- Access them instantly with `Ctrl-F`
- Only shows your favorite projects in this mode

**How to add/remove:**
```bash
$ claude-favorite        # Toggle favorite for current project
```

### Categories - Organized Discovery

**What it does:**
- Filter projects by type (Research, Medicine, Leisure, Productivity, Finance, Travel, Business, Stay)
- Create custom categories
- Filter with `Ctrl-C` in the selector

**Categories:**
- **Research** - Academic work, literature review, experiments
- **Medicine** - Health-related projects
- **Leisure** - Hobbies, personal interests
- **Productivity** - Work tools, automation
- **Finance** - Budget tracking, investments
- **Travel** - Trip planning, location research
- **Business** - Professional work, clients
- **Stay** - Home, personal admin

### Helper Commands - Project Management

**Use these without launching the full selector:**

```bash
claude           # Launch project selector (default)
claude --new     # Create new project
claude --help    # Show help

claude-favorite  # Toggle favorite for current project
claude-info      # Show detailed metadata for current project
claude-list mode # List projects (quick/favorite/category/browse)
claude-status    # Show statistics about your projects
```

---

## 4. Keyboard Reference (Detailed)

### In the Selector

| Key | Action | Example |
|-----|--------|---------|
| **Type** | Fuzzy search | `> ml` matches "machine-learning" |
| **↑ / ↓** | Navigate list | Move cursor up/down |
| **Ctrl-F** | Toggle favorites | Show only starred projects |
| **Ctrl-C** | Filter by category | Choose a category |
| **Ctrl-B** | Browse all folders | Explore directory tree |
| **Ctrl-S** | Toggle preview pane | Hide/show metadata preview |
| **Enter** | Select project | Open selected project, cd into it |
| **Escape** | Cancel | Return to shell, don't select anything |

### Examples

**Find a project by typing:**
```
> finance              # Finds: "claude-personal-finance", "finance-tracker"
> ml                   # Finds: "machine-learning", "ml-experiments"
> 2024                 # Finds: projects with "2024" in name or metadata
```

**Switch modes in selector:**
```
While in selector:
- Ctrl-F: Show only favorites
- Ctrl-C: Filter by category
- Ctrl-B: Browse all folders
- Ctrl-S: Show/hide preview pane
```

---

## 5. CLI Commands (Detailed)

### claude - Main Command

**Launch the project selector:**
```bash
$ claude
```

**Create a new project:**
```bash
$ claude --new
# Prompts for project name, description, category
```

**Show help:**
```bash
$ claude --help
```

### claude-favorite - Favorite Management

**Toggle favorite status for current project:**
```bash
$ cd ~/.claude/projects/my-project
$ claude-favorite
# Output: "✓ Added to favorites" or "✓ Removed from favorites"
```

### claude-info - Project Information

**Show detailed metadata for current project:**
```bash
$ cd ~/.claude/projects/my-project
$ claude-info
```

**Output:**
```
Project: my-research-project
Description: Literature review and experiment notes
Tags: research, machine-learning, 2024
Category: Research
Status: active
Created: 2024-01-15
Last accessed: 2026-01-30
Sessions: 42
Git: https://github.com/user/my-research-project
```

### claude-list - List Projects

**List in different modes:**
```bash
$ claude-list quick      # Recent projects
$ claude-list favorite   # Only favorites
$ claude-list category   # Filter by category
$ claude-list browse     # All folders, browsable
$ claude-list            # Default (quick mode)
```

### claude-status - Statistics

**Show workflow statistics:**
```bash
$ claude-status
```

**Output:**
```
📊 Claude Project Workflow - Statistics

Total Projects: 15
Active: 12
Paused: 2
Archived: 1

Most Accessed: research-papers (62 sessions)
Least Accessed: archived-project (1 session)

Favorites: 5
Categories: 8

Recent Activity:
- my-current-project (5 hours ago)
- machine-learning-exp (2 days ago)
- finance-tracker (1 week ago)
```

---

## 6. Workflows (Common Tasks)

### Create a New Project

```bash
$ claude --new
# Prompts for:
# - Project name
# - Description
# - Category (Research/Medicine/Leisure/etc)
#
# Creates folder in ~/.claude/projects/
# Auto-registers in .registry.json
# Ready to use immediately
```

### Open Your Most Recent Project

```bash
$ claude
# Shows recent projects at top
# Press Enter on first one
```

### Open Your Favorite Projects

```bash
$ claude
# (In selector) Ctrl-F
# Shows only favorited projects
# Select with Enter
```

### Browse All Folders

```bash
$ claude
# (In selector) Ctrl-B
# Shows all folders recursively
# Navigate and select with arrow keys/Enter
```

### Star a Project as Favorite

```bash
$ cd ~/.claude/projects/my-project
$ claude-favorite
# Output: ✓ Added to favorites
```

### View Project Information

```bash
$ cd ~/.claude/projects/my-project
$ claude-info
# Shows: description, tags, category, dates, git link
```

### See All Your Projects

```bash
$ claude-status --detailed
# Lists all projects with activity stats
```

### Organize by Category

```bash
$ claude
# (In selector) Ctrl-C
# Choose a category to filter
# Only shows projects in that category
```

---

## 7. Metadata System

### What Gets Tracked

For each project, the system stores 9 pieces of metadata:

1. **display_name** - Human-readable project name
2. **description** - What the project is about
3. **tags** - Searchable keywords (array)
4. **category** - Type (Research, Medicine, Leisure, etc.)
5. **status** - active, paused, or archived
6. **created** - Creation timestamp
7. **last_accessed** - When you last opened it
8. **session_count** - How many times you've opened it
9. **git_link** - GitHub/GitLab URL if available

### Where It's Stored

**Central registry:**
```
~/.claude/projects/.registry.json
```

**Per-project overrides:**
```
~/.claude/projects/{project-name}/PROJECT.json
```

### How to Edit Metadata

**Use helper commands:**
```bash
# From within a project:
$ claude-favorite        # Toggle favorite

# Edit via command (planned):
$ claude-set-category Research
$ claude-set-description "My awesome project"
```

**Manual editing (advanced):**
```bash
# Edit registry directly (be careful!):
$ nano ~/.claude/projects/.registry.json

# Example entry:
{
  "id": "my-project",
  "display_name": "My Project",
  "description": "Project description",
  "tags": ["tag1", "tag2"],
  "category": "Research",
  "status": "active",
  "created": "2026-01-15",
  "last_accessed": "2026-01-30",
  "session_count": 42,
  "favorite": true,
  "git_link": "https://github.com/user/my-project"
}
```

**If registry is corrupted:**
```bash
$ ~/.claude/scripts/registry-recover.sh list
# Shows all backups

$ ~/.claude/scripts/registry-recover.sh restore-latest
# Restores from most recent backup
```

---

## 8. Troubleshooting

### FZF Not Working

**Symptom:** `fzf: command not found`

**Solution:**
```bash
# Install FZF
$ nix profile install nixpkgs#fzf

# Reload shell
$ exec zsh
```

### Selector Shows Wrong Projects

**Symptom:** Missing projects or old names

**Solution:**
```bash
# Rebuild registry
$ ~/.claude/scripts/registry-init.sh --force

# This re-scans all projects and updates metadata
```

### Symlinks Not Updating

**Symptom:** Favorites or active projects not in symlink directories

**Solution:**
```bash
# Reorganize symlinks
$ ~/.claude/scripts/symlink-organize.sh

# Verify symlinks
$ ls -la ~/.claude/projects/active/
$ ls -la ~/.claude/projects/favorites/
```

### Registry Corrupted

**Symptom:** JSON parse errors, selector crashes

**Solution:**
```bash
# List all backups
$ ~/.claude/scripts/registry-recover.sh list

# Restore most recent backup
$ ~/.claude/scripts/registry-recover.sh restore-latest

# Or restore specific backup
$ ~/.claude/scripts/registry-recover.sh restore 1234567890
```

### Helper Commands Not Found

**Symptom:** `claude-favorite: command not found`

**Solution:**
```bash
# Reload shell
$ exec zsh

# Or source .zshrc
$ source ~/.zshrc

# Verify commands are available
$ type claude-favorite
```

### Preview Pane Not Showing

**Symptom:** Selector works but no metadata visible

**Solution:**
```bash
# In selector, press Ctrl-S to toggle preview pane
# Make sure terminal is wide enough (>80 columns)

# Or rebuild preview data:
$ ~/.claude/scripts/project-preview.sh
```

---

## 9. Advanced Topics

### Editing Registry Directly

**Registry structure (.registry.json):**
```json
{
  "metadata": {
    "version": "1.0",
    "last_updated": "2026-01-30T12:00:00Z",
    "categories": ["Research", "Medicine", "Leisure", "Productivity", "Finance", "Travel", "Business", "Stay"]
  },
  "projects": [
    {
      "id": "project-folder-name",
      "display_name": "Project Name",
      "description": "What it's about",
      "tags": ["keyword1", "keyword2"],
      "category": "Research",
      "status": "active",
      "created": "2026-01-15",
      "last_accessed": "2026-01-30",
      "session_count": 42,
      "favorite": true,
      "git_link": "https://github.com/user/project"
    }
  ]
}
```

**To edit:**
```bash
# Backup first
$ cp ~/.claude/projects/.registry.json ~/.claude/projects/.registry.json.backup

# Edit
$ nano ~/.claude/projects/.registry.json

# Validate
$ ~/.claude/scripts/registry-recover.sh validate
```

### Custom Categories

**Add to registry:**
```json
"metadata": {
  "version": "1.0",
  "categories": [
    "Research", "Medicine", "Leisure", "Productivity", "Finance", "Travel", "Business", "Stay",
    "CustomCategory1", "CustomCategory2"
  ]
}
```

Then assign projects to custom categories in the registry.

### Project Statuses

**What each status means:**

- **active** - In use, appears in quick mode and searches
- **paused** - Not currently active, still searchable, lower priority
- **archived** - Completed or no longer needed, usually hidden

**To change status:**
```bash
# Via registry (edit manually)
$ nano ~/.claude/projects/.registry.json
# Change "status": "active" to "archived"

# Verify
$ ~/.claude/scripts/registry-recover.sh validate
```

### Session Tracking

**The system automatically tracks:**
- Last access timestamp (`last_accessed`)
- Number of times opened (`session_count`)

**Used by:**
- Quick mode (shows most recently accessed projects first)
- Statistics (`claude-status` shows most accessed projects)

### Rollback to Old System

**If you need to go back to the old numbered menu:**
```bash
$ ~/.claude/scripts/rollback.sh

# This restores your original .zshrc
# And optionally removes registry/symlinks
```

**To re-enable new system:**
```bash
$ ~/.claude/scripts/migrate.sh --force
```

---

## 10. FAQ

### Q: How do I add a new project?

**A:** Use the `claude --new` command:
```bash
$ claude --new
# Follow prompts for name, description, category
```

Or manually:
```bash
$ mkdir -p ~/.claude/projects/my-new-project
$ cd ~/.claude/projects/my-new-project
$ git init
# ... do project setup ...

# Then rebuild registry:
$ ~/.claude/scripts/registry-init.sh
```

### Q: Can I edit project metadata?

**A:** Yes! Three ways:

1. **Helper commands** (easiest):
   ```bash
   $ cd ~/path/to/project
   $ claude-favorite     # Toggle favorite
   ```

2. **Edit registry** (intermediate):
   ```bash
   $ nano ~/.claude/projects/.registry.json
   # Edit the project entry, save, validate
   ```

3. **Per-project override** (advanced):
   ```bash
   $ cat > ~/.claude/projects/my-project/PROJECT.json <<EOF
   {
     "description": "Updated description",
     "tags": ["newtag1", "newtag2"],
     "category": "Research"
   }
   EOF
   ```

### Q: What if I break the registry?

**A:** Easy recovery:
```bash
# List all backups
$ ~/.claude/scripts/registry-recover.sh list

# Restore from most recent backup
$ ~/.claude/scripts/registry-recover.sh restore-latest

# Or restore from specific point in time
$ ~/.claude/scripts/registry-recover.sh restore 1234567890
```

### Q: How do I hide archived projects?

**A:** Change their status in the registry:
```bash
$ nano ~/.claude/projects/.registry.json

# Find the project and change:
"status": "archived"

# Archived projects still appear in searches but not in quick mode
```

### Q: Can I use symlinks to projects outside ~/.claude/projects/?

**A:** Yes, but with limitations:
- Projects must be in ~/.claude/projects/ for registration
- You can use symlinks in /active/ or /favorites/ to point elsewhere
- Manual symlink management is supported

```bash
$ ln -s /path/to/external/project ~/.claude/projects/symlink-name
# Then rebuild registry:
$ ~/.claude/scripts/registry-init.sh
```

### Q: How often is the registry updated?

**A:** It's updated when you:
- Launch the project selector (`claude`)
- Run helper commands (`claude-favorite`, etc.)
- Run migration script
- Manually run `registry-update.sh`

The system does NOT auto-update in the background. Updates are on-demand.

### Q: Can I share my project registry?

**A:** Yes, but carefully:
- Registry contains only metadata (names, descriptions, paths)
- Does NOT contain project code or secrets
- Safe to share the `.registry.json` file
- Each user should have their own `.registry.json` to reflect their own projects

### Q: What's the difference between .registry.json and PROJECT.json?

**A:**

- **.registry.json** - Central metadata store for ALL projects
- **PROJECT.json** - Per-project overrides and local settings

If a project has PROJECT.json, its values override the registry values.

### Q: How do I delete a project?

**A:**
```bash
# Remove from registry
$ nano ~/.claude/projects/.registry.json
# Find and delete the project entry

# Remove folder (careful!)
$ rm -rf ~/.claude/projects/my-old-project

# Cleanup symlinks
$ rm ~/.claude/projects/active/my-old-project
$ rm ~/.claude/projects/favorites/my-old-project
```

---

## 11. Getting Help

### Online Resources

- **Registry troubleshooting:** Run `~/.claude/scripts/registry-recover.sh` for diagnostics
- **Verification steps:** See VERIFICATION.md
- **Troubleshooting guide:** See TROUBLESHOOTING.md
- **Architecture details:** See ARCHITECTURE.md

### Quick Reference

See QUICK_REFERENCE.txt for a one-page cheat sheet of all commands and shortcuts.

### Command Help

```bash
$ claude --help
$ registry-recover.sh help
$ project-select.sh help
```

### Reporting Issues

If something isn't working:

1. **Run diagnostics:**
   ```bash
   $ ~/.claude/scripts/registry-recover.sh validate
   ```

2. **Check logs:**
   ```bash
   $ cat ~/.claude/migration-*.log
   ```

3. **Restore from backup:**
   ```bash
   $ ~/.claude/scripts/registry-recover.sh restore-latest
   ```

---

## 12. Tips & Tricks

### Fast Project Switching

Instead of full selector, use fuzzy matching:
```bash
$ claude
> finance
# Matches finance-related projects instantly
# Press Enter to select
```

### Organizing by Type

Use categories to organize large project collections:
```bash
# View only research projects:
$ claude
> Ctrl-C (select category)
> Research
```

### Finding Forgotten Projects

Use `claude-status` to see all projects with activity:
```bash
$ claude-status
# Shows: most accessed, least accessed, recent activity
```

### Quickly Mark Favorites

For projects you use daily:
```bash
$ cd ~/.claude/projects/frequent-project
$ claude-favorite      # Now in favorites
$ claude              # Ctrl-F to see favorites only
```

### Backup Your Registry

Backups are automatic, but you can also manual backup:
```bash
$ cp ~/.claude/projects/.registry.json ~/.claude/projects/.registry.json.manual-backup
```

### Clean Up Old Backups

```bash
# See old backups
$ ~/.claude/scripts/registry-recover.sh list

# Clean backups older than 30 days (dry-run)
$ ~/.claude/scripts/registry-recover.sh cleanup 30

# Actually delete them
$ ~/.claude/scripts/registry-recover.sh cleanup-execute 30
```

---

## 13. Performance Tips

### Selector Speed

If the selector is slow:

1. **Check system load:**
   ```bash
   $ top -l1 | head -10
   ```

2. **Rebuild registry:**
   ```bash
   $ ~/.claude/scripts/registry-init.sh
   ```

3. **Optimize terminal:**
   - Use modern terminal emulator (iTerm2, Alacritty)
   - Increase terminal width for better FZF performance
   - Ensure FZF is installed: `nix profile install nixpkgs#fzf`

### Registry Performance

For 100+ projects:
- Registry updates still <100ms
- Searches remain responsive
- Browse mode may be slower (showing all folders)

**Optimization:**
```bash
# Use categories to narrow search
$ claude
> Ctrl-C (select category)
```

---

## Summary

The Enhanced Claude Project Workflow makes project management:

✅ **Fast** - Find any project in <2 seconds
✅ **Easy** - Fuzzy search, not memorization
✅ **Organized** - Categories, tags, favorites
✅ **Safe** - Atomic writes, automatic backups
✅ **Flexible** - Multiple modes, customizable

**Get started now:**
```bash
$ claude           # Launch selector
$ claude-status    # See statistics
$ claude-favorite  # Star your most-used projects
```

Enjoy! 🚀
