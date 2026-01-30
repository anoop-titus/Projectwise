# Roadmap: Enhanced Claude Project Workflow

## Overview

CLI project management system built in 5 phases: metadata infrastructure → fuzzy selection interface → folder organization → shell integration → migration and polish. Validates the approach locally before considering distribution in v2.

## Domain Expertise

None

## Phases

- [ ] **Phase 1: Registry Foundation** - Metadata storage and project scanning
- [ ] **Phase 2: FZF Selector** - Multi-mode fuzzy search interface
- [ ] **Phase 3: Browsing & Symlinks** - Folder navigation and organization
- [ ] **Phase 4: Session Integration** - .zshrc integration and auto-updates
- [ ] **Phase 5: Polish** - Migration, documentation, rollback safety

## Phase Details

### Phase 1: Registry Foundation
**Goal**: Central metadata store with 9-field schema. Auto-index 11 existing projects.
**Depends on**: Nothing (first phase)
**Research**: Unlikely (JSON schema, local filesystem operations)
**Plans**: 3 plans

Plans:
- [ ] 01-01: Create `.registry.json` schema and initialization script
- [ ] 01-02: Auto-scan existing projects and generate display names
- [ ] 01-03: Per-project `PROJECT.json` template and validation

### Phase 2: FZF Selector
**Goal**: FZF-based selection with preview pane, fuzzy search, arrow key navigation.
**Depends on**: Phase 1
**Research**: Unlikely (FZF available, CLI patterns established)
**Plans**: 3 plans

Plans:
- [ ] 02-01: Build FZF preview renderer with tabular metadata display
- [ ] 02-02: Implement multi-mode selector (quick/browse/favorite/category)
- [ ] 02-03: Add keyboard bindings (arrow keys, Escape, Enter, Ctrl shortcuts)

### Phase 3: Browsing & Symlinks
**Goal**: Folder browsing, symlink organization, rich metadata display.
**Depends on**: Phase 2
**Research**: Unlikely (folder traversal, file stats, symlink management)
**Plans**: 2 plans

Plans:
- [ ] 03-01: Folder browser with recursive navigation and info pane
- [ ] 03-02: Create `/active/` and `/favorites/` symlink directories

### Phase 4: Session Integration
**Goal**: .zshrc integration with auto-updates, helper commands, atomic registry writes.
**Depends on**: Phase 3
**Research**: Likely (shell function integration, atomic file operations, error handling)
**Research topics**: Best practices for modifying .zshrc safely, atomic JSON updates with jq, error recovery in shell functions
**Plans**: 3 plans

Plans:
- [ ] 04-01: Update `.zshrc` claude() function to use new selector
- [ ] 04-02: Create helper commands (claude-favorite, claude-info, claude-list, claude-status)
- [ ] 04-03: Implement last_accessed auto-update and atomic registry writes

### Phase 5: Polish
**Goal**: One-time migration script, documentation, rollback mechanism, testing.
**Depends on**: Phase 4
**Research**: Unlikely (migration scripts, documentation, straightforward validation)
**Plans**: 2 plans

Plans:
- [ ] 05-01: Create migration script with rollback capability and backup
- [ ] 05-02: Documentation (guide, quick reference, verification steps)

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|---|---|---|
| 1. Registry Foundation | 0/3 | Not started | - |
| 2. FZF Selector | 0/3 | Not started | - |
| 3. Browsing & Symlinks | 0/2 | Not started | - |
| 4. Session Integration | 0/3 | Not started | - |
| 5. Polish | 0/2 | Not started | - |

---

*Last updated: 2026-01-30 after roadmap creation*
