# Installation Guide

## Prerequisites

- macOS 10.14+ or Linux
- bash 4.0+
- jq (for JSON processing)
- fzf (optional, for interactive selection)

## Installation Methods

### 1. Quick Install

```bash
git clone https://github.com/titus/claude-project-manager.git
cd claude-project-manager/package
./scripts/install.sh
```

### 2. Custom Prefix

```bash
./scripts/install.sh /usr/local
```

### 3. Homebrew

```bash
brew tap titus/formulae
brew install claude-project-manager
```

### 4. Nix

```bash
nix profile install github:titus/claude-project-manager#claude-project-manager
```

## Post-Installation

### 1. Initialize Registry

```bash
claude-pm registry init
```

This creates `~/.claude/registry.json` with the initial structure.

### 2. Shell Integration

Add to your shell configuration file (`~/.zshrc` or `~/.bashrc`):

```bash
export CLAUDE_REGISTRY_PATH="$HOME/.claude/registry.json"

# Optional: Add convenience aliases
alias cpm="claude-pm"
alias cpp="claude-pm select"
```

Reload your shell:

```bash
source ~/.zshrc  # or source ~/.bashrc
```

### 3. Verify Installation

```bash
claude-pm help
claude-pm list
```

## Dependency Installation

### macOS

Using Homebrew:

```bash
# Required
brew install jq

# Optional but recommended
brew install fzf
```

Using Nix:

```bash
nix profile install nixpkgs#jq
nix profile install nixpkgs#fzf
```

### Linux

Using apt (Debian/Ubuntu):

```bash
sudo apt-get install jq fzf
```

Using yum (RedHat/CentOS):

```bash
sudo yum install jq fzf
```

## Upgrade

### From Git

```bash
cd /path/to/claude-project-manager
git pull origin main
./scripts/install.sh
```

### From Homebrew

```bash
brew upgrade claude-project-manager
```

### From Nix

```bash
nix profile upgrade claude-project-manager
```

## Uninstallation

### Script-based

```bash
./scripts/uninstall.sh
```

Or with custom prefix:

```bash
./scripts/uninstall.sh /usr/local
```

### Homebrew

```bash
brew uninstall claude-project-manager
```

### Nix

```bash
nix profile remove claude-project-manager
```

## Troubleshooting

### Command not found

Ensure `/usr/local/bin` is in your PATH:

```bash
echo $PATH
export PATH="/usr/local/bin:$PATH"
```

Add to your shell configuration if missing.

### Permission denied

Make scripts executable:

```bash
chmod +x /usr/local/bin/claude-pm
chmod +x /usr/local/bin/cpm
```

### Registry initialization fails

Check directory permissions:

```bash
mkdir -p ~/.claude
chmod 755 ~/.claude
```

### fzf not found

Install fzf:

```bash
brew install fzf
```

Or build from source:

```bash
git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
~/.fzf/install
```

## Verification

After installation, verify:

```bash
# Check binary location
which claude-pm

# Check version
claude-pm version

# Initialize registry
claude-pm registry init

# List projects (should be empty initially)
claude-pm list
```

## Next Steps

1. Add your first project: `claude-pm registry add <name> <path>`
2. Create a new project: `claude-pm create <name>`
3. Try interactive selection: `claude-pm select`
4. Set up shell aliases for convenience

See USAGE.md for more commands and examples.
