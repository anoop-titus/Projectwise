#!/usr/bin/env bats

load test_helper

setup() { setup_test_registry; }
teardown() { teardown_test_registry; }

@test "full workflow: add, list, get, remove" {
  registry_add "workflow-proj" "Workflow Test" "Full test" "Testing"
  registry_list_names | grep -q "workflow-proj"
  local desc
  desc=$(registry_get_project "workflow-proj" | jq -r '.description')
  [ "$desc" = "Full test" ]
  registry_remove "workflow-proj"
  [ "$(jq '.projects | length' "$TEST_PROJECTS_DIR/.registry.json")" -eq 0 ]
}

@test "multiple projects" {
  registry_add "p1" "Project 1"
  registry_add "p2" "Project 2"
  registry_add "p3" "Project 3"
  [ "$(jq '.projects | length' "$TEST_PROJECTS_DIR/.registry.json")" -eq 3 ]
}

@test "set status and category" {
  registry_add "status-proj" "Status Test"
  registry_set_status "status-proj" "paused"
  [ "$(jq -r '.projects[0].status' "$TEST_PROJECTS_DIR/.registry.json")" = "paused" ]
  registry_set_category "status-proj" "Development"
  [ "$(jq -r '.projects[0].category' "$TEST_PROJECTS_DIR/.registry.json")" = "Development" ]
}

@test "backup created on update" {
  registry_add "backup-proj" "Backup Test"
  local backups
  backups=$(ls "$TEST_PROJECTS_DIR/.registry.json.backup."* 2>/dev/null | wc -l)
  [ "$backups" -gt 0 ]
}
