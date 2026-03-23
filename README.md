<div align="center">

# Projectwise

**One 1.3 MB binary. Zero config. Every project at your fingertips.**

[![Version](https://img.shields.io/badge/version-3.1.0-blue?style=flat-square)](package/VERSION)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange?style=flat-square)](Cargo.toml)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](package/LICENSE)
[![Tests](https://img.shields.io/badge/tests-7%20passing-brightgreen?style=flat-square)](#testing)

A Rust CLI that replaces 700 lines of shell scripts with a fast, atomic,<br>
interactive project manager for Claude Code workspaces.

</div>

---

```
$ claude
  ╭─────────────────────────────────────────────────╮
  │  Select project              3 matches / 12     │
  │                                                 │
  │ > trading-engine        Rust   ★  3 sessions    │
  │   openclaw-gateway      Python    17 sessions   │
  │   projectwise           Rust   ★  8 sessions    │
  │                                                 │
  │  enter: select  R: rename  F: fav  ctrl-d: arch │
  ╰─────────────────────────────────────────────────╯

  → cd ~/.claude/projects/trading-engine
  → axon analyze (background)
  → tldr warm (background)
  → Entering Claude Code...
```

Pick a project. Code intelligence refreshes automatically. You're coding in under two seconds.

---

## Why Projectwise Exists

If you manage more than a handful of Claude Code projects, you know the pain: `cd`-ing into the right directory, remembering which project you touched last, manually refreshing code indexes, wondering if your registry JSON got corrupted by a half-written update.

Projectwise solves all of it:

| Problem | Projectwise |
|---------|-------------|
| Hunting for project directories | FZF fuzzy picker with live preview |
| Stale code intelligence | Auto-refreshes Axon graphs + tldr indexes on every entry |
| Corrupted project registry | Atomic writes via tempfile + POSIX rename, with file locking |
| Phantom deleted directories | Integrity checker detects and repairs mismatches |
| 700 lines of fragile shell | Single 1.3 MB Rust binary, 0 compiler warnings |

---

## Install

```bash
git clone https://github.com/anoop-titus/Projectwise.git
cd Projectwise
cargo build --release
cp target/release/cpm ~/.local/bin/
```

Add to your `.zshrc` or `.bashrc`:

```bash
eval "$(cpm shell-init)"
```

Initialize the registry:

```bash
cpm registry init
```

**Dependencies:** [fzf](https://github.com/junegunn/fzf) (required). [Axon](https://github.com/harshkedia177/axon) and tldr are optional -- Projectwise degrades gracefully without them.

---

## Usage

### Select & Enter a Project

```bash
claude              # FZF picker → enters Claude Code in the selected project
cpm select          # just the picker (returns folder name)
cpm select all      # include archived projects
```

The `claude` shell wrapper uses the same pattern as [zoxide](https://github.com/ajeetdsouza/zoxide) -- a thin shell function that calls the binary and `cd`s the parent shell.

### Interactive Project Table

```bash
cpm list              # Ratatui TUI with dark theme, alternating rows
cpm list favorite     # favorites only
cpm list all          # including archived
```

Navigate with `j`/`k` or arrow keys. `Enter` for details. `q` to quit.

### Manage Projects

```bash
cpm create              # interactive creation with dialoguer prompts
cpm edit <folder>       # edit name, description, category, status, tags, git link
cpm archive <folder>    # soft-delete to archive directory
cpm restore <folder>    # bring it back
cpm delete <folder>     # permanent removal (requires 2 confirmations)
```

### Inspect

```bash
cpm preview <folder>    # styled terminal preview (powers the FZF preview pane)
cpm info <folder>       # full JSON detail
```

### Registry Operations

```bash
cpm registry add <folder> [name]       # register a project
cpm registry remove <folder>           # unregister
cpm registry list                      # list all folder names
cpm registry touch <folder>            # bump last_accessed + session_count
cpm registry toggle-fav <folder>       # toggle favorite star
cpm registry set-name <folder> <name>  # rename
cpm registry set-status <folder> <s>   # active / paused / archived
cpm registry set-tags <folder> <csv>   # comma-separated tags
```

### Integrity & Cleanup

```bash
cpm integrity check               # show registry ↔ filesystem mismatches
cpm integrity repair               # auto-fix: archive missing, add untracked

cpm cleanup prune --days 30        # remove stale .axon/.tldr caches
cpm cleanup report                 # per-project size breakdown
```

### FZF Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Select and enter project |
| `R` | Rename project |
| `F` | Toggle favorite |
| `Ctrl-D` | Archive project |
| `Esc` | Cancel |

---

## How It Works

```
eval "$(cpm shell-init)"     # emits a ~25-line claude() shell wrapper
        │
        ▼
    cpm select               # FZF picker with themed preview
        │
        ▼
    cpm pre-launch <folder>  # background: axon analyze + tldr warm
        │                    #   + registry touch + doc review prompt
        ▼
    command claude "$@"      # enters Claude Code in the project dir
```

**Pre-launch** runs code intelligence tools in background threads so they never block your workflow. Tools that aren't installed are silently skipped.

---

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CLAUDE_PROJECTS_DIR` | `~/.claude/projects` | Root directory for projects |
| `CLAUDE_ARCHIVE_DIR` | `~/.claude/archive` | Archive directory |
| `PAGER` | `less` | Pager for doc review |

---

## Architecture

```
src/
├── main.rs       # clap CLI dispatcher + all command implementations
├── models.rs     # Project, Registry, ProjectStatus, ListMode structs
├── registry.rs   # CRUD, atomic writes (tempfile → rename), backup rotation
└── theme.rs      # Ratatui dark theme: cyan / green / amber / gold
```

**Design decisions worth noting:**

- **External fzf, not skim** -- FZF keybindings shell out to `cpm` for mutations, then reload the list. This keeps the picker and the data layer cleanly separated.
- **Atomic writes** -- every registry mutation writes to a tempfile in the same directory, then calls `fs::rename` (POSIX atomic). No partial writes, ever.
- **Backup rotation** -- the last 10 timestamped registry snapshots live in `.backups/`.
- **Graceful degradation** -- `cmd_exists()` checks before spawning axon, tldr, or claude-context. Missing tools are silently skipped.

---

## Testing

```bash
cargo test    # 7 unit tests: registry CRUD, sorting, favorites, set_field
```

---

## Changelog

### v3.1.0

Ratatui TUI table with dark theme and keyboard navigation. Interactive `cpm edit` via dialoguer. Cleanup commands. Themed FZF colors. claude-context integration. Zero compiler warnings.

### v3.0.0

Complete Rust rewrite. Single 1.3 MB binary (stripped + LTO). Atomic registry writes with backup rotation. Integrity checker. Background code intelligence refresh. 7 unit tests.

### v2.x

Original Bash implementation. Shell cleanup, dedup fixes, FZF keybinding wiring.

---

## License

MIT -- see [LICENSE](package/LICENSE).

## Author

**Anoop Titus** -- [github.com/anoop-titus](https://github.com/anoop-titus)
