# Enhanced Claude Project Workflow - Verification Checklist

Complete this checklist to verify the system is working correctly. All items should pass before declaring the system ready for use.

---

## Part 1: System Components Check

### 1.1 Registry File

- [ ] Registry file exists
  ```bash
  test -f ~/.claude/projects/.registry.json && echo "PASS: Registry exists"
  ```

- [ ] Registry is valid JSON
  ```bash
  jq empty ~/.claude/projects/.registry.json && echo "PASS: Valid JSON"
  ```

- [ ] Registry has expected structure
  ```bash
  jq 'has("metadata") and has("projects")' ~/.claude/projects/.registry.json | grep true
  ```

- [ ] Registry contains projects
  ```bash
  jq '.projects | length' ~/.claude/projects/.registry.json | grep -v '^0$'
  ```

### 1.2 Required Scripts

- [ ] registry-init.sh exists
  ```bash
  test -x ~/.claude/scripts/registry-init.sh && echo "PASS"
  ```

- [ ] project-select.sh exists
  ```bash
  test -x ~/.claude/scripts/project-select.sh && echo "PASS"
  ```

- [ ] registry-update.sh exists
  ```bash
  test -x ~/.claude/scripts/registry-update.sh && echo "PASS"
  ```

- [ ] registry-recover.sh exists
  ```bash
  test -x ~/.claude/scripts/registry-recover.sh && echo "PASS"
  ```

- [ ] symlink-organize.sh exists
  ```bash
  test -x ~/.claude/scripts/symlink-organize.sh && echo "PASS"
  ```

- [ ] project-preview.sh exists
  ```bash
  test -x ~/.claude/scripts/project-preview.sh && echo "PASS"
  ```

- [ ] folder-browse.sh exists
  ```bash
  test -x ~/.claude/scripts/folder-browse.sh && echo "PASS"
  ```

### 1.3 Shell Integration

- [ ] .zshrc contains new claude() function
  ```bash
  grep -q "function claude" ~/.zshrc && echo "PASS: claude function found"
  ```

- [ ] Helper commands available (claude-favorite)
  ```bash
  grep -q "claude-favorite" ~/.zshrc && echo "PASS: claude-favorite found"
  ```

- [ ] Helper commands available (claude-info)
  ```bash
  grep -q "claude-info" ~/.zshrc && echo "PASS: claude-info found"
  ```

- [ ] Helper commands available (claude-status)
  ```bash
  grep -q "claude-status" ~/.zshrc && echo "PASS: claude-status found"
  ```

- [ ] Helper commands available (claude-list)
  ```bash
  grep -q "claude-list" ~/.zshrc && echo "PASS: claude-list found"
  ```

- [ ] .zshrc has no syntax errors
  ```bash
  zsh -n ~/.zshrc && echo "PASS: No syntax errors"
  ```

### 1.4 Directories

- [ ] ~/.claude/projects/ exists
  ```bash
  test -d ~/.claude/projects && echo "PASS"
  ```

- [ ] ~/.claude/projects/active/ exists
  ```bash
  test -d ~/.claude/projects/active && echo "PASS"
  ```

- [ ] ~/.claude/projects/favorites/ exists
  ```bash
  test -d ~/.claude/projects/favorites && echo "PASS"
  ```

- [ ] ~/.claude/scripts/ exists
  ```bash
  test -d ~/.claude/scripts && echo "PASS"
  ```

---

## Part 2: Functionality Tests

### 2.1 Main Selector

**Start fresh shell (new terminal or exec zsh)**

- [ ] `claude` command exists
  ```bash
  type claude
  # Should show: claude is a shell function
  ```

- [ ] `claude` selector opens
  ```bash
  timeout 5 bash -c 'echo "Enter" | claude' >/dev/null 2>&1 && echo "PASS"
  ```

- [ ] Selector shows projects
  ```bash
  # Run: claude
  # Check that projects are listed (don't type, just look)
  # Expected: Shows 5+ project names
  ```

### 2.2 Fuzzy Search

- [ ] Type to search works
  ```bash
  # Run: claude
  # Type: "research" (or first letters of a project name)
  # Expected: Matches projects containing "research"
  ```

- [ ] Search results update in real-time
  ```bash
  # Run: claude
  # Type: "test"
  # Expected: List updates instantly as you type
  ```

- [ ] Clearing search restores full list
  ```bash
  # Run: claude
  # Type: "zzzzz" (matches nothing)
  # Ctrl-U to clear search
  # Expected: Full project list restored
  ```

### 2.3 Keyboard Navigation

- [ ] Arrow up/down navigates list
  ```bash
  # Run: claude
  # Press: ↑ or ↓
  # Expected: Cursor moves, selection changes
  ```

- [ ] Enter selects project
  ```bash
  # Run: claude
  # Press: Enter (on any project)
  # Expected: cd into project, shell prompt appears
  ```

- [ ] Escape cancels selector
  ```bash
  # Run: claude
  # Press: Escape
  # Expected: Returns to shell prompt, didn't cd anywhere
  ```

### 2.4 Selection Modes

- [ ] Ctrl-F switches to favorites mode
  ```bash
  # Run: claude
  # Press: Ctrl-F
  # Expected: List shows only favorited projects (if any)
  ```

- [ ] Ctrl-C opens category filter
  ```bash
  # Run: claude
  # Press: Ctrl-C
  # Expected: Category menu appears, can select category
  ```

- [ ] Ctrl-B opens folder browse mode
  ```bash
  # Run: claude
  # Press: Ctrl-B
  # Expected: Shows all folders recursively, can navigate
  ```

- [ ] Ctrl-S toggles preview pane
  ```bash
  # Run: claude
  # Press: Ctrl-S
  # Expected: Preview pane appears/disappears
  ```

### 2.5 Preview Pane

- [ ] Preview shows project metadata
  ```bash
  # Run: claude
  # Look at right side of selector
  # Expected: Shows Name, Category, Description, Tags, etc.
  ```

- [ ] Preview updates when selection changes
  ```bash
  # Run: claude
  # Press: ↓ (move to different project)
  # Expected: Preview updates to show new project's info
  ```

- [ ] Preview shows last_accessed and session_count
  ```bash
  # Run: claude
  # Look at preview for any project
  # Expected: Shows dates and access count
  ```

### 2.6 Project Selection Behavior

- [ ] Selecting project cd's into it
  ```bash
  $ pwd
  /current/directory
  $ claude  # Select a project and press Enter
  $ pwd
  /Users/titus/.claude/projects/selected-project
  # Expected: Working directory changed
  ```

- [ ] Registry updates last_accessed after selection
  ```bash
  # Note timestamp before:
  $ jq '.projects[0].last_accessed' ~/.claude/projects/.registry.json

  # Select the project:
  $ claude  # Select first project, press Enter

  # Note timestamp after:
  $ jq '.projects[0].last_accessed' ~/.claude/projects/.registry.json
  # Expected: Timestamp is newer
  ```

- [ ] Registry increments session_count after selection
  ```bash
  # Note count before:
  $ jq '.projects[0].session_count' ~/.claude/projects/.registry.json

  # Select the project:
  $ claude  # Select first project, press Enter

  # Note count after:
  $ jq '.projects[0].session_count' ~/.claude/projects/.registry.json
  # Expected: Count increased by 1
  ```

---

## Part 3: Helper Commands

### 3.1 claude-favorite

**Prerequisites: Reload shell first**
```bash
$ exec zsh
```

- [ ] Command exists
  ```bash
  type claude-favorite
  # Should show: claude-favorite is a shell function
  ```

- [ ] Can toggle favorite (on)
  ```bash
  $ cd ~/.claude/projects/my-project
  $ claude-favorite
  # Expected: Shows "✓ Added to favorites" or similar
  ```

- [ ] Favorite appears in registry
  ```bash
  $ jq '.projects[] | select(.id=="my-project") | .favorite' ~/.claude/projects/.registry.json
  # Expected: true
  ```

- [ ] Can toggle favorite (off)
  ```bash
  $ cd ~/.claude/projects/my-project
  $ claude-favorite
  # Expected: Shows "✓ Removed from favorites" or similar
  ```

- [ ] Symlink added to favorites directory
  ```bash
  $ ls -la ~/.claude/projects/favorites/
  # Expected: Symlink to favorited project appears
  ```

- [ ] Symlink removed from favorites directory
  ```bash
  $ ls -la ~/.claude/projects/favorites/
  # Expected: Symlink to unfavorited project is gone
  ```

### 3.2 claude-info

- [ ] Command exists
  ```bash
  type claude-info
  # Should show: claude-info is a shell function
  ```

- [ ] Shows project metadata
  ```bash
  $ cd ~/.claude/projects/my-project
  $ claude-info
  # Expected output includes:
  # - Project name
  # - Description
  # - Tags
  # - Category
  # - Status
  # - Dates
  # - Session count
  ```

- [ ] Metadata is accurate
  ```bash
  $ claude-info
  # Compare with:
  $ jq '.projects[] | select(.id=="my-project")' ~/.claude/projects/.registry.json
  # Expected: Values match
  ```

### 3.3 claude-list

- [ ] Quick mode works
  ```bash
  $ claude-list quick
  # Expected: Lists recent projects
  ```

- [ ] Favorite mode works
  ```bash
  $ claude-list favorite
  # Expected: Lists only favorited projects
  ```

- [ ] Category mode works
  ```bash
  $ claude-list category
  # Expected: Shows projects, filtered by category
  ```

- [ ] Browse mode works
  ```bash
  $ claude-list browse
  # Expected: Shows all folders recursively
  ```

- [ ] Default (no args) works
  ```bash
  $ claude-list
  # Expected: Shows something (default to quick mode)
  ```

### 3.4 claude-status

- [ ] Command exists
  ```bash
  type claude-status
  # Should show: claude-status is a shell function
  ```

- [ ] Shows total project count
  ```bash
  $ claude-status
  # Expected: Shows "Total Projects: N"
  ```

- [ ] Shows projects by status
  ```bash
  $ claude-status
  # Expected: Shows "Active: X, Paused: Y, Archived: Z"
  ```

- [ ] Shows statistics
  ```bash
  $ claude-status
  # Expected: Shows most accessed, least accessed, recent activity
  ```

- [ ] Shows favorite count
  ```bash
  $ claude-status
  # Expected: Shows "Favorites: N"
  ```

- [ ] Shows categories
  ```bash
  $ claude-status
  # Expected: Shows "Categories: N"
  ```

---

## Part 4: Edge Cases

### 4.1 Cancel/Escape Handling

- [ ] Escape doesn't crash selector
  ```bash
  $ claude
  # Press: Escape
  # Expected: Returns to shell prompt, no error messages
  ```

- [ ] Escape doesn't cd anywhere
  ```bash
  $ pwd
  $ cd /tmp
  $ pwd
  $ claude  # Press Escape
  $ pwd
  # Expected: Still in /tmp
  ```

- [ ] Multiple Escapes work
  ```bash
  $ claude
  # Press: Escape, Escape, Escape
  # Expected: Returns to shell, no errors
  ```

### 4.2 Invalid Input Handling

- [ ] Search for non-existent project
  ```bash
  $ claude
  > zzzzzzzzz (doesn't match anything)
  # Expected: No matches shown, selector still works
  ```

- [ ] Empty registry doesn't crash
  ```bash
  # (Only do this if you want to test)
  # Backup registry: cp ~/.claude/projects/.registry.json ~/.claude/projects/.registry.json.backup
  # Create empty: echo '{"metadata":{},"projects":[]}' > ~/.claude/projects/.registry.json
  # Run: claude
  # Expected: Opens but shows no projects
  # Restore: cp ~/.claude/projects/.registry.json.backup ~/.claude/projects/.registry.json
  ```

### 4.3 Registry Corruption Detection

- [ ] Corrupted registry detected
  ```bash
  # Create corrupted registry:
  $ echo 'invalid json {' > ~/.claude/projects/.registry.json

  # Run validator:
  $ ~/.claude/scripts/registry-recover.sh validate
  # Expected: Shows error about invalid JSON

  # Restore:
  $ ~/.claude/scripts/registry-recover.sh restore-latest
  ```

### 4.4 Symlink Maintenance

- [ ] Broken symlinks handled gracefully
  ```bash
  # Create broken symlink:
  $ ln -s /nonexistent ~/.claude/projects/active/broken-link

  # Run organizer:
  $ ~/.claude/scripts/symlink-organize.sh
  # Expected: Removes broken link or handles it gracefully
  ```

- [ ] Symlinks stay in sync with registry
  ```bash
  # After adding/removing favorites:
  $ ~/.claude/scripts/symlink-organize.sh
  # Check:
  $ ls ~/.claude/projects/active/ | wc -l
  # Should match number of active projects in registry
  ```

---

## Part 5: Performance

### 5.1 Speed Tests

- [ ] Selector opens in <2 seconds
  ```bash
  $ time claude
  # Press: Escape
  # Expected: real time < 2s (mostly FZF startup)
  ```

- [ ] Search responsive (<500ms per keystroke)
  ```bash
  $ claude
  # Type quickly: "test"
  # Expected: Results appear instantly as you type
  ```

- [ ] Registry operations <100ms
  ```bash
  $ time ~/.claude/scripts/registry-update.sh update_last_accessed my-project
  # Expected: real time < 0.1s
  ```

### 5.2 Load Tests

- [ ] 50+ projects searchable
  ```bash
  $ jq '.projects | length' ~/.claude/projects/.registry.json
  # If >= 50, this is passing
  # Selector should still be responsive
  ```

- [ ] 100+ projects searchable
  ```bash
  # If you have 100+ projects, verify selector still responsive
  # Type a search query
  # Expected: Results appear in <1 second
  ```

---

## Part 6: Data Integrity

### 6.1 Backup Mechanism

- [ ] Backups created automatically
  ```bash
  $ ls -la ~/.claude/projects/.registry.json.backup.* 2>/dev/null | head -5
  # Expected: Shows several backup files with timestamps
  ```

- [ ] Backups are valid JSON
  ```bash
  $ for backup in ~/.claude/projects/.registry.json.backup.*; do
      jq empty "$backup" && echo "PASS: $backup"
    done
  # Expected: All backups validate
  ```

### 6.2 Recovery Mechanism

- [ ] Recovery script lists backups
  ```bash
  $ ~/.claude/scripts/registry-recover.sh list
  # Expected: Shows list of available backups
  ```

- [ ] Can restore from backup
  ```bash
  $ ~/.claude/scripts/registry-recover.sh restore-latest
  # Expected: Registry restored, validation passes
  ```

- [ ] No data loss during recovery
  ```bash
  # Before:
  $ jq '.projects | length' ~/.claude/projects/.registry.json
  # Should equal number after recovery
  ```

---

## Part 7: Cross-Functional Integration

### 7.1 Workflow: Create and Use Project

- [ ] Can create new project
  ```bash
  $ claude --new
  # Follow prompts
  # Expected: New project folder created in ~/.claude/projects/
  ```

- [ ] New project appears in selector
  ```bash
  $ claude
  # Should see new project in list
  ```

- [ ] Can select new project
  ```bash
  $ claude
  > new-project
  # Press: Enter
  # Expected: cd into new project
  ```

### 7.2 Workflow: Favorite and Access

- [ ] Can favorite project
  ```bash
  $ cd ~/.claude/projects/some-project
  $ claude-favorite
  # Expected: Added to favorites
  ```

- [ ] Can access via Ctrl-F
  ```bash
  $ claude
  # Press: Ctrl-F
  # Expected: Shows only favorite projects including the one just added
  ```

### 7.3 Workflow: Organize by Category

- [ ] Can filter by category
  ```bash
  $ claude
  # Press: Ctrl-C
  # Press: Enter (select category)
  # Expected: Shows only projects in that category
  ```

- [ ] Categories are accurate
  ```bash
  # Check a few projects match their displayed category:
  $ jq '.projects[] | {name: .display_name, category: .category}' ~/.claude/projects/.registry.json
  # Compare with what selector shows for that category
  ```

### 7.4 Workflow: Browse and Navigate

- [ ] Can browse all folders
  ```bash
  $ claude
  # Press: Ctrl-B
  # Expected: Shows folder tree, can navigate with arrow keys
  ```

- [ ] Can select from browse mode
  ```bash
  $ claude
  # Press: Ctrl-B
  # Navigate to a project folder
  # Press: Enter
  # Expected: cd into selected project
  ```

---

## Part 8: Migration Verification

### 8.1 One-Time Migration

- [ ] Migration happened once
  ```bash
  $ test -f ~/.claude/migration-completed.txt && echo "PASS"
  ```

- [ ] Backup of old .zshrc exists
  ```bash
  $ ls ~/.claude/backups/zshrc-before-migration-* 2>/dev/null | head -1
  # Expected: At least one backup
  ```

- [ ] Migration can be re-run
  ```bash
  $ ~/.claude/scripts/migrate.sh --force
  # Expected: Completes without errors
  ```

### 8.2 Rollback Capability

- [ ] Rollback script exists
  ```bash
  $ test -x ~/.claude/scripts/rollback.sh && echo "PASS"
  ```

- [ ] Can rollback if needed
  ```bash
  # (Optional, only test if you want to)
  $ ~/.claude/scripts/rollback.sh
  # Should restore old .zshrc and old claude() function
  ```

---

## Final Validation Checklist

Run this comprehensive final check:

```bash
#!/bin/bash
echo "=== Final Validation ==="

# 1. System components
echo "1. System components:"
test -f ~/.claude/projects/.registry.json && echo "  ✓ Registry exists" || echo "  ✗ Registry missing"
test -x ~/.claude/scripts/registry-init.sh && echo "  ✓ Scripts exist" || echo "  ✗ Scripts missing"
grep -q "function claude" ~/.zshrc && echo "  ✓ Shell functions updated" || echo "  ✗ Shell functions missing"
zsh -n ~/.zshrc && echo "  ✓ .zshrc valid" || echo "  ✗ .zshrc has errors"

# 2. Functionality
echo ""
echo "2. Functionality:"
exec zsh -c 'type claude >/dev/null 2>&1' && echo "  ✓ claude command works" || echo "  ✗ claude command missing"
exec zsh -c 'type claude-favorite >/dev/null 2>&1' && echo "  ✓ claude-favorite works" || echo "  ✗ claude-favorite missing"
exec zsh -c 'type claude-info >/dev/null 2>&1' && echo "  ✓ claude-info works" || echo "  ✗ claude-info missing"
exec zsh -c 'type claude-status >/dev/null 2>&1' && echo "  ✓ claude-status works" || echo "  ✗ claude-status missing"

# 3. Data integrity
echo ""
echo "3. Data integrity:"
jq empty ~/.claude/projects/.registry.json 2>/dev/null && echo "  ✓ Registry valid" || echo "  ✗ Registry invalid"
test -f ~/.claude/projects/.registry.json.backup.* && echo "  ✓ Backups exist" || echo "  ✗ Backups missing"

# 4. Performance
echo ""
echo "4. Performance:"
projects=$(jq '.projects | length' ~/.claude/projects/.registry.json)
echo "  ✓ Registry contains $projects projects"

echo ""
echo "=== Validation Complete ==="
```

Save as `~/validate.sh` and run:
```bash
$ chmod +x ~/validate.sh
$ ~/validate.sh
```

---

## Remediation Guide

If any check fails, consult TROUBLESHOOTING.md for specific solutions.

**Quick fixes for common issues:**

1. **Registry missing/invalid:**
   ```bash
   $ ~/.claude/scripts/registry-recover.sh restore-latest
   ```

2. **Shell functions not loading:**
   ```bash
   $ exec zsh
   ```

3. **Scripts not found:**
   ```bash
   $ chmod +x ~/.claude/scripts/*.sh
   ```

4. **Selector not working:**
   ```bash
   $ ~/.claude/scripts/registry-init.sh --force
   ```

---

## Approval Criteria

All items checked = System Ready

- [ ] All system components present
- [ ] All functionality tests pass
- [ ] All helper commands work
- [ ] All edge cases handled
- [ ] Performance acceptable (<2 seconds)
- [ ] Data integrity verified
- [ ] Cross-functional workflows work
- [ ] Migration/rollback verified

**Ready for production:** When all 8 criteria are met.
