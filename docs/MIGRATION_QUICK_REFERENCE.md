# Enhanced Claude Project Workflow - Migration Quick Reference

## Quick Start

### For Existing Users (First Time)
```bash
~/.claude/scripts/migrate.sh
```

The script will:
1. Backup your current .zshrc
2. Initialize the project registry
3. Create symlink directories
4. Update your .zshrc with the new claude() function
5. Verify everything works
6. Show a summary of changes

Takes approximately 30-60 seconds.

### If Something Goes Wrong
```bash
~/.claude/scripts/rollback.sh
```

This will:
1. Restore your .zshrc from backup
2. Ask if you want to remove new components
3. Verify the rollback succeeded
4. Provide instructions for re-migration

## What Changes?

### Before Migration
- Simple numbered project menu in .zshrc
- Manual project selection
- Limited metadata about projects

### After Migration
- FZF-based fuzzy search interface
- Multiple selection modes (quick, favorites, categories, browse)
- Rich project metadata (description, tags, timestamps)
- Helper commands for managing projects
- Automatic project tracking

## New Commands Available

After migration, you'll have:

```bash
# Main project selector (replaces numbered menu)
claude              # FZF-based project selector with keyboard shortcuts
                    # Ctrl-F: Favorites | Ctrl-C: Categories | Ctrl-B: Browse

# Helper commands
claude-favorite     # Toggle current project as favorite
claude-info         # Show project metadata
claude-list [mode]  # List projects (quick/favorite/category/browse)
claude-status       # Show system statistics and summary
```

## What Gets Backed Up?

### Automatic Backups (Created by migrate.sh)
- `.zshrc` → `~/.claude/backups/zshrc-before-migration-{timestamp}.sh`
- Registry → `~/.claude/projects/.registry.json.pre-migration`

### Preserved During Rollback
- All backup files are kept for recovery
- Old projects are not affected
- Your work is never deleted

## Directory Structure

```
~/.claude/
├── scripts/
│   ├── migrate.sh          ← Run this to migrate
│   ├── rollback.sh         ← Run this to rollback
│   ├── registry-init.sh    ← Auto-scan and index projects
│   ├── project-select.sh   ← FZF selection interface
│   ├── folder-browse.sh    ← Browse all folders
│   ├── symlink-organize.sh ← Organize symlinks
│   └── registry-update.sh  ← Update project metadata
├── backups/
│   └── zshrc-before-migration-*.sh  ← Your backups
├── projects/
│   ├── .registry.json              ← Project metadata
│   ├── .registry.json.pre-migration ← Pre-migration backup
│   ├── active/                      ← Symlinks to active projects
│   ├── favorites/                   ← Symlinks to favorites
│   └── [project folders]            ← Your actual projects
└── migration-completed.txt          ← Migration record
```

## Troubleshooting

### Q: Is my data safe?
**A:** Yes! All backups are automatic, and rollback is always available. No projects are ever deleted.

### Q: Can I run migrate.sh multiple times?
**A:** Yes! The script is idempotent (safe to run multiple times). On subsequent runs, it will ask if you want to re-run. Use `--force` flag to skip the prompt: `migrate.sh --force`

### Q: What if migrate.sh fails?
**A:** The script has automatic error handling. If any step fails, it will ask if you want to rollback. Your .zshrc is immediately restored to the pre-migration state.

### Q: How do I fully remove the new system?
**A:** Run rollback.sh and choose to remove components. To fully clean up:
```bash
~/.claude/scripts/rollback.sh    # Restore .zshrc
rm -rf ~/.claude/projects/.registry.json ~/.claude/projects/active ~/.claude/projects/favorites
```

### Q: Can I re-migrate after rollback?
**A:** Yes! Just run `migrate.sh` again. The script will detect that you've rolled back and allow you to re-migrate.

## Logs

Migration and rollback create detailed logs:
- `~/.claude/migration-{timestamp}.log` - Detailed migration log
- `~/.claude/rollback-{timestamp}.log` - Detailed rollback log

Check these if you need to debug any issues.

## File Locations

| File | Location | Purpose |
|------|----------|---------|
| migrate.sh | `~/.claude/scripts/migrate.sh` | Run migration |
| rollback.sh | `~/.claude/scripts/rollback.sh` | Emergency rollback |
| Registry | `~/.claude/projects/.registry.json` | Project metadata |
| .zshrc backups | `~/.claude/backups/zshrc-before-migration-*.sh` | Recovery files |
| Migration log | `~/.claude/migration-{timestamp}.log` | Detailed log |
| Completion record | `~/.claude/migration-completed.txt` | When migration was done |

## Next Steps

1. **Run migration**: `~/.claude/scripts/migrate.sh`
2. **Open new shell** to load the updated .zshrc
3. **Try the new interface**: `claude` and use keyboard shortcuts
4. **Explore commands**: `claude-status`, `claude-favorite`, `claude-list`

## Support

For detailed documentation, see:
- `~/.claude/projects/PROJECT_WORKFLOW_GUIDE.md` - Comprehensive guide
- `~/.claude/projects/claude_1769760221/.planning/phases/05-polish/05-01-SUMMARY.md` - Technical summary

---

**Status**: ✅ Production Ready
**Version**: 1.0
**Last Updated**: 2026-01-30
