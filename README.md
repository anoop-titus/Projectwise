# Claude Project Manager (Projectwise)

**A production-ready shell utility for managing development projects with integrated workflow automation and FZF-based project selection.**

[![GitHub](https://img.shields.io/badge/GitHub-Projectwise-blue?style=flat-square)](https://github.com/anoop-titus/Projectwise)
[![Version](https://img.shields.io/badge/Version-2.0.0--refactored-brightgreen?style=flat-square)](package/VERSION)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](package/LICENSE)

---

## 🎯 Overview

Claude Project Manager (Projectwise) is a sophisticated shell utility that streamlines development workflow by providing:

- **Interactive Project Selection** - FZF-based TUI for browsing and managing projects
- **Workflow Automation** - Automatic TLDR indexing, documentation review, and registry updates
- **Project Organization** - Central registry of development projects with metadata
- **Session Tracking** - Monitor project activity and session history
- **Favorite Projects** - Quick access to frequently-used projects
- **Project Templates** - Pre-configured project structures for quick setup

### Key Features

✨ **Smart Project Selection**
- Interactive FZF selector with real-time preview
- Browse by mode (quick, favorite, category)
- Keyboard shortcuts for rename, edit, delete, archive

🔍 **Project Discovery**
- Central registry with rich metadata
- Project search and filtering
- Activity tracking and statistics

🚀 **Workflow Integration**
- Automatic documentation review on project entry
- TLDR code indexing for fast navigation
- Registry updates on project access

📊 **Project Analytics**
- Session counting and tracking
- Last-accessed timestamps
- Project statistics and reporting

---

## 📋 System Requirements

- **OS**: macOS or Linux
- **Shell**: Bash 4.0+ or Zsh 5.0+
- **Dependencies**:
  - `fzf` - Interactive fuzzy finder
  - `jq` - JSON processor
  - `gum` (optional) - Enhanced UI prompts
  - `tldr` (optional) - Code documentation indexing

---

## 📦 Installation

### Method 1: Nix (Recommended)

If you have Nix installed:

```bash
# From the project directory
nix run .

# Or install into profile
nix profile install .
```

### Method 2: Manual Installation

1. **Clone or download the project**:
```bash
git clone https://github.com/anoop-titus/Projectwise.git
cd Projectwise
```

2. **Create symlinks to bin directory**:
```bash
# Make scripts available on PATH
mkdir -p ~/.local/bin
ln -sf $(pwd)/package/bin/claude-pm ~/.local/bin/
ln -sf $(pwd)/package/bin/cpm ~/.local/bin/
```

3. **Add to shell profile** (`.zshrc` or `.bashrc`):
```bash
# For Zsh (.zshrc)
eval "$(~/.local/bin/claude-pm shell-init)"
export CLAUDE_PROJECTS_DIR="$HOME/.claude/projects"

# For Bash (.bashrc)
eval "$($HOME/.local/bin/claude-pm shell-init)"
export CLAUDE_PROJECTS_DIR="$HOME/.claude/projects"
```

4. **Install dependencies**:
```bash
# macOS (using Homebrew)
brew install fzf jq gum

# Linux (using apt)
sudo apt-get install fzf jq gum

# Using Nix
nix profile install nixpkgs#fzf nixpkgs#jq nixpkgs#gum
```

5. **Reload shell**:
```bash
source ~/.zshrc  # or ~/.bashrc
```

### Method 3: Using Homebrew (When Available)

Coming soon!

```bash
brew tap anoop-titus/projectwise
brew install projectwise
```

---

## 🚀 Quick Start

### Initialize Projects Directory

```bash
# First time setup (interactive)
claude-pm setup

# Or manually initialize
mkdir -p ~/.claude/projects
claude-pm registry init
```

### Basic Usage

```bash
# Open project selector (interactive)
claude

# Or use the full command
claude-pm select

# List projects
claude-pm list
claude-pm list quick      # Quick view
claude-pm list favorite   # Only favorites
claude-pm list all        # All projects

# Create a new project
claude-pm create

# Show project info
claude-pm info my-project

# Edit project metadata
claude-pm edit my-project

# Archive/restore projects
claude-pm archive my-project
claude-pm restore my-project

# Permanently delete from registry
claude-pm delete my-project

# View status and statistics
claude-status
claude-status --detailed
claude-status --json
```

### Environment Variables

```bash
# Set custom projects directory
export CLAUDE_PROJECTS_DIR="$HOME/.my-projects"

# Set custom archive directory
export CLAUDE_ARCHIVE_DIR="$HOME/.my-archive"

# Set custom pager
export PAGER="less -R"
```

---

## 📖 Usage Guide

### Interactive Project Selection

The `claude` command opens an interactive FZF-based project selector:

```
Keys:
  R         - Rename project
  M         - Edit metadata
  F         - Toggle favorite
  Ctrl-D    - Archive project
  Enter     - Select and enter project
  Escape    - Cancel
```

### Project Registry

The registry stores project metadata in JSON format:

```json
{
  "version": "2.0.0",
  "projects": [
    {
      "folder_name": "my-project",
      "display_name": "My Project",
      "description": "Project description",
      "category": "Development",
      "status": "active",
      "created": "2026-01-30T10:00:00Z",
      "last_accessed": "2026-01-30T12:30:00Z",
      "session_count": 5,
      "favorite": true
    }
  ]
}
```

### Creating Projects

```bash
# Interactive creation
claude-pm create

# Follow the prompts to set:
# - Project folder name
# - Display name
# - Description
# - Category
# - Create git repo (optional)
```

### Organizing Projects

Create directory structure for organization:

```bash
~/.claude/projects/
├── active/         # Current projects
├── favorites/      # Frequently-used projects
├── archived/       # Inactive projects
└── templates/      # Project templates
```

---

## 🔧 Configuration

### Custom Projects Directory

```bash
# In .zshrc or .bashrc
export CLAUDE_PROJECTS_DIR="/path/to/projects"
eval "$(claude-pm shell-init)"
```

### Disable Documentation Review

```bash
# Edit shell integration to remove doc review section
# Or unset $PAGER variable
unset PAGER
```

### Customize FZF Options

Edit `package/lib/selector.sh` to modify FZF behavior:

```bash
fzf \
  --ansi \
  --delimiter $'\t' \
  --with-nth 1 \
  # ... modify options here
```

---

## 🔒 Security

This refactored version includes security improvements:

- ✅ **Input validation** - Prevents crashes and injection attacks
- ✅ **Binary validation** - Ensures dependencies are available
- ✅ **Secure FZF handling** - Prevents shell injection via placeholders
- ✅ **Directory isolation** - Working directory changes use subshells
- ✅ **Error handling** - Comprehensive error messages without leaking sensitive data

---

## 📚 Documentation

- **[BUG_FIX_REPORT.md](BUG_FIX_REPORT.md)** - Detailed technical analysis of 10 bug fixes
- **[REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md)** - Implementation overview and verification checklist
- **[package/README.md](package/README.md)** - Package-level documentation
- **[package/docs/](package/docs/)** - Additional user guides

### Architecture

See [package/ARCHITECTURE.md](package/ARCHITECTURE.md) for technical architecture details.

### Testing

```bash
# Run BATS test suite
bats package/tests/*.bats

# Run specific tests
bats package/tests/registry-init.bats
```

---

## 🐛 Troubleshooting

### "claude command not found"

**Problem**: The `claude` CLI from Claude Code is not installed.

**Solution**: Install Claude Code:
```bash
# macOS
brew install anthropic/claude/claude

# Or follow installation instructions at https://claude.com/download
```

### "fzf not found"

**Problem**: FZF is not installed.

**Solution**: Install FZF:
```bash
# macOS
brew install fzf

# Linux
sudo apt-get install fzf

# Nix
nix profile install nixpkgs#fzf
```

### "Projects directory not found"

**Problem**: Projects directory doesn't exist.

**Solution**: Initialize projects:
```bash
claude-pm setup
```

### Permission denied errors

**Problem**: Shell scripts not executable.

**Solution**:
```bash
chmod +x package/bin/claude-pm
chmod +x package/bin/cpm
chmod +x package/lib/*.sh
```

---

## 📝 Version History

### v2.0.0-refactored (2026-01-30)

**Major Release**: Production-ready refactoring

✅ Fixed 10 critical/high/medium severity issues:
- Binary validation check added
- Working directory preservation fixed
- Input validation added
- Trap signal stacking resolved
- Backup file rotation implemented
- Shell injection vulnerability patched
- Pager error handling improved
- Date parsing enhanced for timezones
- Emoji sequences corrected
- Shell option inheritance fixed

See [BUG_FIX_REPORT.md](BUG_FIX_REPORT.md) for details.

### v2.0.0 (2026-01-29)

Initial production release of Claude Project Manager.

---

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Commit with clear messages (`git commit -m 'feat: add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

See [package/docs/CONTRIBUTING.md](package/docs/CONTRIBUTING.md) for detailed guidelines.

---

## 📄 License

This project is licensed under the MIT License - see [package/LICENSE](package/LICENSE) for details.

---

## 👤 Author

**Anoop Titus** - [GitHub Profile](https://github.com/anoop-titus)

---

## 🔗 Links

- **GitHub Repository**: https://github.com/anoop-titus/Projectwise
- **Issue Tracker**: https://github.com/anoop-titus/Projectwise/issues
- **Discussions**: https://github.com/anoop-titus/Projectwise/discussions

---

## 📞 Support

For issues, questions, or suggestions:

1. **Check the documentation**: [BUG_FIX_REPORT.md](BUG_FIX_REPORT.md) and [package/docs/](package/docs/)
2. **Search existing issues**: [GitHub Issues](https://github.com/anoop-titus/Projectwise/issues)
3. **Open a new issue**: [Create Issue](https://github.com/anoop-titus/Projectwise/issues/new)
4. **Start a discussion**: [GitHub Discussions](https://github.com/anoop-titus/Projectwise/discussions)

---

## 🎓 Learning Resources

- **Shell Scripting**: [Google's Shell Style Guide](https://google.github.io/styleguide/shellstyle.html)
- **FZF**: [FZF Documentation](https://github.com/junegunn/fzf)
- **Bash**: [GNU Bash Manual](https://www.gnu.org/software/bash/manual/)
- **Zsh**: [Zsh User Guide](http://zsh.sourceforge.net/Guide/)

---

## 📊 Project Statistics

- **Total Commits**: 8
- **Contributors**: 1
- **Last Updated**: 2026-01-30
- **Production Ready**: ✅ Yes
- **Security Audited**: ✅ Yes
- **Code Reviewed**: ✅ Yes

---

## ⭐ Show Your Support

If you find this project useful, please:

1. ⭐ Star the repository
2. 🔔 Watch for updates
3. 💬 Share your feedback
4. 🐛 Report issues
5. 🚀 Submit pull requests

---

**Happy project managing! 🚀**
