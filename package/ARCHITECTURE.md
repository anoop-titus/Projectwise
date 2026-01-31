# Architecture Overview

## Directory Structure

```
claude-project-manager/
├── bin/                          # CLI executables
│   ├── claude-pm                # Main CLI entry point
│   └── cpm                       # Short alias
├── lib/                          # Core library scripts
│   ├── core.sh                  # Utility functions and error handling
│   ├── registry.sh              # Registry management operations
│   ├── selector.sh              # FZF-based project selection
│   ├── list.sh                  # Project listing with formatting
│   ├── preview.sh               # Project preview functionality
│   ├── info.sh                  # Project information display
│   ├── create.sh                # Project creation
│   ├── delete.sh                # Project deletion
│   └── symlink.sh               # Symlink management
├── templates/                    # Configuration templates
│   ├── registry-template.json   # Empty registry structure
│   ├── project-template.json    # New project metadata
│   ├── zshrc-snippet.sh         # Shell integration for zsh
│   └── bashrc-snippet.sh        # Shell integration for bash
├── scripts/                      # Installation and utility scripts
│   ├── install.sh               # Installation script
│   ├── uninstall.sh             # Uninstallation script
│   └── migrate-from-zshrc.sh    # Migration utility
├── tests/                        # Test suite
│   ├── test_helper.bash         # Test utilities
│   ├── registry-init.bats       # Registry tests
│   ├── project-select.bats      # Selection tests
│   └── integration.bats         # Integration tests
├── docs/                         # Documentation
│   ├── INSTALLATION.md          # Installation guide
│   ├── USAGE.md                 # Usage guide
│   ├── CONTRIBUTING.md          # Contributing guide
│   └── ARCHITECTURE.md          # This file
├── Formula/                      # Homebrew formula
│   └── claude-project-manager.rb
├── nix/                          # Nix package definition
│   └── default.nix
├── .github/workflows/            # CI/CD workflows
│   ├── test.yml                 # Test workflow
│   └── build.yml                # Release build workflow
├── README.md                     # Quick start guide
├── CHANGELOG.md                  # Version history
├── VERSION                       # Version number
├── package.json                  # Package metadata
├── LICENSE                       # MIT License
├── .gitignore                    # Git ignore rules
└── ARCHITECTURE.md              # This file
```

## Component Design

### Entry Points

**bin/claude-pm** - Main CLI wrapper
- Routes commands to appropriate library
- Handles help and version
- Manages error reporting

**bin/cpm** - Short alias
- Convenience wrapper
- Delegates to claude-pm

### Core Libraries

**lib/core.sh** - Utilities and error handling
- Color output functions
- Error handling and logging
- File operations
- Registry path management
- Cleanup handlers

**lib/registry.sh** - Registry management
- Initialize empty registry
- Add/remove projects
- List projects
- Query project metadata
- Update metadata timestamps

**lib/selector.sh** - Interactive selection
- FZF integration
- Project selection UI
- Navigation to selected project

**lib/list.sh** - Project listing
- Formatted table output
- Project counting
- Status information

**lib/preview.sh** - Project preview
- Display project details
- Git information
- File/directory statistics

**lib/info.sh** - Project information
- Detailed project data
- Metadata display
- Timestamps

**lib/create.sh** - Project creation
- Directory initialization
- Git repository setup
- Structure creation
- Registry integration

**lib/delete.sh** - Project deletion
- Removal from registry
- Directory cleanup
- Confirmation prompts

**lib/symlink.sh** - Symlink management
- Browse existing symlinks
- Create symlink organization
- Status checking

### Data Format

**Registry Format (JSON)**
```json
{
  "version": "1.0.0",
  "projects": {
    "project-name": {
      "path": "/absolute/path",
      "added": "ISO8601",
      "lastAccessed": "ISO8601"
    }
  },
  "metadata": {
    "created": "ISO8601",
    "updated": "ISO8601",
    "totalProjects": 1
  }
}
```

### Workflow Integration

1. **Registry Initialization**
   - Creates ~/.claude/registry.json
   - Sets initial structure and timestamps

2. **Project Management**
   - Add: Create registry entry with metadata
   - Delete: Remove from registry, optionally delete directory
   - Create: Initialize directory and git repo

3. **Navigation**
   - List: Show all projects formatted
   - Select: Interactive FZF selection
   - Preview: Show project details

4. **Symlinks**
   - Browse: List all project symlinks
   - Organize: Create symlinks in target directory

## Dependencies

### Required
- bash 4.0+ - Shell scripting
- jq - JSON query and manipulation

### Optional
- fzf - Interactive selection (required for select command)
- git - Project initialization (required for create command)

## Installation Targets

### Homebrew
- Installs to /usr/local/bin
- Creates registry directory
- Sets up post-install hooks

### Nix
- Installs to Nix store
- Propagates jq and fzf dependencies
- Provides symlink convenience

### Manual
- Configurable installation prefix
- Copies files to destination
- Updates library paths

## Testing Strategy

### Test Coverage
- 80% minimum code coverage
- Unit tests for all functions
- Integration tests for workflows
- Edge case testing

### Test Suite
- **registry-init.bats** - Registry initialization
- **project-select.bats** - Project selection
- **integration.bats** - Complete workflows

## Error Handling

### Error Types
1. Missing dependencies - User-friendly message
2. Invalid input - Validation error
3. File operations - Permission/existence errors
4. Registry corruption - Backup restoration

### Error Recovery
- Automatic backups before modifications
- Graceful degradation
- Clear error messages

## Performance Considerations

1. **Registry Queries** - Uses jq for efficient JSON parsing
2. **Project Listing** - Caches registry data
3. **FZF Selection** - Streamed list for large projects
4. **File Operations** - Lazy initialization where possible

## Security

1. **Input Validation** - All user inputs validated
2. **File Permissions** - Respects directory permissions
3. **Registry Integrity** - Validates JSON before modifications
4. **Backup Creation** - Automatic backups before changes

## Future Extensibility

- Plugin system for custom commands
- Remote registry support
- Project templates
- Integration with other tools
- Tab completion scripts
- Web UI for management
