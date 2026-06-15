<div align="center">

# Projectwise

**One small binary. Zero config. Every Claude Code project — and its governance — at your fingertips.**

[![Version](https://img.shields.io/badge/version-3.7.1-blue?style=flat-square)](Cargo.toml)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange?style=flat-square)](Cargo.toml)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](package/LICENSE)
[![Tests](https://img.shields.io/badge/tests-18%20passing-brightgreen?style=flat-square)](#testing)

A Rust CLI/TUI that replaces fragile shell scripts with a fast, atomic,<br>
interactive manager for Claude Code workspaces — now with a tabbed cockpit
for projects, the Tokenizer optimizer, your global/project `CLAUDE.md`, and
your `RULES`.

</div>

---

Pick a project. Code intelligence refreshes, the latest PROGRESS + ARCHITECTURE
get absorbed into Claude's context, and Tokenizer keeps your `~/.claude` lean —
all in under two seconds.

---

## Why Projectwise Exists

If you manage more than a handful of Claude Code projects, you know the pain:
`cd`-ing into the right directory, remembering which project you touched last,
manually refreshing code indexes, and re-explaining the project to Claude every
session. Projectwise solves all of it:

| Problem | Projectwise |
|---------|-------------|
| Hunting for project directories | FZF fuzzy picker + in-TUI `/` search (scoped to `~/.claude/projects`) |
| Stale code intelligence | Auto-refreshes Axon graphs + tldr indexes on entry |
| Claude starts each session "cold" | Auto-absorbs the project's PROGRESS + ARCHITECTURE digest as the opening prompt |
| Corrupted project registry | Atomic writes via tempfile + POSIX rename, with file locking |
| Phantom deleted directories | Integrity checker detects and repairs mismatches |
| Bloated `~/.claude` token cost | Bundled Tokenizer tab + auto-boost compress rules to `.toon` |
| Governance files are hard to read/edit | Tabs for `CLAUDE.md` and `RULES`, with a readable `.md` ⇄ `.toon` round-trip |

---

## Features (v3.7.1)

**Tabbed TUI cockpit** — switch with `[` / `]` or `1`–`4`:

1. **Projects** — 3-panel layout (list + directory tree + info), 40 opaline
   themes (`t`), mouse support, dashboard (`d`), and `/` fuzzy search over
   project folders.
2. **Tokenizer** — status of the bundled [Tokenizer](https://github.com/anoop-titus/Tokenizer)
   optimizer (binary / launch timer / session hook) and one-key actions: run
   optimize (`o`), open the full Tokenizer TUI (`T`), install timer (`i`) /
   hook (`h`). Tokenizer is also auto-installed + boosted on every summon.
3. **CLAUDE.md** — view/scroll your **global** (`~/.claude/CLAUDE.md`) and
   **project** (`~/CLAUDE.md`) instructions; toggle with `g` / `p`, edit in
   `$EDITOR` with `e`.
4. **RULES** — browse every rule in `~/.claude/rules/` rendered **human-readable**,
   and edit it (`e`). Edits go to a readable `.md` master in `~/.claude/rules-md/`
   and are recompiled back to the token-optimized `.toon` that Claude loads.

**Context absorption** — on launch, Projectwise writes a `context-digest.md`
(latest PROGRESS pulled from `~/.claude/progress/tasks.json` + `ARCHITECTURE.md`)
and passes it to Claude as the opening prompt, alongside a "interview me for the
real goal / small specs" directive.

**Plus the classics** — atomic registry (tempfile + POSIX rename, backup
rotation, file locking), integrity checker, background code-intelligence refresh,
and a single small Rust binary with zero compiler warnings.

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

This emits the `projectwise` / `clauded` shell functions (thin wrappers that
call the binary and `cd` the parent shell, same pattern as
[zoxide](https://github.com/ajeetdsouza/zoxide)). On each summon they also
ensure Tokenizer is installed + fire a background optimize, and keep the
readable rule mirror fresh — all fail-open.

Initialize the registry and build the rule mirror:

```bash
cpm registry init
cpm rules-sync
```

### Prerequisites

| Tool | Required for | Behavior if missing |
|------|--------------|---------------------|
| [`fzf`](https://github.com/junegunn/fzf) | project picker + `/` search | picker disabled |
| `claude` (Claude Code CLI) | launching sessions | wrapper aborts with a message |
| [`tokenizer`](https://github.com/anoop-titus/Tokenizer) | Tokenizer tab + auto-boost | tab shows "missing"; launch still works (fail-open) |
| `md_to_json` + `toon` ([`@toon-format/cli`](https://www.npmjs.com/package/@toon-format/cli)) | RULES `.md` → `.toon` round-trip on save | edit saves the `.md` master; the old `.toon` is preserved (no data loss) |
| Axon / tldr / claude-context | background code-intelligence refresh | silently skipped |

Override tool locations with `TOKENIZER_BIN`, `MD_TO_JSON_BIN`, `TOON_BIN`.

### Platform

Targeted at **macOS and Linux**. The readable-rule mirror uses symlinks
(`#[cfg(unix)]`); on non-unix the `.toon` symlinks are skipped but the `.md`
masters and round-trip still work.

---

## Usage

### Select & enter a project

```bash
projectwise          # TUI picker → enters Claude Code in the selected project
clauded              # alias — same behavior
cpm select           # FZF picker (returns folder name)
cpm select all       # include archived projects
```

### Interactive cockpit

```bash
cpm list              # tabbed Ratatui TUI (Projects · Tokenizer · CLAUDE.md · RULES)
cpm list favorite     # favorites only
cpm list all          # including archived
```

**TUI keybindings:**

| Key | Action |
|-----|--------|
| `[` / `]` or `1`–`4` | Switch tab |
| `j`/`k` or arrows | Navigate / scroll |
| `/` | Fuzzy-search project folders (Projects tab) |
| `Enter` | Select and enter project |
| `t` | Cycle 40 opaline themes |
| `r` / `x`·`Del` | Rename / delete project (Projects tab) |
| `d` | Dashboard (heatmap, charts, sparkline) |
| `g` / `p` | Global / project file (CLAUDE.md tab) |
| `e` | Edit current file in `$EDITOR` (CLAUDE.md / RULES tabs) |
| `o` `T` `i` `h` | Optimize / open TUI / install timer / install hook (Tokenizer tab) |
| Mouse / scroll | Select / navigate |
| `q` / `Esc` | Quit |

### Rules round-trip

```bash
cpm rules-sync   # build/refresh ~/.claude/rules-md/ readable masters,
                 # make ~/.claude/rules/ strictly .toon-only, prune dead symlinks
```

In the RULES tab, `e` edits the readable master in `~/.claude/rules-md/`; on
save it recompiles to `~/.claude/rules/<rule>.toon` (via `md_to_json | toon`).
`rules-sync` is idempotent and self-healing: it prunes broken `.toon` symlinks
and reports orphaned masters without ever deleting a master or resurrecting a
deprecated rule.

### Manage / inspect

```bash
cpm create | edit <f> | archive <f> | restore <f> | delete <f>
cpm preview <folder>    # styled preview (powers the FZF preview pane)
cpm info <folder>       # full JSON detail
```

### Registry / integrity / cleanup

```bash
cpm registry add|remove|list|touch|toggle-fav|set-name|set-status|set-tags ...
cpm integrity check | repair
cpm cleanup prune --days 30 | report
```

---

## How It Works

```
eval "$(cpm shell-init)"        # emits projectwise() + clauded() wrappers
        |
        v
  _projectwise_tokenizer        # ensure Tokenizer timer/hook installed + boost (bg)
  cpm rules-sync (bg)           # keep readable rule mirror fresh
        |
        v
  cpm list --select             # tabbed TUI picker
        |
        v
  cpm pre-launch <folder>       # bg: axon analyze + tldr warm + registry touch
        |                       #   + write .projectwise/context-digest.md
        v
  claude "<interview + digest>" # enters Claude Code with PROGRESS+ARCHITECTURE in context
```

Pre-launch runs code-intelligence tools in background threads so they never
block. Missing tools are silently skipped (`cmd_exists()` gate).

---

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CLAUDE_PROJECTS_DIR` | `~/.claude/projects` | Root directory for projects |
| `CLAUDE_ARCHIVE_DIR` | `~/.claude/archive` | Archive directory |
| `TOKENIZER_BIN` | `~/.cargo/bin/tokenizer` | Tokenizer binary |
| `MD_TO_JSON_BIN` | `~/.local/bin/md_to_json` | Markdown→JSON converter |
| `TOON_BIN` | (PATH / newest nvm) | TOON encoder CLI |
| `VISUAL` / `EDITOR` | `vi` | Editor for the CLAUDE.md / RULES tabs |

---

## Architecture

```
src/
├── main.rs       # clap CLI dispatcher + tabbed Ratatui TUI + all commands
├── models.rs     # Project, Registry, ProjectStatus, ListMode
├── registry.rs   # CRUD, atomic writes (tempfile → rename), backup rotation
├── sessions.rs   # session logging + activity stats (dashboard)
├── filetree.rs   # directory-tree widget (Projects tab)
└── theme.rs      # 40 opaline themes with live switching
```

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full component diagram.

**Design decisions worth noting:**

- **External fzf, not skim** — FZF keybindings shell out to `cpm` for mutations,
  then reload the list, keeping picker and data layer separate.
- **Atomic writes everywhere** — registry mutations and `.toon` compilation both
  write to a temp file then `fs::rename` (POSIX atomic). No partial writes.
- **Reuse, don't reinvent** — the RULES round-trip uses the same
  `md_to_json | toon -e -o` pipeline as the user's own conversion hook.
- **Safety first** — a rule's `.md` is removed from `~/.claude/rules/` only after
  a non-empty `.toon` is written; on converter failure the master is kept and the
  previous `.toon` preserved.
- **Graceful degradation** — every external tool is `cmd_exists()`-gated and
  fail-open; a missing tool never blocks launching Claude.

---

## Testing

```bash
cargo test     # 18 unit tests: registry CRUD/sorting/favorites, tab cycling,
               # toon_to_readable, strip_html, titlecase, path-traversal guards,
               # multibyte cursor, converter env-override + safety guard
cargo clippy   # 0 warnings
```

End-to-end CLI behavior is covered by a headless sweep (version, shell-init,
registry/integrity/cleanup, rules-sync idempotency, mirror invariants, md→toon
round-trip, context digest, path-safety). Interactive TUI flows are validated by
a manual smoke pass.

---

## Changelog

### v3.7.1
PROD hardening: panic fixes (home-dir fallbacks, multibyte cursor, picker
bounds), self-healing `rules-sync` (prune dead symlinks, report orphans),
clippy at 0, tests 12 → 18, full E2E sweep.

### v3.7.0
Readable `.md` rule mirror (`~/.claude/rules-md/`) with `.toon` round-trip;
`~/.claude/rules/` becomes strictly `.toon`-only; `cpm rules-sync`.

### v3.6.0
`CLAUDE.md` and `RULES` tabs (readable view + `$EDITOR` editing); three global
working rules; session "interview me / small specs" prompt on launch.

### v3.5.0
`/` fuzzy search in the TUI; Tokenizer tab + auto-install/boot/boost; auto-absorb
PROGRESS + ARCHITECTURE into Claude's context via a launch digest.

### v3.3.0
40 opaline themes, mouse support, interactive overlays, TUI rename/delete,
3-panel layout, dashboard with heatmap/charts/sparkline.

### v3.0.0
Complete Rust rewrite. Single small binary (stripped + LTO). Atomic registry,
integrity checker, background code intelligence.

---

## License

MIT — see [LICENSE](package/LICENSE).

## Author

**Anoop Titus** — [github.com/anoop-titus](https://github.com/anoop-titus)
