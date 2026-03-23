# Claude Project Manager (cpm)

**A Rust CLI for managing Claude Code projects with FZF selection, Ratatui TUI, and automatic code intelligence refresh.**

[![Version](https://img.shields.io/badge/Version-3.1.0-brightgreen?style=flat-square)](package/VERSION)
[![Language](https://img.shields.io/badge/Language-Rust-orange?style=flat-square)](Cargo.toml)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](package/LICENSE)

---

## Overview

`cpm` is a single Rust binary that manages Claude Code project directories. It replaces the original ~700-line shell implementation with a type-safe, fast, and polished experience.

**What it does:**
- FZF-based interactive project selector with themed preview panels
- Ratatui TUI table with dark theme, keyboard navigation, alternating rows
- Automatic code intelligence refresh (Axon + tldr) on every project entry
- Registry integrity checking — detects missing/untracked project directories
- Atomic JSON registry writes with backup rotation

**How it works:**
```
eval "$(cpm shell-init)"    <-- adds claude() shell wrapper (~25 lines)
       |
       v
   cpm select               <-- FZF picker, returns folder name
       |
       v
   cpm pre-launch <folder>  <-- axon analyze + tldr warm (background)
       |
       v
   command claude "$@"       <-- enters Claude Code in the project dir
```

The shell wrapper is necessary because no binary can `cd` the parent shell. This is the same pattern as `zoxide`.

---

## Requirements

- **Rust** 1.70+ (for building)
- **fzf** — interactive fuzzy finder
- **axon** (optional) — graph-powered code intelligence ([github.com/harshkedia177/axon](https://github.com/harshkedia177/axon))
- **tldr** (optional) — code indexing

---

## Installation

### From source (recommended)

```bash
git clone https://github.com/anoop-titus/Projectwise.git
cd Projectwise
cargo build --release
cp target/release/cpm ~/.local/bin/
```

### Add shell integration

Add to `.zshrc` or `.bashrc`:

```bash
eval "$(cpm shell-init)"
```

Then reload your shell:

```bash
source ~/.zshrc  # or ~/.bashrc
```

### Initialize the registry

```bash
cpm registry init
```

---

## Usage

### Project selection (default)

```bash
claude          # opens FZF selector, then enters Claude Code
cpm select      # just the FZF selector
cpm select all  # include archived projects
```

**FZF keybindings:**
| Key | Action |
|-----|--------|
| Enter | Select and enter project |
| R | Rename project |
| F | Toggle favorite |
| Ctrl-D | Archive project |
| Esc | Cancel |

### List projects (Ratatui TUI)

```bash
cpm list              # active projects (interactive table)
cpm list favorite     # favorites only
cpm list all          # all including archived
```

**Table keybindings:** `j`/`k` or arrows to navigate, `Enter` for details, `q` to quit.

### Project management

```bash
cpm create              # interactive project creation
cpm edit <folder>       # edit metadata (name, description, category, status, tags, git link)
cpm archive <folder>    # move to archive directory
cpm restore <folder>    # restore from archive
cpm delete <folder>     # permanent delete (2 confirmations)
```

### Project info

```bash
cpm preview <folder>    # styled terminal preview (used by FZF)
cpm info <folder>       # full JSON detail
```

### Registry operations

```bash
cpm registry init                     # create empty registry
cpm registry add <folder> [name]      # add project
cpm registry remove <folder>          # remove from registry
cpm registry list                     # list folder names
cpm registry touch <folder>           # update last_accessed + session_count
cpm registry toggle-fav <folder>      # toggle favorite
cpm registry set-name <folder> <name> # rename
cpm registry set-status <folder> <s>  # active/paused/archived
cpm registry set-tags <folder> <csv>  # comma-separated tags
```

### Integrity checking

```bash
cpm integrity check    # show registry <-> filesystem mismatches
cpm integrity repair   # auto-fix: archive missing, add untracked
```

### Cleanup

```bash
cpm cleanup prune --days 30   # remove stale .axon/.tldr caches
cpm cleanup report            # per-project size breakdown
```

### Pre-launch hooks

```bash
cpm pre-launch <folder>   # runs on every project entry:
                           #   - axon analyze (background)
                           #   - tldr warm (background)
                           #   - claude-context index (background, if installed)
                           #   - registry touch
                           #   - doc review prompt
```

### Other

```bash
cpm version       # show version
cpm shell-init    # emit claude() shell wrapper
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLAUDE_PROJECTS_DIR` | `~/.claude/projects` | Projects root directory |
| `CLAUDE_ARCHIVE_DIR` | `~/.claude/archive` | Archive directory |
| `PAGER` | `less` | Pager for doc review |

---

## Architecture

```
Projectwise/
├── Cargo.toml          # dependencies: clap, serde, ratatui, crossterm, dialoguer, colored, chrono, tempfile, walkdir, dirs, anyhow
├── src/
│   ├── main.rs         # clap CLI dispatcher + all command implementations
│   ├── models.rs       # Project, Registry, ProjectStatus, ListMode structs
│   ├── registry.rs     # CRUD, atomic writes (tempfile → persist), backup rotation, 7 unit tests
│   └── theme.rs        # Ratatui dark theme palette (cyan/green/amber/gold)
└── package/
    └── VERSION         # 3.1.0
```

**Key design decisions:**
- External `fzf` kept (not skim crate) — keybindings call back into `cpm` binary for mutations + reload
- Atomic registry writes: write to tempfile in same dir, then `fs::rename` (POSIX atomic)
- Backup rotation: last 10 timestamped backups in `.backups/`
- Background threads for axon/tldr refresh — don't block project entry
- `cmd_exists()` checks before spawning optional tools — graceful degradation

---

## Testing

```bash
cargo test    # 7 unit tests covering registry CRUD, sorting, favorites, set_field
```

---

## Version History

### v3.1.0 (2026-03-23)

- Full Ratatui TUI table for `cpm list` with dark theme and keyboard navigation
- `theme.rs` with polished cyan/green/amber palette
- Interactive `cpm edit` with dialoguer prompts
- `cpm cleanup prune` and `cpm cleanup report` commands
- Themed FZF selector with matching dark colors
- claude-context integration in pre-launch hooks
- 0 compiler warnings

### v3.0.0 (2026-03-23)

- Complete Rust rewrite replacing all shell scripts
- Single `cpm` binary (1.3MB stripped+LTO)
- Atomic registry writes with backup rotation
- Registry integrity checking (missing/untracked detection)
- Background axon + tldr refresh on every project entry
- 7 unit tests

### v2.1.0 (2026-03-23)

- Shell cleanup: source guard, dedup fix, shell injection fix, FZF keybindings wired

### v2.0.0 (2026-01-30)

- Initial production release (Bash)

---

## Author

**Anoop Titus** — [GitHub](https://github.com/anoop-titus)

## License

MIT — see [package/LICENSE](package/LICENSE)
