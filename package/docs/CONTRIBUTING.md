# Contributing Guide

## Code of Conduct

Be respectful, inclusive, and professional in all interactions.

## Getting Started

### 1. Fork and Clone

```bash
git clone https://github.com/titus/claude-project-manager.git
cd claude-project-manager
git remote add upstream https://github.com/titus/claude-project-manager.git
```

### 2. Create Feature Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming:
- `feature/new-command` - New features
- `fix/bug-description` - Bug fixes
- `docs/update-name` - Documentation
- `test/test-name` - Tests

### 3. Development Setup

```bash
cd package
./scripts/install.sh .

# Run tests
bats tests/*.bats
```

## Code Style

### Shell Script Standards

1. **Shebang**: Always use `#!/usr/bin/env bash`
2. **Set options**: Use `set -euo pipefail` at script start
3. **Variable names**: Use UPPER_CASE for constants, lower_case for variables
4. **Functions**: Use descriptive names, document with comments
5. **Error handling**: Always check command success
6. **Comments**: Document complex logic

### Example Function

```bash
# Description of what this function does
# Args: $1 - first argument description
#       $2 - second argument description
# Returns: 0 on success, 1 on error
my_function() {
  local arg1="$1"
  local arg2="$2"
  
  # Validate inputs
  if [[ -z "${arg1}" ]]; then
    print_error "arg1 cannot be empty"
    return 1
  fi
  
  # Do work
  local result
  result=$(do_something "${arg1}" "${arg2}") || return 1
  
  # Return success
  echo "${result}"
  return 0
}
```

## Testing

### Test Requirements

- Minimum 80% code coverage
- Tests for all new functions
- Integration tests for workflows
- Edge case testing

### Running Tests

```bash
cd package
bats tests/*.bats

# Run specific test file
bats tests/registry-init.bats

# Run with verbose output
bats --verbose tests/*.bats
```

### Writing Tests

Create test files in `tests/` with `.bats` extension:

```bash
#!/usr/bin/env bats

load test_helper

@test "registry_init creates registry file" {
  run registry_init
  [ "$status" -eq 0 ]
  [ -f "$REGISTRY_PATH" ]
}

@test "registry_add_project adds project" {
  run registry_add_project "test-project" "/path/to/project"
  [ "$status" -eq 0 ]
}
```

## Commit Guidelines

### Message Format

```
<type>: <subject>

<body>

<footer>
```

### Types

- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `test` - Test additions/changes
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `chore` - Build, dependencies, etc.

### Examples

```
feat: add symlink organize command

Allows users to create symlinks for all projects in a target directory.
Implements --force flag for overwriting existing symlinks.

Closes #42
```

```
fix: handle empty registry gracefully

Previously crashed when registry had no projects.
Now displays helpful message and exits cleanly.

Fixes #39
```

## Pull Request Process

1. **Update main branch**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run tests and linting**
   ```bash
   bats tests/*.bats
   shellcheck bin/* lib/*
   ```

3. **Push to fork**
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create pull request**
   - Clear description of changes
   - Reference related issues
   - Screenshots/examples if applicable

5. **Address review feedback**
   ```bash
   git add <files>
   git commit -m "Address review feedback"
   git push origin feature/your-feature-name
   ```

## Documentation

### Update README

- Add command to appropriate section
- Update examples if applicable
- Add to Table of Contents if new section

### Update USAGE.md

Add usage examples for new features:

```markdown
### New Feature

Description of feature.

```bash
claude-pm new-command [options]
```

Example output and behavior.
```

### Update CHANGELOG

Add entry to `Unreleased` section:

```markdown
## [Unreleased]

### Added
- New symlink organize feature
- Registry export functionality

### Fixed
- Handle empty project paths

### Changed
- Improved error messages
```

## Issue Reporting

### Bug Reports

Include:
1. Claude PM version: `claude-pm version`
2. OS and version: `uname -a`
3. Steps to reproduce
4. Expected vs actual behavior
5. Error messages/logs

### Feature Requests

Include:
1. Use case/motivation
2. Proposed behavior
3. Example commands
4. Acceptance criteria

## Release Process

Maintainers only:

1. Update VERSION file
2. Update CHANGELOG.md
3. Create git tag: `git tag v1.0.0`
4. Push changes and tag
5. Create GitHub release
6. Update Homebrew formula
7. Update Nix package

## Questions?

- Open an issue for discussion
- Check existing issues/PRs first
- Ask in discussions section

## License

All contributions are licensed under MIT License.

By contributing, you agree that your contributions will be licensed under its MIT License.
