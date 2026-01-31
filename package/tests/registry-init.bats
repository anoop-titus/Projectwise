#!/usr/bin/env bats

load test_helper

setup() { setup_test_registry; }
teardown() { teardown_test_registry; }

@test "registry file exists after setup" {
  [ -f "$TEST_PROJECTS_DIR/.registry.json" ]
}

@test "registry has correct version" {
  local version
  version=$(jq -r '.version' "$TEST_PROJECTS_DIR/.registry.json")
  [ "$version" = "2.0.0" ]
}

@test "registry starts with empty projects" {
  local count
  count=$(jq '.projects | length' "$TEST_PROJECTS_DIR/.registry.json")
  [ "$count" -eq 0 ]
}

@test "add project creates entry" {
  registry_add "test-proj" "Test Project" "A test" "Development"
  local count
  count=$(jq '.projects | length' "$TEST_PROJECTS_DIR/.registry.json")
  [ "$count" -eq 1 ]
  local name
  name=$(jq -r '.projects[0].display_name' "$TEST_PROJECTS_DIR/.registry.json")
  [ "$name" = "Test Project" ]
}

@test "remove project deletes entry" {
  registry_add "test-proj" "Test" "desc" "Dev"
  registry_remove "test-proj"
  local count
  count=$(jq '.projects | length' "$TEST_PROJECTS_DIR/.registry.json")
  [ "$count" -eq 0 ]
}
