#!/usr/bin/env bash
# Migrate existing projects from zshrc/bashrc to registry

set -euo pipefail

# Colors
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_NC='\033[0m'

echo -e "${COLOR_BLUE}Claude Project Manager - Migration Utility${COLOR_NC}\n"

REGISTRY_PATH="${HOME}/.claude/registry.json"

if [[ ! -f "${REGISTRY_PATH}" ]]; then
  echo -e "${COLOR_YELLOW}Registry not found. Run 'claude-pm registry init' first.${COLOR_NC}"
  exit 1
fi

# This script would scan for existing project directories
# and migrate them to the registry format

echo -e "${COLOR_BLUE}Scanning for projects...${COLOR_NC}"

# Look for common project directories
for dir in ~/projects ~/work ~/dev ~/.claude/projects; do
  if [[ -d "${dir}" ]]; then
    echo -e "\nFound directory: ${dir}"
    
    for subdir in "${dir}"/*; do
      if [[ -d "${subdir}" && "${subdir}" != "${dir}/."* ]]; then
        local project_name
        project_name=$(basename "${subdir}")
        
        read -p "Add '${project_name}' to registry? (y/n): " add_project
        if [[ "${add_project}" == "y" ]]; then
          if command -v claude-pm &> /dev/null; then
            claude-pm registry add "${project_name}" "${subdir}"
          else
            echo -e "${COLOR_YELLOW}claude-pm not found in PATH${COLOR_NC}"
          fi
        fi
      fi
    done
  fi
done

echo -e "\n${COLOR_GREEN}✓ Migration complete${COLOR_NC}"
