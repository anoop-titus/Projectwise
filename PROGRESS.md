# Projectwise — Progress Log

## v3.8.0 — 2026-06-16 — Agents tab, intro-prompt template + in-TUI editor

Three features (version 3.7.1 → 3.8.0 in `Cargo.toml` + CLI/banner); tab set grows 4 → 6:

1. **Agents tab (tab 5 / key `5`)** — new `render_agents_tab` reads the global agent registry (`~/.claude/agents/registry.json`, a flat array of ~145 agents). Scrollable id+name list on the left, detail pane on the right (alias, industry, framework, skillset, use case, source URL), mirroring the PROGRESS.html Agents view. Read-only: `j`/`k` select, PgUp/PgDn scroll the detail pane.
2. **Intro-prompt editor tab (tab 6 / key `6`)** — the Claude session intro prompt (previously hardcoded in shell-init) is externalized to an editable template file. Resolution order at launch: per-project `<project>/.projectwise/intro-prompt.tmpl` → global `~/.claude/.projectwise/intro-prompt.tmpl` → built-in default. `g`/`p` switch between the global template and the selected project's override; `e` opens the in-TUI editor; `j`/`k` scroll. The global template is seeded with the default text on first edit.
3. **In-TUI CLAUDE.md editing** — the CLAUDE.md tab's `e` now opens a true in-TUI multiline text editor (new `src/editor.rs` module: UTF-8 aware, `Ctrl+S` save / `Esc` cancel with an unsaved-changes guard) instead of shelling out to `$EDITOR`. The same `editor.rs` module backs the Intro tab.

- **Tabs are now:** 1 Projects · 2 Tokenizer · 3 CLAUDE.md · 4 RULES · 5 Agents · 6 Intro (switch with `[`/`]` or `1`–`6`).
- **Per-project tasks.json migration — no cpm change required:** confirmed cpm reads the aggregate `~/.claude/progress/tasks.json`, which is kept in legacy shape, so the separate per-project tasks.json migration needed no code change here.

**Docs:** README.md updated — version badge 3.7.1 → 3.8.0, tagline/why-table/features rows for the two new tabs + in-TUI editor, tab list (`1`–`6`) everywhere it appears, keybindings table (`Ctrl+S`/`Esc`, PgUp/PgDn, revised `g`/`p` + `e`), `src/` tree gains `editor.rs`, Configuration `$EDITOR` note, new v3.8.0 changelog entry.

## v3.7.1 — 2026-06-14 — PROD hardening (code review + E2E)

Code-architecture-reviewer pass + full E2E/unit sweep ahead of PROD.
- **Panics fixed:** `dirs::home_dir().unwrap()` in `get_home`/`cmd_archive`/`cmd_restore`/`sessions::claude_dir` (→ safe fallback); `strip_html` char-boundary defence.
- **Correctness fixed:** `TextInput`/`RenameInput` cursor was byte-incremented by 1 → multi-byte input could panic; now char-boundary aware. `CategoryPicker` div-by-zero / OOB on empty options guarded.
- **rules-sync self-healing:** prunes dead `.toon` symlinks when a rule's `.toon` is removed externally, and reports orphaned masters (never auto-deletes a master, never auto-recompiles → no data loss, no resurrection of deprecated rules). Removed the decommissioned `agtrace-observability` master.
- **Clippy:** 2 pre-existing `unnecessary_sort_by` (registry.rs/sessions.rs) fixed → clippy now 0.
- **Tests:** 12 → 18 (added validate_folder_name/safe_join traversal, multibyte cursor, strip_html edge cases).
- **E2E (19 checks):** version, shell-init surface, registry/integrity/cleanup, rules-sync idempotency+cleanliness, mirror invariants (20 masters=20 symlinks, 0 broken, rules/ .toon-only), md→toon round-trip, context digest, path-traversal safety (info validated; preview is registry-only/safe), no-panic on bad input — all green.

**Status:** build clean, clippy 0, 18/18 tests, E2E green. Installed `~/.local/bin/cpm` v3.7.1.
**Shipped:** PR #1 merged to `main` as `7c9abe2`, tagged `v3.7.1`, 2026-06-14. README/CI/.gitignore included; manual TUI smoke test passed by user.

## v3.7.0 — 2026-06-14 — Readable .md rule mirror + .toon round-trip

- **New `~/.claude/rules-md/` mirror** of readable `.md` masters; `~/.claude/rules/` is now strictly `.toon` (token-optimized, what Claude reads). Each mirror entry carries a `<stem>.toon` symlink → the live rules `.toon`.
- **`cpm rules-sync`** subcommand (idempotent bootstrap): seeds masters, compiles `.md`-source rules to `.toon` and removes their `.md` from rules/, decodes `.toon`-only rules to readable masters (leaving the existing `.toon` untouched), and creates symlinks. Summary printed. Also fired (backgrounded) from the shell wrapper on each summon.
- **RULES tab round-trip:** `e` now edits the persistent master in `rules-md/` via `$EDITOR`; on save it recompiles to `rules/<stem>.toon` using the user's own `md_to_json | toon -e -o` pipeline (helpers `md_to_json_bin`/`toon_bin`/`md_to_toon`/`ensure_toon_symlink`/`ensure_rule_master`/`compile_rule_master`). A status banner reports save/compile result.
- **Safety (failure-class (a)/(e)):** `.toon` is written atomically via a temp file; on converter failure the master is kept and the previous `.toon` is preserved (no data loss). `.md` is removed from rules/ only after a non-empty `.toon` is written.
- **Bootstrap result:** 21 rules present at run time → 21 readable masters + 21 `.toon` symlinks; rules/ now 0 `.md` / 21 `.toon`. Round-trip + idempotency (2nd run = 0 changes) verified.
- **Caveat surfaced:** 4 rules seen at session start (aqua-agent, docker-permissions, document-auto-processing, mcp-architecture) were already absent before `rules-sync` ran (removed ~21:00 by the user's `_backups_mcp_cleanup`); `rules-sync` only ever deletes `.md`, never `.toon`. aqua-agent is recoverable from backups; the other 3 have no local backup.

**Verification:** `cargo build --release` clean; clippy 0 errors (2 pre-existing); `cargo test` 12/12 (incl. converter env-override + safety-guard test). `cpm v3.7.0` installed to `~/.local/bin/cpm`.

## v3.6.0 — 2026-06-14 — CLAUDE.md & RULES tabs, global rules, interview prompt

Four features (all `src/main.rs` unless noted; version 3.5.0 → 3.6.0):

1. **CLAUDE.md tab** — `Tab` enum extended to 4 (Projects/Tokenizer/ClaudeMd/Rules) with `prev()`; tab bar + `[`/`]` + `1`–`4`. `render_text_tab` shows the active CLAUDE.md scrollably; `g`/`p` toggles global `~/.claude/CLAUDE.md` vs project `~/CLAUDE.md`; `e` edits in `$EDITOR` (`edit_file_external`); `j`/`k`/PgUp/PgDn scroll.
2. **RULES tab** — `render_rules_tab` (list + readable preview). `rule_readable` resolves human-readable text: live `.md` → Tokenizer manifest backup → best-effort `toon_to_readable` decode (since TOON has no lossless decoder and 20/25 rules predate the backup store). `e` edits a temp `.md`, saving via `save_rule_edit`: `.md` rules write back; `.toon` rules write a readable `.md` master, drop the stale `.toon`, and re-run `tokenizer optimize` to reconvert. Helpers: `list_rule_stems`, `manifest_backup_for`, `titlecase`.
3. **Global CLAUDE.md additions** — appended AGILE-not-Waterfall, SPEC-per-plan, and be-precise rules to `~/.claude/CLAUDE.md` (surfaced by the global view of tab 3).
4. **Session interview prompt** — `_projectwise_launch` (shell-init) now always prepends the "interview me / small compartmentalized specs / verify decisions" directive to Claude's initial prompt for project launches (Quick Session unchanged).

**Verification:** `cargo build --release` clean; clippy 0 errors (2 pre-existing warnings); `cargo test` 11/11 (4 new: titlecase, toon_to_readable, tab cycle, strip_html). Decoder previewed on real `agent-spawning.toon` (clean MD) + `dev-standards.toon` (code fences restored). `cpm version` → 3.6.0; shell-init emits interview prefix + 4 tabs. Installed to `~/.local/bin/cpm`.

## v3.5.0 — 2026-06-14 — Search, Tokenizer tab, auto-context

Three features added (all in `src/main.rs`, version bumped 3.4.0 → 3.5.0 in `Cargo.toml` + CLI/banner):

1. **In-TUI `/` fuzzy search** — new `run_project_search` + `suspend_tui`/`resume_tui` helpers. `/` in the Projects tab suspends Ratatui, runs `fzf` over folder names directly under `~/.claude/projects` (disk scope only), and jumps the selection to the match. Footer hint updated.
2. **Tokenizer integration** — new top tab bar (`Tab` enum: Projects | Tokenizer; switch with `[`/`]` or `1`/`2`). Tokenizer tab (`render_tokenizer_tab`) shows binary/timer/hook install state (read-only checks) and action keys: `o` background optimize, `T` open full `tokenizer tui`, `i` install-timer, `h` install-hook (`tokenizer_bin`, `tok_status`, `spawn_tokenizer_optimize`, `run_tokenizer_cmd`). `cmd_shell_init` now bootstraps Tokenizer on every summon: idempotent `install-timer`/`install-hook` + background `optimize --quiet` boost (fail-open).
3. **Auto-absorb PROGRESS + ARCHITECTURE** — `build_context_digest` (+ `strip_html`) writes `<project>/.projectwise/context-digest.md` from `~/.claude/progress/tasks.json` (clean structured progress; PROGRESS.md/PROGRESS.html fallback) plus `ARCHITECTURE.md`. `cmd_pre_launch` builds it non-interactively (replacing the old `less` "Review docs?" prompt); `cmd_shell_init`'s `_projectwise_launch` passes the digest to `claude` as the initial prompt.

**Verification:** `cargo build --release` clean; `cargo clippy` 0 errors (2 pre-existing warnings in registry.rs/sessions.rs); `cargo test` 7/7 pass. Digest generation confirmed on `Tokenizer_1779847795` (progress) and `SOCKS5-Brave_1779094066` (progress + architecture). `cpm version` → 3.5.0; `cpm shell-init` emits the tokenizer bootstrap + digest-injection wrapper. Binary installed to `~/.local/bin/cpm`.

---

## Scribe task log (append-only)

`2026-06-14` · task `pw-v3.5.0` · `v3.5.0 — In-TUI fzf search, Tokenizer tab, context digest` · `done` · commit `90505fa` (PR #1)
`2026-06-14` · task `pw-v3.6.0` · `v3.6.0 — CLAUDE.md + RULES tabs, global rules, interview prompt` · `done` · commit `90505fa` (PR #1)
`2026-06-14` · task `pw-v3.7.0` · `v3.7.0 — Readable .md rule mirror + .toon round-trip` · `done` · commit `90505fa` (PR #1)
`2026-06-14` · task `pw-v3.7.1` · `v3.7.1 — PROD hardening (code review + E2E)` · `done` · commit `90505fa`; merged `7c9abe2`; finalize `67b4ae2`
`2026-06-14` · task `pw-release-v3.7.1` · `Release + install — README, CI, install, PR merge, tag v3.7.1` · `done` · commit `7c9abe2` (merge PR #1), tag `v3.7.1` at `67b4ae2`
`2026-06-16` · task `pw-v3.8.0` · `v3.8.0 — Agents tab, intro-prompt template + in-TUI editor (src/editor.rs)` · `done`
`2026-07-09` · task `pw-v3.9.0` · `v3.9.0 — per-project Model + Effort columns + self-consolidating session handoff` · `done` · commit `fe095f0` on branch `feat/model-effort-handoff-v3.9.0` (not yet merged to main)
`2026-07-09` · task `pw-v3.10.0` · `v3.10.0 — replace Tokenizer with Cloptimizer in tab 2 + shell glue` · `done` · commit `4e9ef03` on branch `feat/cloptimizer-tab-v3.10.0` (builds on v3.9.0's `fe095f0`; not yet merged to main)
