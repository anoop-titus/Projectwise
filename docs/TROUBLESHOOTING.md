# Enhanced Claude Project Workflow - Troubleshooting Guide

## 1. Installation & Setup Issues

### Issue: FZF Not Working

**Symptom:**
```
$ claude
-bash: fzf: command not found
```

**Cause:** FZF is not installed on your system.

**Solution:**
```bash
# Install FZF via Nix
$ nix profile install nixpkgs#fzf

# Verify installation
$ which fzf
/path/to/fzf

# Reload shell to pick up changes
$ exec zsh
```

---

### Issue: Scripts Not Found

**Symptom:**
```
$ registry-init.sh
-bash: registry-init.sh: command not found
```

**Cause:** ~/.claude/scripts is not in your PATH.

**Solution:**
```bash
# Option 1: Run with full path
$ ~/.claude/scripts/registry-init.sh

# Option 2: Add to PATH in ~/.zshrc
export PATH="$PATH:$HOME/.claude/scripts"

# Option 3: Reload shell
$ exec zsh
```

---

### Issue: Permission Denied

**Symptom:**
```
$ ~/.claude/scripts/registry-init.sh
-bash: /Users/titus/.claude/scripts/registry-init.sh: Permission denied
```

**Cause:** Scripts are not executable.

**Solution:**
```bash
# Make scripts executable
$ chmod +x ~/.claude/scripts/*.sh

# Verify
$ ls -l ~/.claude/scripts/registry-init.sh
# Should show: -rwxr-xr-x

# Run again
$ ~/.claude/scripts/registry-init.sh
```

---

### Issue: .zshrc Not Loading New Functions

**Symptom:**
```
$ claude-favorite
-bash: claude-favorite: command not found
```

**Cause:** Shell hasn't reloaded .zshrc with new functions.

**Solution:**
```bash
# Option 1: Reload shell
$ exec zsh

# Option 2: Source .zshrc directly
$ source ~/.zshrc

# Option 3: Open new terminal tab

# Verify
$ type claude
# Should show: claude is a shell function
```

---

## 2. Selector Issues

### Issue: Selector Not Opening

**Symptom:**
```
$ claude
(Nothing happens, selector doesn't appear)
```

**Cause:** Various - FZF not installed, registry not found, or script error.

**Solution:**
```bash
# Step 1: Check FZF installation
$ which fzf
# If not found, install: nix profile install nixpkgs#fzf

# Step 2: Check registry exists
$ test -f ~/.claude/projects/.registry.json && echo "Registry exists" || echo "Registry missing"

# Step 3: Run with debug output
$ bash -x ~/.claude/scripts/project-select.sh 2>&1 | head -50

# Step 4: Rebuild registry
$ ~/.claude/scripts/registry-init.sh
```

---

### Issue: No Projects Showing in Selector

**Symptom:**
```
$ claude
(Selector opens but shows no projects)
```

**Cause:** Registry is empty or projects not registered.

**Solution:**
```bash
# Step 1: Check registry
$ jq '.projects | length' ~/.claude/projects/.registry.json
# Should show a number > 0

# Step 2: Rebuild registry
$ ~/.claude/scripts/registry-init.sh

# Step 3: Check project folders exist
$ ls -la ~/.claude/projects/
# Should show project folders like "my-project", etc.

# Step 4: Validate registry
$ ~/.claude/scripts/registry-recover.sh validate
```

---

### Issue: Search (Fuzzy Matching) Not Working

**Symptom:**
```
$ claude
> machine
(No matches appear, even though "machine-learning" project exists)
```

**Cause:** FZF search mode not active, or project name not in registry.

**Solution:**
```bash
# Step 1: Make sure you're typing (search starts automatically)
# Just start typing, don't press any keys first

# Step 2: Check project is in registry
$ grep "machine" ~/.claude/projects/.registry.json
# Should find matches

# Step 3: Rebuild registry
$ ~/.claude/scripts/registry-init.sh

# Step 4: Try again
$ claude
> machine
```

---

### Issue: Keyboard Shortcuts Not Working

**Symptom:**
```
$ claude
(Ctrl-F doesn't switch to favorites mode)
(Ctrl-C doesn't show categories)
```

**Cause:** FZF version incompatibility or terminal not supporting keybindings.

**Solution:**
```bash
# Step 1: Check FZF version
$ fzf --version
# Should be 0.29.0 or later

# Step 2: Update FZF
$ nix profile upgrade fzf

# Step 3: Check terminal supports keybindings
$ echo $TERM
# Should be: xterm-256color or similar (not 'dumb')

# Step 4: Try with explicit terminal
$ TERM=xterm-256color claude

# Step 5: If still not working, check FZF binds
$ fzf --help | grep -i bind
```

---

### Issue: Preview Pane Not Showing

**Symptom:**
```
$ claude
(Selector works but no preview pane with metadata visible)
```

**Cause:** Terminal window too narrow, preview script missing, or Ctrl-S toggled it off.

**Solution:**
```bash
# Step 1: Toggle preview pane
$ claude
# (In selector) Press Ctrl-S
# Should show/hide preview pane

# Step 2: Make terminal wider
# Resize window to >120 columns for best preview

# Step 3: Check preview script exists
$ test -f ~/.claude/scripts/project-preview.sh && echo "Exists" || echo "Missing"

# Step 4: Rebuild preview data
$ ~/.claude/scripts/project-preview.sh

# Step 5: Rebuild registry
$ ~/.claude/scripts/registry-init.sh
```

---

## 3. Registry Issues

### Issue: Registry Corrupted (JSON Parse Error)

**Symptom:**
```
$ claude
[jq: parse error: ... ]
(Selector crashes)
```

**Cause:** Registry file corrupted or invalid JSON.

**Solution:**
```bash
# Step 1: Check registry validity
$ jq empty ~/.claude/projects/.registry.json
# If error appears, registry is corrupted

# Step 2: List available backups
$ ~/.claude/scripts/registry-recover.sh list
# Shows all backup copies

# Step 3: Restore most recent backup
$ ~/.claude/scripts/registry-recover.sh restore-latest
# Restores from latest good copy

# Step 4: Verify restored registry
$ ~/.claude/scripts/registry-recover.sh validate
# Should show: Registry is valid

# Step 5: Test
$ claude
# Should work now
```

---

### Issue: Symlinks Not Updating

**Symptom:**
```
$ ls ~/.claude/projects/active/
(Old symlinks, new projects not there)

$ ls ~/.claude/projects/favorites/
(Not updated after running claude-favorite)
```

**Cause:** Symlink organization script not run or failed silently.

**Solution:**
```bash
# Step 1: Run symlink organizer
$ ~/.claude/scripts/symlink-organize.sh

# Step 2: Verify symlinks updated
$ ls -la ~/.claude/projects/active/
$ ls -la ~/.claude/projects/favorites/

# Step 3: Check for errors
$ ~/.claude/scripts/symlink-organize.sh 2>&1 | tail -20
# Look for error messages

# Step 4: Rebuild from scratch
$ rm -rf ~/.claude/projects/active/
$ rm -rf ~/.claude/projects/favorites/
$ ~/.claude/scripts/symlink-organize.sh

# Step 5: Verify
$ ls ~/.claude/projects/active/
```

---

### Issue: Projects Not Being Tracked (Missing from Registry)

**Symptom:**
```
$ ls ~/.claude/projects/
my-project  another-project

$ claude
(Only shows my-project, not another-project)
```

**Cause:** Unregistered projects or registry not updated.

**Solution:**
```bash
# Step 1: Rebuild registry
$ ~/.claude/scripts/registry-init.sh

# Step 2: Check registry now has both
$ jq '.projects | length' ~/.claude/projects/.registry.json

# Step 3: Verify both appear
$ jq '.projects[] | .display_name' ~/.claude/projects/.registry.json

# Step 4: Test selector
$ claude
# Should now show all projects
```

---

### Issue: Metadata Missing or Wrong

**Symptom:**
```
$ claude-info
(Shows empty description, missing tags, wrong category)
```

**Cause:** Registry not fully initialized or data lost.

**Solution:**
```bash
# Step 1: Rebuild registry
$ ~/.claude/scripts/registry-init.sh

# Step 2: Check registry has metadata
$ jq '.projects[0]' ~/.claude/projects/.registry.json

# Step 3: Edit manually if needed
$ nano ~/.claude/projects/.registry.json
# Find the project entry and fill in fields

# Step 4: Validate
$ ~/.claude/scripts/registry-recover.sh validate

# Step 5: Test
$ claude-info
```

---

## 4. Performance Issues

### Issue: Selector Very Slow

**Symptom:**
```
$ claude
(Takes 5+ seconds to open)
(Takes 2+ seconds per keystroke)
```

**Cause:** Too many projects, system load, or slow FZF version.

**Solution:**
```bash
# Step 1: Check system load
$ top -l1 | head -5
# High CPU or memory usage? Close other apps.

# Step 2: Count projects
$ jq '.projects | length' ~/.claude/projects/.registry.json
# >500 projects? May need optimization.

# Step 3: Update FZF to latest
$ nix profile upgrade fzf

# Step 4: Rebuild registry (cleaner data)
$ ~/.claude/scripts/registry-init.sh

# Step 5: Use categories to filter
$ claude
> Ctrl-C
# Select category to search smaller subset
```

---

### Issue: Registry Operations Taking Too Long

**Symptom:**
```
$ claude-favorite
(Takes 5+ seconds)
(No output for a while)
```

**Cause:** Large registry, slow disk, or concurrent operations.

**Solution:**
```bash
# Step 1: Check disk usage
$ du -sh ~/.claude/projects/
# If >1GB, may need cleanup

# Step 2: Check concurrent operations
$ ps aux | grep registry
# Multiple processes? Wait for one to finish.

# Step 3: Run operation with verbose output
$ bash -x ~/.claude/scripts/registry-update.sh ... 2>&1 | tee /tmp/debug.log
# Look for slow steps in output

# Step 4: Rebuild registry
$ ~/.claude/scripts/registry-init.sh

# Step 5: Cleanup old backups
$ ~/.claude/scripts/registry-recover.sh cleanup 30
```

---

### Issue: Too Many Projects Slowing Things Down

**Symptom:**
```
$ claude
(Loads 200+ projects, selector becomes slow)

$ claude-status
(Takes a long time)
```

**Cause:** Registry getting too large.

**Solution:**
```bash
# Step 1: Archive inactive projects
$ nano ~/.claude/projects/.registry.json
# Find inactive projects, change "status": "archived"

# Step 2: Clean old backups
$ ~/.claude/scripts/registry-recover.sh cleanup 30
$ ~/.claude/scripts/registry-recover.sh cleanup-execute 30

# Step 3: Use categories to filter
$ claude
> Ctrl-C
# Filter by category to search subset

# Step 4: Rebuild registry optimized
$ ~/.claude/scripts/registry-init.sh --force
```

---

## 5. Recovery Procedures

### Complete Registry Corruption - Nuclear Option

**When:** Registry is severely corrupted and won't load at all.

**Steps:**
```bash
# Step 1: Stop everything
$ ~/.claude/scripts/stop-prep-container.sh 2>/dev/null

# Step 2: List all available backups
$ ~/.claude/scripts/registry-recover.sh list

# Step 3: Try most recent backup
$ ~/.claude/scripts/registry-recover.sh restore-latest

# Step 4: If that fails, try by timestamp
$ ~/.claude/scripts/registry-recover.sh restore 1704067200

# Step 5: If all backups fail, rebuild from scratch
$ rm ~/.claude/projects/.registry.json
$ ~/.claude/scripts/registry-init.sh

# Step 6: Test
$ claude
```

---

### Restore from Specific Backup

**When:** You want to go back to a known good state.

**Steps:**
```bash
# Step 1: List backups
$ ~/.claude/scripts/registry-recover.sh list
# Output:
# 2026-01-30 10:00:00  .registry.json.backup.1704067200
# 2026-01-29 14:30:00  .registry.json.backup.1704066600

# Step 2: Pick a timestamp
# Let's restore from Jan 29

# Step 3: Restore
$ ~/.claude/scripts/registry-recover.sh restore 1704066600

# Step 4: Verify
$ ~/.claude/scripts/registry-recover.sh validate

# Step 5: Test
$ claude
```

---

### Rebuild Registry from Scratch

**When:** You want a completely fresh start.

**Steps:**
```bash
# Step 1: Backup current registry
$ cp ~/.claude/projects/.registry.json ~/.claude/projects/.registry.json.before-rebuild

# Step 2: Remove current registry
$ rm ~/.claude/projects/.registry.json

# Step 3: Rebuild
$ ~/.claude/scripts/registry-init.sh

# Step 4: Check new registry
$ jq '.projects | length' ~/.claude/projects/.registry.json

# Step 5: Restore custom metadata if needed
$ nano ~/.claude/projects/.registry.json
# Add back any custom descriptions, tags, categories

# Step 6: Test
$ claude
```

---

### Recover from Accidental Deletion

**When:** You deleted a project folder and want to recover it.

**Steps:**
```bash
# Step 1: Check if Trash has it
$ ls -la ~/.Trash/ | grep my-project

# Step 2: If found, restore
$ mv ~/.Trash/my-project ~/.claude/projects/

# Step 3: Rebuild registry to register it again
$ ~/.claude/scripts/registry-init.sh

# Step 4: Test
$ claude
```

---

## 6. When All Else Fails

### Rollback to Old System

**When:** The new system is causing problems and you need the old numbered menu back.

**Steps:**
```bash
# Step 1: Run rollback script
$ ~/.claude/scripts/rollback.sh

# Step 2: When prompted, answer yes to remove components
$ [Y/n]: y

# Step 3: Reload shell
$ exec zsh

# Step 4: Verify old claude() function works
$ type claude
# Should show: claude is a shell function from ~/.zshrc (old one)

# Step 5: Test
$ claude
# Should show old numbered menu
```

---

### Re-enable New System After Rollback

**When:** You rolled back but want to use the new system again.

**Steps:**
```bash
# Step 1: Run migration again
$ ~/.claude/scripts/migrate.sh --force

# Step 2: Test
$ claude
# Should show new FZF selector

# Step 3: Enable features
$ claude-status
$ claude-favorite
```

---

### Manual Recovery Steps

**When:** Scripts aren't working and you need to fix things manually.

**Steps:**
```bash
# Step 1: Verify FZF installed
$ which fzf || nix profile install nixpkgs#fzf

# Step 2: Verify jq installed
$ which jq || nix profile install nixpkgs#jq

# Step 3: Verify scripts exist
$ ls ~/.claude/scripts/project-select.sh
$ ls ~/.claude/scripts/registry-init.sh

# Step 4: Verify registry exists
$ test -f ~/.claude/projects/.registry.json || \
  ~/.claude/scripts/registry-init.sh

# Step 5: Test selector manually
$ FZF_DEFAULT_COMMAND='jq -r ".projects[] | .display_name" ~/.claude/projects/.registry.json' \
  fzf

# Step 6: If that works, shell functions might be the issue
$ exec zsh
```

---

### Getting Detailed Error Messages

**When:** You need to see what's actually failing.

**Steps:**
```bash
# Run with debug mode enabled
$ bash -x ~/.claude/scripts/registry-init.sh 2>&1 | tee /tmp/debug.log

# Look for the error in output
$ grep -i error /tmp/debug.log

# Check specific line in output
$ head -100 /tmp/debug.log | tail -50

# Save for later analysis
$ cat /tmp/debug.log
```

---

## 7. When to Ask for Help

**Contact support if:**
- Multiple recovery attempts have failed
- You can't determine the cause from error messages
- The system is completely non-functional
- You've lost important project data

**Include in bug report:**
```bash
# System info
$ uname -a

# Shell version
$ zsh --version

# FZF version
$ fzf --version

# Registry validation
$ ~/.claude/scripts/registry-recover.sh validate

# Debug output
$ bash -x ~/.claude/scripts/registry-init.sh 2>&1
```

---

## Quick Diagnosis Script

**Save this as ~/diagnose.sh to run full diagnostics:**

```bash
#!/bin/bash

echo "=== Claude Project Workflow Diagnostics ==="
echo ""

echo "1. System Info:"
uname -a
echo ""

echo "2. Shell:"
echo $SHELL && zsh --version
echo ""

echo "3. Required Tools:"
which fzf && fzf --version || echo "FZF: MISSING"
which jq && jq --version || echo "JQ: MISSING"
echo ""

echo "4. Scripts:"
ls -1 ~/.claude/scripts/*.sh | wc -l && echo "scripts found"
echo ""

echo "5. Registry:"
test -f ~/.claude/projects/.registry.json && echo "Registry exists" || echo "Registry MISSING"
jq empty ~/.claude/projects/.registry.json && echo "Registry valid" || echo "Registry INVALID"
echo ""

echo "6. Projects:"
jq '.projects | length' ~/.claude/projects/.registry.json
echo ""

echo "7. Functions Loaded:"
type claude 2>/dev/null && echo "claude: OK" || echo "claude: MISSING"
type claude-favorite 2>/dev/null && echo "claude-favorite: OK" || echo "claude-favorite: MISSING"
echo ""

echo "=== Diagnostics Complete ==="
```

**Run it:**
```bash
$ chmod +x ~/diagnose.sh
$ ~/diagnose.sh
```

---

## Summary

Most issues fall into these categories:

1. **Installation** - Missing FZF, jq, or scripts
2. **Registry** - Corrupted, missing, or out of date
3. **Symlinks** - Not being updated properly
4. **Performance** - Too many projects or slow disk
5. **Shell** - Functions not loaded or .zshrc not sourced

**Always start with:**
```bash
$ exec zsh                              # Reload shell
$ ~/.claude/scripts/registry-init.sh    # Rebuild registry
$ ~/.claude/scripts/symlink-organize.sh # Update symlinks
$ claude-status                         # Test system
```

In 95% of cases, this fixes the issue!
