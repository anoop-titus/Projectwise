#!/usr/bin/env bash
# Project cleanup: prune old data, report sizes

source "$(dirname "${BASH_SOURCE[0]}")/core.sh"

prune_old_data() {
  local days="${1:-30}"

  print_info "Pruning data older than ${days} days..."
  echo ""

  local total_freed=0

  for project_dir in "$CPM_HOME"/*/; do
    [[ -d "$project_dir" ]] || continue
    local name
    name=$(basename "$project_dir")
    local freed=0

    for subdir in .cache .tldr; do
      if [[ -d "$project_dir/$subdir" ]]; then
        local before
        before=$(du -sk "$project_dir/$subdir" 2>/dev/null | awk '{print $1}')
        find "$project_dir/$subdir" -type f -mtime "+${days}" -delete 2>/dev/null
        find "$project_dir/$subdir" -type d -empty -delete 2>/dev/null
        local after
        after=$(du -sk "$project_dir/$subdir" 2>/dev/null | awk '{print $1}')
        freed=$((freed + before - after))
      fi
    done

    # Old jsonl logs
    local log_size
    log_size=$(find "$project_dir" -maxdepth 1 -name "*.jsonl" -mtime "+${days}" -exec du -sk {} + 2>/dev/null | awk '{s+=$1}END{print s+0}')
    find "$project_dir" -maxdepth 1 -name "*.jsonl" -mtime "+${days}" -delete 2>/dev/null
    freed=$((freed + log_size))

    if ((freed > 0)); then
      echo "  $name: freed ${freed}KB"
      total_freed=$((total_freed + freed))
    fi
  done

  echo ""
  print_success "Total freed: ${total_freed}KB"
}

size_report() {
  echo -e "${CPM_CYAN}Project Size Report${CPM_NC}"
  echo "==================="
  echo ""

  printf "%-45s %8s %8s %8s\n" "Project" "Total" "Data" "Code"
  printf "%-45s %8s %8s %8s\n" "-------" "-----" "----" "----"

  local grand_total=0

  for project_dir in "$CPM_HOME"/*/; do
    [[ -d "$project_dir" ]] || continue
    local name
    name=$(basename "$project_dir")
    local total
    total=$(du -sh "$project_dir" 2>/dev/null | awk '{print $1}')
    local total_kb
    total_kb=$(du -sk "$project_dir" 2>/dev/null | awk '{print $1}')
    grand_total=$((grand_total + total_kb))

    local data_kb=0
    for subdir in .cache .tldr; do
      [[ -d "$project_dir/$subdir" ]] && \
        data_kb=$((data_kb + $(du -sk "$project_dir/$subdir" 2>/dev/null | awk '{print $1}')))
    done
    data_kb=$((data_kb + $(find "$project_dir" -maxdepth 1 -name "*.jsonl" -exec du -sk {} + 2>/dev/null | awk '{s+=$1}END{print s+0}')))

    local code_kb=$((total_kb - data_kb))
    local data_h="${data_kb}K" code_h="${code_kb}K"
    ((data_kb > 1024)) && data_h="$((data_kb / 1024))M"
    ((code_kb > 1024)) && code_h="$((code_kb / 1024))M"

    printf "%-45s %8s %8s %8s\n" "$name" "$total" "$data_h" "$code_h"
  done

  echo ""
  local grand_h="${grand_total}K"
  ((grand_total > 1024)) && grand_h="$((grand_total / 1024))M"
  echo "Grand total: $grand_h"
}

cleanup_main() {
  local subcommand="${1:-help}"
  case "$subcommand" in
    prune)
      shift
      local days=30
      while [[ $# -gt 0 ]]; do
        case "$1" in
          --days) days="$2"; shift 2 ;;
          *) shift ;;
        esac
      done
      prune_old_data "$days"
      ;;
    report)  size_report ;;
    help|--help|-h)
      cat << 'HELP'
Cleanup commands:
  prune [--days N]   Remove old .cache, .tldr, logs (default: 30 days)
  report             Show project size breakdown
HELP
      ;;
    *) die "Unknown cleanup command: $subcommand" ;;
  esac
}
