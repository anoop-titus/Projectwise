# Enhanced Claude Project Workflow

## What This Is

A CLI-based project management system that replaces the simple numbered menu in `~/.zshrc` with an intelligent fuzzy search interface. Shows project metadata (descriptions, tags, categories, timestamps) in an FZF preview pane with rich table formatting, enabling discovery of any project in <2 seconds without remembering names or numbers.

For personal use (1 person, 11+ projects); potential foundation for shareable CLI tool in v2.

## Core Value

**Fast, intuitive project discovery that doesn't require naming conventions or memory of folder structures.**

When you type `claude`, you should be able to find any project in seconds through fuzzy search or category browsing, with enough context to remember what it's about. The system should be invisible once it works — you barely think about project navigation.

## Requirements

### Validated

(None yet — ship to validate)

### Active

**Core Selection Interface:**
- [ ] FZF-based fuzzy selector replacing numbered menu in `.zshrc`
- [ ] Keyboard navigation with arrow keys (↑↓) for all interactions
- [ ] Escape key to cancel, Enter to select, type to search
- [ ] FZF preview pane showing tabular view: Name | Created | Modified | Category | Status | Git Link

**Metadata & Registry:**
- [ ] `.registry.json` as central metadata store with 9 fields per project:
  - display_name, description, tags[], category, status, created, last_accessed, session_count, git_link
- [ ] Per-project `PROJECT.json` template for local overrides
- [ ] Registry scanner: auto-index 11 existing projects on first run
- [ ] Auto-generate sensible display names from path-encoded folder names

**Multiple Selection Modes:**
- [ ] Quick mode: Recent projects + full fuzzy search (default)
- [ ] Favorites mode: `Ctrl-F` shows only starred projects
- [ ] Category mode: `Ctrl-C` filters by category (8 defaults + custom)
- [ ] Browse mode: `Ctrl-B` shows ALL folders recursively with folder-level metadata

**Helper Commands:**
- [ ] `claude-favorite`: Toggle favorite for current project
- [ ] `claude-info`: Display metadata for current project
- [ ] `claude-list [mode]`: List projects with filtering (quick/browse/favorite/category)
- [ ] `claude-status`: Show project statistics and summary

**Organization & Navigation:**
- [ ] `~/.claude/projects/active/` — symlinks to active projects (managed automatically)
- [ ] `~/.claude/projects/favorites/` — symlinks to favorite projects (user-managed)
- [ ] Category system: Research, Medicine, Leisure, Productivity, Finance, Travel, Business, Stay + custom
- [ ] Project statuses: active, paused, archived

**Integration:**
- [ ] Update `.zshrc` claude() function to use new selector instead of numbered menu
- [ ] Registry auto-updates last_accessed on project selection
- [ ] One-time migration script for existing projects
- [ ] Simple rollback mechanism (backup old .zshrc section)

**Backwards Compatibility:**
- [ ] All 11 existing projects continue working unchanged
- [ ] No breaking changes to `.claude/projects/` structure
- [ ] Registry is additive (projects work without metadata)
- [ ] Can disable new system and revert to old menu instantly

### Out of Scope

- **GUI/web interface** — Pure CLI only, no graphical components
- **Sync/cloud features** — Local filesystem only, no remote sync
- **Data analytics/stats** — No usage tracking, metrics, or dashboards
- **Project creation flow** — `claude --new` handles that separately
- **Homebrew/nixpkgs distribution** — Defer to v2 after validating locally
- **Full package release** — v1 is integrated in .zshrc, v2 considers standalone packaging
- **Nested project hierarchies** — Flat structure only (one level of projects)
- **Project duplication/templates** — Not in scope, use external tools

## Context

**Current State:**
- Numbered menu in `.zshrc` (lines 450-572) showing last 10 projects only
- Projects stored in `~/.claude/projects/` with path-encoded folder names (e.g., `-Users-titus--claude-python`)
- Minimal metadata tracking (only filesystem timestamps)
- No search, filtering, or categorization

**User Needs:**
- Can find any project in <2 seconds without thinking about navigation
- Quick access to frequently-used projects (favorites)
- Organize growing project collection by type
- Rich context visible (description, tags, when created/modified, git link if available)
- Keyboard-first experience (arrow keys, simple navigation)

**Existing Infrastructure:**
- FZF already installed via Nix
- Gum needs installation: `nix profile install nixpkgs#gum`
- jq available for JSON processing
- All shell tools integrated into .zshrc

**Prior Work:**
- Detailed implementation plan created with 5 phases
- File structure and architecture documented
- Script templates sketched out
- Categories and metadata schema defined

## Constraints

- **Integration Point**: Must work within existing `.zshrc` claude() function wrapper
- **Backwards Compatible**: Cannot break 11 existing projects in `~/.claude/projects/`
- **Simple Rollback**: If system fails, reverting to old menu must take <5 minutes
- **Tool Availability**: Only use tools available via Nix (or Homebrew as fallback)
- **No Breaking Changes**: Existing project folders, SESSION INDEX, or .cache structure must remain untouched
- **Performance**: Project selection should complete in <2 seconds (including registry read)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Bash + Gum + FZF (not Go/Rust) | User preferred keeping implementation simple and shell-based; no need for compiled binary yet | — Pending |
| Defer packaging to v2 | First validate the system locally works well, then decide if standalone distribution matters | — Pending |
| Arrow keys (not Vim/Tab) | Most intuitive for mixed audience, easier to teach than hjkl or sequential tab navigation | — Pending |
| Rich metadata (9 fields) | Enables discovery via context; session count and git link support future analytics if needed | — Pending |
| Symlinks for favorites/active | Filesystem-native way to organize; complements registry metadata; trivial to implement | — Pending |
| Custom categories allowed | 8 defaults cover most use cases; users can define custom for specific needs | — Pending |
| FZF preview (not separate command) | Keeps selection interface unified; table view in preview avoids context switching | — Pending |

---

*Last updated: 2026-01-30 after initialization*
