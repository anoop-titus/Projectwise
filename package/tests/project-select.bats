#!/usr/bin/env bats

load test_helper

setup() { setup_test_registry; }
teardown() { teardown_test_registry; }

@test "list names returns added projects" {
  registry_add "proj-a" "Project A"
  registry_add "proj-b" "Project B"
  local names
  names=$(registry_list_names)
  echo "$names" | grep -q "proj-a"
  echo "$names" | grep -q "proj-b"
}

@test "get project returns correct data" {
  registry_add "my-proj" "My Project" "Description" "Research"
  local dn
  dn=$(registry_get_project "my-proj" | jq -r '.display_name')
  [ "$dn" = "My Project" ]
}

@test "toggle favorite works" {
  registry_add "fav-proj" "Fav"
  [ "$(jq -r '.projects[0].favorite' "$TEST_PROJECTS_DIR/.registry.json")" = "false" ]
  registry_toggle_favorite "fav-proj"
  [ "$(jq -r '.projects[0].favorite' "$TEST_PROJECTS_DIR/.registry.json")" = "true" ]
}

@test "set display name updates correctly" {
  registry_add "rename-proj" "Old Name"
  registry_set_display_name "rename-proj" "New Name"
  [ "$(jq -r '.projects[0].display_name' "$TEST_PROJECTS_DIR/.registry.json")" = "New Name" ]
}

@test "increment sessions works" {
  registry_add "session-proj" "Sessions"
  registry_increment_sessions "session-proj"
  registry_increment_sessions "session-proj"
  [ "$(jq -r '.projects[0].session_count' "$TEST_PROJECTS_DIR/.registry.json")" -eq 2 ]
}
