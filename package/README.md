# Claude Project Manager

A TUI-based project manager for Claude Code. FZF-powered selector with rich metadata preview, inline editing, archive/restore, and shell integration.

## Features

- **Interactive TUI Selector**: FZF with metadata preview pane, keybindings for rename/edit/favorite/archive
- **Rich Project Registry**: Display name, description, category, tags, status, session count, favorites
- **Archive/Restore**: Move projects to external archive, restore when needed
- **Cleanup Tools**: Prune old caches/logs, size reports per project
- **Shell Integration**: `claude()` wrapper with project selector, doc review, tldr indexing
- **Gum Optional**: Enhanced TUI with [gum](https://github.com/charmbracelet/gum), plain-text fallbacks without it

## Install

### Nix (recommended — ships all dependencies)

```bash
nix profile install github:titus/claude-project-manager
```

### From source

```bash
git clone https://github.com/titus/claude-project-manager.git
cd claude-project-manager
./scripts/install.sh
```

### Dependencies

| Dependency | Required | Shipped via Nix |
|------------|----------|-----------------|
| bash 4.0+  | Yes      | Yes             |
| jq         | Yes      | Yes             |
| fzf        | Yes      | Yes             |
| gum        | No       | Yes             |

## Setup

```bash
claude-pm setup
```

This prompts you to choose your projects directory:
1. **Default** (`~/.claude/projects`)
2. **Custom path** (enter or drag-drop a folder)
3. **Current directory**

Then add to your `~/.zshrc` or `~/.bashrc`:

```bash
export CLAUDE_PROJECTS_DIR="$HOME/.claude/projects"
eval "$(claude-pm shell-init)"
```

## Usage

### TUI Selector (default command)

```bash
claude-pm              # Opens FZF selector
```

**Keybindings in selector:**

| Key | Action |
|-----|--------|
| Enter | Select project |
| R | Rename display name |
| M | Edit full metadata |
| F | Toggle favorite |
| Ctrl-D | Archive project |
| Esc | Cancel |

Special entries at bottom: **New Project**, **Quick Session (no project)**

### Commands

```bash
claude-pm select           # TUI selector
claude-pm list             # Table of active projects
claude-pm list all         # Include archived
claude-pm create           # Interactive new project
claude-pm edit <folder>    # Edit metadata
claude-pm info <folder>    # Full project details
claude-pm preview <folder> # Compact preview

claude-pm archive <folder> # Archive project
claude-pm restore <folder> # Restore from archive
claude-pm delete <folder>  # Permanent removal

claude-pm cleanup report   # Size breakdown
claude-pm cleanup prune    # Remove old caches (--days 30)

claude-pm registry init    # Initialize registry
claude-pm registry help    # Registry subcommands

claude-pm setup            # First-time setup
claude-pm shell-init       # Output shell integration code
claude-pm help             # Full help
```

### Shell Integration

After `eval "$(claude-pm shell-init)"`, typing `claude` will:

1. Open the TUI project selector
2. On select: `cd` into the project
3. Run `tldr warm .` if index doesn't exist
4. Offer to review project docs (PROJECT.md, README.md, etc.)
5. Launch `claude` CLI

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLAUDE_PROJECTS_DIR` | `~/.claude/projects` | Projects directory |
| `CLAUDE_ARCHIVE_DIR` | `~/.claude/archive` | Archive directory |

## Registry Format

```json
{
  "version": "2.0.0",
  "projects": [
    {
      "id": "my-project",
      "folder_name": "my-project",
      "display_name": "My Project",
      "description": "A cool project",
      "tags": ["web", "api"],
      "category": "Development",
      "status": "active",
      "created": "2026-01-30T00:00:00Z",
      "last_accessed": "2026-01-30T12:00:00Z",
      "session_count": 5,
      "git_link": "https://github.com/user/repo",
      "favorite": true
    }
  ]
}
```

## Development

```bash
# Dev shell with all deps + shellcheck + bats
nix develop

# Run tests
bats tests/*.bats

# Lint
shellcheck lib/*.sh bin/claude-pm
```

## License

MIT
