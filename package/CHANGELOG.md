# Changelog

## [2.0.0] - 2026-01-30

### Added
- Rich TUI selector with FZF preview pane showing full project metadata
- Inline keybindings: R (rename), M (metadata), F (favorite), Ctrl-D (archive)
- Special entries in selector: "New Project" and "Quick Session (no project)"
- Archive/restore system with external storage (`~/.claude/archive`)
- Cleanup tools: `prune` (remove old caches/logs) and `report` (size breakdown)
- Interactive `setup` command for first-time configuration
- `shell-init` command outputs shell integration for eval
- `edit` command for interactive metadata editing
- Size warnings in preview pane (yellow >5MB, red >10MB)
- Gum-enhanced prompts with plain-text fallbacks
- Nix flake with all dependencies (jq, fzf, gum) shipped
- Configurable projects dir via `CLAUDE_PROJECTS_DIR`
- `claude()` shell wrapper with doc review and tldr indexing

### Changed
- Registry format: array-based projects with rich metadata (v2.0.0)
- Projects sorted chronologically by last_accessed
- Registry path now lives inside projects dir (not separate)
- Replaced Homebrew formula with Nix flake (ships all deps)
- All prompts use gum with read fallback (no hard gum dependency)

### Removed
- Symlink management (replaced by archive/restore)
- Homebrew formula (Nix only)
- Old key-value registry format (v1.0.0)

## [1.0.0] - 2026-01-30

### Added
- Initial release
- Registry with key-value project format
- FZF project selection (basic)
- Symlink management
- Homebrew formula and Nix package definition
