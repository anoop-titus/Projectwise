mod editor;
mod filetree;
mod models;
mod registry;
mod sessions;
mod theme;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

use models::ListMode;
use registry::RegistryManager;

/// Validate that a folder name is safe: no path separators, no traversal,
/// no shell metacharacters, no null bytes. Only [a-zA-Z0-9._-] allowed.
fn validate_folder_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("folder name cannot be empty");
    }
    if name == "." || name == ".." {
        anyhow::bail!("folder name cannot be '.' or '..'");
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
    {
        anyhow::bail!("folder name contains invalid characters (only [a-zA-Z0-9._-] allowed)");
    }
    Ok(())
}

/// Resolve a folder name against a base directory and verify the result
/// stays within the base. Returns the validated child path.
fn safe_join(base: &std::path::Path, folder: &str) -> Result<PathBuf> {
    validate_folder_name(folder)?;
    let joined = base.join(folder);
    let canonical_base = if base.exists() {
        base.canonicalize()?
    } else {
        base.to_path_buf()
    };
    let canonical_joined = if joined.exists() {
        joined.canonicalize()?
    } else {
        canonical_base.join(folder)
    };
    if !canonical_joined.starts_with(&canonical_base) {
        anyhow::bail!("path traversal detected: folder escapes base directory");
    }
    Ok(canonical_joined)
}

/// Truncate a string to at most `max_chars` characters (char-boundary safe),
/// appending "..." if truncated.
fn truncate_display(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

fn get_home() -> PathBuf {
    std::env::var("CLAUDE_PROJECTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".claude/projects")
        })
}

#[derive(Parser)]
#[command(
    name = "cpm",
    version = "3.8.0",
    about = "Projectwise — TUI project manager for Claude Code"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive FZF project selector
    Select {
        #[arg(default_value = "quick")]
        mode: String,
    },
    /// List projects in a styled table
    List {
        #[arg(default_value = "quick")]
        mode: String,
        /// Output selected project folder to stdout (for shell integration)
        #[arg(long)]
        select: bool,
    },
    /// Preview a project (styled panel)
    Preview { folder: String },
    /// Show detailed project info (JSON)
    Info { folder: String },
    /// Create a new project
    Create,
    /// Edit project metadata interactively
    Edit { folder: String },
    /// Archive a project
    Archive { folder: String },
    /// Restore an archived project
    Restore { folder: String },
    /// Permanently delete a project
    Delete { folder: String },
    /// Cleanup old data
    Cleanup {
        #[command(subcommand)]
        sub: CleanupSub,
    },
    /// Registry operations
    Registry {
        #[command(subcommand)]
        sub: RegistrySub,
    },
    /// Pre-launch hooks (axon, tldr, integrity)
    PreLaunch { folder: String },
    /// Check registry/filesystem integrity
    Integrity {
        #[command(subcommand)]
        sub: IntegritySub,
    },
    /// Emit shell integration code
    ShellInit,
    /// Build/refresh the readable .md rule mirror (~/.claude/rules-md) and make
    /// ~/.claude/rules strictly .toon-only
    #[command(name = "rules-sync")]
    RulesSync,
    /// Show version
    Version,
    /// [internal] TSV output for FZF reload
    #[command(name = "_list-fzf")]
    ListFzf {
        #[arg(default_value = "quick")]
        mode: String,
    },
    /// [internal] Single-line input prompt for FZF keybindings
    #[command(name = "_prompt-input")]
    PromptInput { label: String },
}

#[derive(Subcommand)]
enum CleanupSub {
    /// Remove stale cache/index dirs older than N days
    Prune {
        #[arg(long, default_value = "30")]
        days: u32,
    },
    /// Show per-project size breakdown
    Report,
}

#[derive(Subcommand)]
enum RegistrySub {
    Init,
    Add {
        folder: String,
        #[arg(default_value = "")]
        name: String,
        #[arg(default_value = "Project")]
        description: String,
        #[arg(default_value = "Research")]
        category: String,
    },
    Remove {
        folder: String,
    },
    List,
    Get {
        folder: String,
    },
    Touch {
        folder: String,
    },
    #[command(name = "set-name")]
    SetName {
        folder: String,
        name: String,
    },
    #[command(name = "set-status")]
    SetStatus {
        folder: String,
        status: String,
    },
    #[command(name = "set-field")]
    SetField {
        folder: String,
        field: String,
        value: String,
    },
    #[command(name = "toggle-fav")]
    ToggleFav {
        folder: String,
    },
    #[command(name = "set-tags")]
    SetTags {
        folder: String,
        tags: String,
    },
    /// Auto-detect categories for all projects based on directory contents
    #[command(name = "auto-categorize")]
    AutoCategorize,
}

#[derive(Subcommand)]
enum IntegritySub {
    Check,
    Repair,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let home = get_home();
    theme::init_theme();
    let mgr = RegistryManager::new(&home);

    match cli.command {
        None | Some(Commands::Select { .. }) => {
            let mode_str = match &cli.command {
                Some(Commands::Select { mode }) => mode.as_str(),
                _ => "quick",
            };
            cmd_select(&mgr, &home, mode_str)
        }
        Some(Commands::List { mode, select }) => cmd_list(&mgr, &home, &mode, select),
        Some(Commands::Preview { folder }) => cmd_preview(&mgr, &home, &folder),
        Some(Commands::Info { folder }) => cmd_info(&mgr, &folder),
        Some(Commands::Registry { sub }) => cmd_registry(&mgr, sub),
        Some(Commands::ShellInit) => cmd_shell_init(),
        Some(Commands::RulesSync) => {
            let summary = rules_sync()?;
            println!("{summary}");
            Ok(())
        }
        Some(Commands::Version) => {
            println!("Projectwise v3.8.0");
            Ok(())
        }
        Some(Commands::ListFzf { mode }) => cmd_list_fzf(&mgr, &mode),
        Some(Commands::PromptInput { label }) => cmd_prompt_input(&label),
        Some(Commands::PreLaunch { folder }) => cmd_pre_launch(&mgr, &home, &folder),
        Some(Commands::Create) => cmd_create(&mgr, &home),
        Some(Commands::Edit { folder }) => cmd_edit(&mgr, &folder),
        Some(Commands::Archive { folder }) => cmd_archive(&mgr, &home, &folder),
        Some(Commands::Restore { folder }) => cmd_restore(&mgr, &home, &folder),
        Some(Commands::Delete { folder }) => cmd_delete(&mgr, &home, &folder),
        Some(Commands::Integrity { sub }) => cmd_integrity(&mgr, &home, sub),
        Some(Commands::Cleanup { sub }) => cmd_cleanup(&home, sub),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Select — FZF interactive picker
// ═══════════════════════════════════════════════════════════════════════

fn cmd_select(mgr: &RegistryManager, _home: &std::path::Path, mode: &str) -> Result<()> {
    let mode_parsed: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    let projects = mgr.list_sorted(mode_parsed)?;

    let mut lines = String::new();
    for p in projects.iter() {
        let fav = if p.favorite { "\u{2605} " } else { "  " };
        lines.push_str(&format!("{fav}{}\t{}\n", p.display_name, p.folder_name));
    }
    lines.push_str("  \u{2795} New Project\t__NEW_PROJECT__\n");
    lines.push_str("  \u{1f4ac} Quick Session\t__QUICK_SESSION__\n");

    let cpm = std::env::current_exe()?.display().to_string();

    let output = std::process::Command::new("fzf")
        .args([
            "--ansi", "--delimiter", "\t", "--with-nth", "1",
            "--header", " R:Rename  F:Fav  Ctrl-D:Archive  Enter:Select",
            "--preview", &format!("{cpm} preview {{2}}"),
            "--preview-window", "right:50%:wrap",
            "--bind", &format!("f:execute-silent({cpm} registry toggle-fav {{2}})+reload({cpm} _list-fzf {mode})"),
            "--bind", &format!("ctrl-d:execute-silent({cpm} registry set-status {{2}} archived)+reload({cpm} _list-fzf {mode})"),
            "--bind", &format!("r:execute-silent({cpm} registry set-name {{2}} $({cpm} _prompt-input Name))+reload({cpm} _list-fzf {mode})"),
            "--exit-0",
            "--color", "bg+:#1c1c28,fg+:#00d2d2,hl:#50dc78,hl+:#50dc78,pointer:#00d2d2,prompt:#00d2d2,header:#3c3c50,border:#3c3c50",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(lines.as_bytes());
            }
            child.wait_with_output()
        })?;

    if !output.status.success() {
        std::process::exit(1);
    }

    let selected = String::from_utf8_lossy(&output.stdout);
    let folder = selected.trim().split('\t').nth(1).unwrap_or("").trim();
    if !folder.is_empty() {
        println!("{folder}");
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// List — Full Ratatui table (3-panel layout)
// ═══════════════════════════════════════════════════════════════════════

fn cmd_list(
    mgr: &RegistryManager,
    home: &std::path::Path,
    mode: &str,
    select_mode: bool,
) -> Result<()> {
    use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::prelude::*;
    use std::io;

    let mode_parsed: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    let projects = mgr.list_sorted(mode_parsed)?;

    // Compute sizes once at startup
    let sizes = compute_sizes(&projects, home);

    enable_raw_mode()?;

    // In select mode, render to stderr so shell $() captures only the folder name from stdout.
    // In normal mode, render to stdout (needed for VHS recording and direct use).
    let result = if select_mode {
        let mut writer = io::stderr();
        execute!(writer, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(writer);
        let mut terminal = Terminal::new(backend)?;
        let r = run_list_ui(&mut terminal, mgr, home, mode, select_mode, projects, sizes);
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        r
    } else {
        let mut writer = io::stdout();
        execute!(writer, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(writer);
        let mut terminal = Terminal::new(backend)?;
        let r = run_list_ui(&mut terminal, mgr, home, mode, select_mode, projects, sizes);
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        r
    };

    match result {
        Ok(Some(folder)) => {
            println!("{folder}");
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Compute sizes for all projects in parallel with home dir
pub fn compute_sizes(projects: &[models::Project], base: &std::path::Path) -> Vec<u64> {
    projects
        .iter()
        .map(|p| {
            let dir = base.join(&p.folder_name);
            if dir.exists() {
                dir_size(&dir)
            } else {
                0
            }
        })
        .collect()
}

/// Format bytes as human-readable string
pub fn human_size(bytes: u64) -> String {
    format_size(bytes)
}

/// Load average progress % and repoPath per project from ~/.claude/progress/tasks.json.
/// Key matches CPM project id (e.g. "Swapfest_1779110431").
/// Returns (avg_pct 0-100, repo_path).
fn load_progress_data(home: &std::path::Path) -> HashMap<String, (u8, Option<String>)> {
    let tasks_path = match home.parent() {
        Some(claude_dir) => claude_dir.join("progress/tasks.json"),
        None => return HashMap::new(),
    };
    let content = match std::fs::read_to_string(&tasks_path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };
    let projects = match json["projects"].as_object() {
        Some(p) => p,
        None => return HashMap::new(),
    };
    projects
        .iter()
        .filter_map(|(key, val)| {
            let tasks = val["tasks"].as_array()?;
            if tasks.is_empty() {
                return None;
            }
            let sum: f64 = tasks
                .iter()
                .filter_map(|t| t["progressPct"].as_f64())
                .sum();
            let pct = (sum / tasks.len() as f64).round() as u8;
            let repo_path = val["repoPath"].as_str().map(String::from);
            Some((key.clone(), (pct, repo_path)))
        })
        .collect()
}

/// Strip HTML to plain text: skips <script>/<style> bodies, drops tags, and
/// collapses whitespace. Used only as a PROGRESS.html fallback for the digest.
fn strip_html(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let n = html.len();
    let mut out = String::with_capacity(n / 2);
    let mut i = 0;
    while i < n {
        if html.as_bytes()[i] == b'<' {
            let rest = &lower[i..];
            if rest.starts_with("<script") {
                match lower[i..].find("</script>") {
                    Some(end) => {
                        i += end + "</script>".len();
                        continue;
                    }
                    None => break,
                }
            } else if rest.starts_with("<style") {
                match lower[i..].find("</style>") {
                    Some(end) => {
                        i += end + "</style>".len();
                        continue;
                    }
                    None => break,
                }
            }
            match html[i..].find('>') {
                Some(end) => {
                    i += end + 1;
                    out.push(' ');
                    continue;
                }
                None => break,
            }
        } else {
            // Safety: i < n and i is always on a char boundary (we only advance
            // by ASCII tag bytes or by ch.len_utf8() below), so next() is Some.
            match html[i..].chars().next() {
                Some(ch) => {
                    out.push(ch);
                    i += ch.len_utf8();
                }
                None => break, // unreachable in well-formed input; guards against UB
            }
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Build `<dir>/.projectwise/context-digest.md` from the project's tasks.json
/// progress data + ARCHITECTURE.md so Claude can absorb the latest status on
/// launch. Returns the digest path when a non-trivial digest was written.
fn build_context_digest(
    home: &std::path::Path,
    folder: &str,
    dir: &std::path::Path,
) -> Option<PathBuf> {
    let mut out = String::new();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC");
    out.push_str(&format!("# Projectwise context digest \u{2014} {folder}\n\n"));
    out.push_str(&format!(
        "> Generated {now}. Last-updated PROGRESS + ARCHITECTURE snapshot for this project. Absorb before starting work.\n\n"
    ));

    let mut has_content = false;

    // Progress from ~/.claude/progress/tasks.json (authoritative source).
    let task_entry = home
        .parent()
        .map(|c| c.join("progress/tasks.json"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .and_then(|j| j["projects"][folder].as_object().cloned());

    if let Some(entry) = task_entry {
        if let Some(tasks) = entry.get("tasks").and_then(|t| t.as_array()) {
            let total = tasks.len();
            let avg = if total > 0 {
                (tasks
                    .iter()
                    .filter_map(|t| t["progressPct"].as_f64())
                    .sum::<f64>()
                    / total as f64)
                    .round() as u8
            } else {
                0
            };
            out.push_str(&format!("## Progress ({total} tasks, {avg}% avg)\n\n"));
            for t in tasks {
                let title = t["title"].as_str().unwrap_or("(untitled)");
                let status = t["status"].as_str().unwrap_or("?");
                let pct = t["progressPct"].as_f64().unwrap_or(0.0) as u8;
                out.push_str(&format!("- **{title}** \u{2014} {status}, {pct}%\n"));
                if let Some(ctx) = t["plan"]["context"].as_str() {
                    if !ctx.trim().is_empty() {
                        out.push_str(&format!("  - {}\n", ctx.replace('\n', " ")));
                    }
                }
            }
            out.push('\n');
            has_content = true;
        }
    }

    // Fallback: PROGRESS.md, then PROGRESS.html stripped to text.
    if !has_content {
        if let Ok(md) = std::fs::read_to_string(dir.join("PROGRESS.md")) {
            out.push_str("## Progress (from PROGRESS.md)\n\n");
            out.push_str(&truncate_display(&md, 4000));
            out.push_str("\n\n");
            has_content = true;
        } else if let Ok(html) = std::fs::read_to_string(dir.join("PROGRESS.html")) {
            out.push_str("## Progress (from PROGRESS.html)\n\n");
            out.push_str(&truncate_display(&strip_html(&html), 4000));
            out.push_str("\n\n");
            has_content = true;
        }
    }

    // Architecture from ARCHITECTURE.md at the project root.
    if let Ok(arch) = std::fs::read_to_string(dir.join("ARCHITECTURE.md")) {
        out.push_str("## Architecture (ARCHITECTURE.md)\n\n");
        out.push_str(&arch);
        out.push('\n');
        has_content = true;
    }

    if !has_content {
        return None;
    }

    let digest_dir = dir.join(".projectwise");
    std::fs::create_dir_all(&digest_dir).ok()?;
    let digest_path = digest_dir.join("context-digest.md");
    std::fs::write(&digest_path, out).ok()?;
    Some(digest_path)
}

/// Produce a compact display string for a PROGRESS.html URL.
/// Shows "✓ ~/short/path/PROGRESS.html" or "— " if unavailable.
fn progress_url_cell(repo_path: &str) -> String {
    let html = std::path::Path::new(repo_path).join("PROGRESS.html");
    let exists = html.exists();
    let short = dirs::home_dir()
        .and_then(|h| {
            let hs = h.to_string_lossy().into_owned();
            repo_path.strip_prefix(&hs).map(|rest| format!("~{rest}"))
        })
        .unwrap_or_else(|| repo_path.to_string());
    let indicator = if exists { "\u{2713}" } else { "\u{00b7}" };
    format!("{indicator} {short}/PROGRESS.html")
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusPanel {
    Table,
    Tree,
    Info,
}

impl FocusPanel {
    fn next(self) -> Self {
        match self {
            FocusPanel::Table => FocusPanel::Tree,
            FocusPanel::Tree => FocusPanel::Info,
            FocusPanel::Info => FocusPanel::Table,
        }
    }
}

// ── Top-level tabs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Projects,
    Tokenizer,
    ClaudeMd,
    Rules,
    Agents,
    IntroPrompt,
}

impl Tab {
    fn next(self) -> Self {
        match self {
            Tab::Projects => Tab::Tokenizer,
            Tab::Tokenizer => Tab::ClaudeMd,
            Tab::ClaudeMd => Tab::Rules,
            Tab::Rules => Tab::Agents,
            Tab::Agents => Tab::IntroPrompt,
            Tab::IntroPrompt => Tab::Projects,
        }
    }
    fn prev(self) -> Self {
        match self {
            Tab::Projects => Tab::IntroPrompt,
            Tab::Tokenizer => Tab::Projects,
            Tab::ClaudeMd => Tab::Tokenizer,
            Tab::Rules => Tab::ClaudeMd,
            Tab::Agents => Tab::Rules,
            Tab::IntroPrompt => Tab::Agents,
        }
    }
    fn index(self) -> usize {
        match self {
            Tab::Projects => 0,
            Tab::Tokenizer => 1,
            Tab::ClaudeMd => 2,
            Tab::Rules => 3,
            Tab::Agents => 4,
            Tab::IntroPrompt => 5,
        }
    }
}

// ── Tokenizer integration helpers ───────────────────────────────────

struct TokStatus {
    bin_exists: bool,
    timer_installed: bool,
    hook_installed: bool,
}

/// Resolve the tokenizer binary path (override with TOKENIZER_BIN).
fn tokenizer_bin() -> PathBuf {
    std::env::var("TOKENIZER_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".cargo/bin/tokenizer")
        })
}

/// Read-only inspection of Tokenizer's install state (matches the paths
/// Tokenizer's daemon.rs writes to).
fn tok_status() -> TokStatus {
    let home = dirs::home_dir().unwrap_or_default();
    let timer = {
        #[cfg(target_os = "macos")]
        {
            home.join("Library/LaunchAgents/com.tokenizer.plist")
        }
        #[cfg(target_os = "linux")]
        {
            home.join(".config/systemd/user/tokenizer.timer")
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            home.join(".config/tokenizer/.timer-installed")
        }
    };
    let hook = {
        #[cfg(windows)]
        {
            home.join(".claude/hooks/tokenizer-post-session.ps1")
        }
        #[cfg(not(windows))]
        {
            home.join(".claude/hooks/tokenizer-post-session.sh")
        }
    };
    TokStatus {
        bin_exists: tokenizer_bin().exists() || cmd_exists("tokenizer"),
        timer_installed: timer.exists(),
        hook_installed: hook.exists(),
    }
}

/// Fire a background `tokenizer optimize --quiet` (non-blocking, fail-open).
fn spawn_tokenizer_optimize() {
    let bin = tokenizer_bin();
    std::thread::spawn(move || {
        let _ = std::process::Command::new(&bin)
            .args(["optimize", "--quiet"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    });
}

/// Run a tokenizer subcommand in the foreground (used after suspending the TUI).
fn run_tokenizer_cmd(args: &[&str]) -> Result<()> {
    let _ = std::process::Command::new(tokenizer_bin())
        .args(args)
        .status();
    Ok(())
}

/// Render the Tokenizer status tab into `area`.
fn render_tokenizer_tab(f: &mut ratatui::Frame, area: ratatui::layout::Rect, st: &TokStatus) {
    use ratatui::prelude::*;
    use ratatui::widgets::*;

    let mark = |b: bool| if b { "\u{2713}" } else { "\u{00b7}" };
    let state = |b: bool, on: &str, off: &str| format!("{} {}", mark(b), if b { on } else { off });

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Tokenizer \u{2014} Claude context optimizer (.md/.json \u{2192} .toon)",
            theme::title(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Binary:        ", theme::dim()),
            Span::styled(
                state(st.bin_exists, "installed", "missing (cargo install --path ~/Tokenizer)"),
                theme::row_normal(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Launch timer:  ", theme::dim()),
            Span::styled(state(st.timer_installed, "installed (hourly)", "not installed"), theme::row_normal()),
        ]),
        Line::from(vec![
            Span::styled("  Session hook:  ", theme::dim()),
            Span::styled(state(st.hook_installed, "installed", "not installed"), theme::row_normal()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  Actions", theme::header())]),
        Line::from(vec![
            Span::styled("    o  ", Style::default().fg(theme::accent())),
            Span::styled("run optimize now (background)", theme::dim()),
        ]),
        Line::from(vec![
            Span::styled("    T  ", Style::default().fg(theme::accent())),
            Span::styled("open the full Tokenizer TUI", theme::dim()),
        ]),
        Line::from(vec![
            Span::styled("    i  ", Style::default().fg(theme::accent())),
            Span::styled("install hourly launch timer", theme::dim()),
        ]),
        Line::from(vec![
            Span::styled("    h  ", Style::default().fg(theme::accent())),
            Span::styled("install post-session hook", theme::dim()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Tokenizer is launched + boosted automatically each time you summon Projectwise.",
            theme::dim(),
        )]),
        Line::from(vec![Span::styled(
            "  Switch tabs with [ and ].",
            theme::dim(),
        )]),
    ];

    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border())
            .title(" Tokenizer ")
            .title_style(theme::title()),
    );
    f.render_widget(p, area);
}

/// Suspend the Ratatui alternate screen so an external program (fzf, tokenizer
/// TUI) can own the terminal.
fn suspend_tui<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
) -> Result<()> {
    use crossterm::event::DisableMouseCapture;
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Restore the Ratatui alternate screen after `suspend_tui`.
fn resume_tui<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
) -> Result<()> {
    use crossterm::event::EnableMouseCapture;
    use crossterm::{
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    };
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal.clear()?;
    Ok(())
}

/// Suspend the TUI, run fzf over the folder names directly under
/// `~/.claude/projects` (the `home` dir), restore the TUI, and return the
/// chosen folder name (if any). Scope is intentionally limited to project
/// folder names on disk.
fn run_project_search<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
    home: &std::path::Path,
) -> Result<Option<String>> {
    let mut names: Vec<String> = Vec::new();
    if home.exists() {
        for entry in std::fs::read_dir(home)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let n = entry.file_name().to_string_lossy().to_string();
                if !n.starts_with('.') {
                    names.push(n);
                }
            }
        }
    }
    names.sort();
    if names.is_empty() {
        return Ok(None);
    }

    suspend_tui(terminal)?;
    let input = names.join("\n");
    let output = std::process::Command::new("fzf")
        .args([
            "--prompt",
            "Search projects > ",
            "--height",
            "100%",
            "--color",
            "bg+:#1c1c28,fg+:#00d2d2,hl:#50dc78,hl+:#50dc78,pointer:#00d2d2,prompt:#00d2d2,header:#3c3c50,border:#3c3c50",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(input.as_bytes());
            }
            child.wait_with_output()
        });
    resume_tui(terminal)?;

    match output {
        Ok(out) if out.status.success() => {
            let sel = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if sel.is_empty() {
                Ok(None)
            } else {
                Ok(Some(sel))
            }
        }
        _ => Ok(None),
    }
}

// ── CLAUDE.md + RULES tab helpers ────────────────────────────────────

/// Whether a rule's on-disk canonical form is a readable `.md` or a Tokenizer
/// `.toon` (which must be reconverted on save).
#[derive(Debug, Clone, Copy, PartialEq)]
enum RuleFormat {
    Md,
    Toon,
}

fn claude_md_global_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".claude/CLAUDE.md")
}

fn claude_md_project_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join("CLAUDE.md")
}

/// Global session intro-prompt template. shell-init reads this (per-project
/// override first, then this global default, then a hardcoded fallback).
fn intro_tmpl_global_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude/.projectwise/intro-prompt.tmpl")
}

/// Per-project intro-prompt override inside a project's `.projectwise/` dir.
fn intro_tmpl_project_path(project_dir: &std::path::Path) -> PathBuf {
    project_dir.join(".projectwise/intro-prompt.tmpl")
}

/// The hardcoded default intro prompt — kept in sync with the shell-init
/// fallback so a fresh global template seeds with the canonical text.
const INTRO_PROMPT_DEFAULT: &str = "Interview me to find the real goal of this project. Bias toward small, compartmentalized specs. Make me verify key decisions explicitly so nothing is missed.";

/// Seed the global intro template with the default text if it does not exist.
fn ensure_intro_global() -> PathBuf {
    let p = intro_tmpl_global_path();
    if !p.exists() {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&p, INTRO_PROMPT_DEFAULT);
    }
    p
}

fn rules_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".claude/rules")
}

/// Readable `.md` master mirror of the rules (edited here; compiled to .toon).
fn rules_md_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".claude/rules-md")
}

/// Resolve the `md_to_json` binary (env MD_TO_JSON_BIN → known path → PATH).
fn md_to_json_bin() -> PathBuf {
    if let Ok(p) = std::env::var("MD_TO_JSON_BIN") {
        return PathBuf::from(p);
    }
    let known = dirs::home_dir().unwrap_or_default().join(".local/bin/md_to_json");
    if known.exists() {
        return known;
    }
    PathBuf::from("md_to_json")
}

/// Resolve the `toon` CLI (env TOON_BIN → PATH → newest nvm install).
fn toon_bin() -> PathBuf {
    if let Ok(p) = std::env::var("TOON_BIN") {
        return PathBuf::from(p);
    }
    if cmd_exists("toon") {
        return PathBuf::from("toon");
    }
    // Fall back to the newest ~/.nvm/.../bin/toon we can find.
    let nvm = dirs::home_dir()
        .unwrap_or_default()
        .join(".nvm/versions/node");
    if let Ok(rd) = std::fs::read_dir(&nvm) {
        let mut versions: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path().join("bin/toon"))
            .filter(|p| p.exists())
            .collect();
        versions.sort();
        if let Some(p) = versions.pop() {
            return p;
        }
    }
    PathBuf::from("toon")
}

/// Compile a readable `.md` master into a `.toon` via `md_to_json | toon -e -o`
/// (the same pipeline the user's rules-to-toon hook uses). Errors if either tool
/// fails or produces an empty file, so callers can preserve the previous `.toon`.
fn md_to_toon(md_path: &std::path::Path, toon_path: &std::path::Path) -> Result<()> {
    use std::process::{Command, Stdio};
    let json = Command::new(md_to_json_bin())
        .arg(md_path)
        .stderr(Stdio::null())
        .output()
        .context("running md_to_json")?;
    if !json.status.success() || json.stdout.is_empty() {
        anyhow::bail!("md_to_json failed or produced no output");
    }
    // Convert to a temp file first, then atomically replace, so a failure never
    // truncates the live .toon.
    let tmp = toon_path.with_extension("toon.tmp");
    let mut child = Command::new(toon_bin())
        .args(["-e", "-o"])
        .arg(&tmp)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawning toon")?;
    {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .context("toon stdin")?
            .write_all(&json.stdout)?;
    }
    let status = child.wait().context("waiting for toon")?;
    let ok = status.success()
        && tmp.metadata().map(|m| m.len() > 0).unwrap_or(false);
    if !ok {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("toon conversion failed or produced empty output");
    }
    std::fs::rename(&tmp, toon_path).context("installing compiled .toon")?;
    Ok(())
}

/// Ensure `rules-md/<stem>.toon` is a symlink to the live `rules/<stem>.toon`.
fn ensure_toon_symlink(stem: &str) {
    let target = rules_dir().join(format!("{stem}.toon"));
    if !target.exists() {
        return;
    }
    let link = rules_md_dir().join(format!("{stem}.toon"));
    // Refresh: drop any stale link/file, then recreate.
    let _ = std::fs::remove_file(&link);
    #[cfg(unix)]
    {
        let _ = std::os::unix::fs::symlink(&target, &link);
    }
}

/// One-time / idempotent bootstrap: build the readable `.md` mirror and make
/// `~/.claude/rules/` strictly `.toon`-only. Safe: never removes a `.md` from
/// rules/ unless its `.toon` exists non-empty. Returns a summary string.
fn rules_sync() -> Result<String> {
    let rules = rules_dir();
    let mirror = rules_md_dir();
    std::fs::create_dir_all(&mirror)?;

    let mut created = 0usize;
    let mut compiled = 0usize;
    let mut pruned = 0usize;
    let mut orphaned: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for stem in list_rule_stems() {
        // Files that must stay human-editable .md in rules/ — never toon-ify.
        if stem == "Comprehensive_Rules" {
            continue;
        }
        let live_md = rules.join(format!("{stem}.md"));
        let live_toon = rules.join(format!("{stem}.toon"));
        let master = mirror.join(format!("{stem}.md"));

        if live_md.exists() {
            // .md-source rule: seed the master if absent, compile, then drop the
            // .md from rules/ only once the .toon is safely written.
            if !master.exists() {
                match std::fs::read_to_string(&live_md) {
                    Ok(t) => {
                        if std::fs::write(&master, t).is_ok() {
                            created += 1;
                        }
                    }
                    Err(e) => warnings.push(format!("{stem}: read live .md failed: {e}")),
                }
            }
            match md_to_toon(&master, &live_toon) {
                Ok(()) => {
                    compiled += 1;
                    let _ = std::fs::remove_file(&live_md); // now .toon-only
                }
                Err(e) => warnings.push(format!(
                    "{stem}: kept .md in rules/ (conversion failed: {e})"
                )),
            }
        } else if live_toon.exists() {
            // .toon-only rule: create a readable master from backup/decode; do
            // NOT recompile the existing .toon (a lossy decode round-trip would
            // degrade it). It is regenerated only when the user edits.
            if !master.exists() {
                if let Some((text, _)) = rule_readable(&stem) {
                    if std::fs::write(&master, text).is_ok() {
                        created += 1;
                    }
                }
            }
        }

        // Self-heal: if a compiled .toon exists, (re)point the symlink; otherwise
        // the rule was removed from rules/ externally — prune the dead symlink and
        // flag the orphaned master (never auto-delete a master = no data loss, and
        // never auto-recompile = a deprecated rule is not resurrected).
        if live_toon.exists() {
            ensure_toon_symlink(&stem);
        } else {
            let link = mirror.join(format!("{stem}.toon"));
            if std::fs::symlink_metadata(&link).is_ok() {
                let _ = std::fs::remove_file(&link);
                pruned += 1;
            }
            if master.exists() {
                orphaned.push(stem.clone());
            }
        }
    }

    let mut summary = format!(
        "rules-sync: {created} master(s) created, {compiled} compiled, {pruned} dead symlink(s) pruned, {} orphaned master(s), {} warning(s)",
        orphaned.len(),
        warnings.len()
    );
    if !orphaned.is_empty() {
        summary.push_str(&format!(
            "\n  orphaned (in rules-md but removed from rules/): {}",
            orphaned.join(", ")
        ));
    }
    for w in &warnings {
        summary.push_str(&format!("\n  ! {w}"));
    }
    Ok(summary)
}

/// Tokenizer's manifest + backup store (macOS Application Support path).
fn tokenizer_manifest_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join("Library/Application Support/tokenizer/manifest.jsonl")
}

/// All rule base-names (stems), union of `.md`/`.toon` in ~/.claude/rules and the
/// readable masters in ~/.claude/rules-md. Deduped + sorted; skips conversion.log.
fn list_rule_stems() -> Vec<String> {
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for dir in [rules_dir(), rules_md_dir()] {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                match path.extension().and_then(|e| e.to_str()) {
                    Some("md") | Some("toon") => {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            set.insert(stem.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    set.into_iter().collect()
}

/// Look up the backup (original .md) path for a converted .toon via Tokenizer's
/// manifest.jsonl. Returns the most recent matching backup_path that exists.
fn manifest_backup_for(converted: &std::path::Path) -> Option<PathBuf> {
    let manifest = std::fs::read_to_string(tokenizer_manifest_path()).ok()?;
    let target = converted.to_string_lossy();
    let mut found: Option<PathBuf> = None;
    for line in manifest.lines() {
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v["converted_path"].as_str() == Some(target.as_ref()) {
            if let Some(bp) = v["backup_path"].as_str() {
                let p = PathBuf::from(bp);
                if p.exists() {
                    found = Some(p); // keep last (most recent) match
                }
            }
        }
    }
    found
}

/// Title-case a space-separated identifier ("agent_spawning" → "Agent Spawning").
fn titlecase(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Best-effort render of a Tokenizer `.toon` back to readable Markdown. TOON has
/// no lossless decoder, but the common cues recover most readability: `>>name`
/// section markers become headers, frontmatter/`@type` noise is dropped, and the
/// pervasive `\n`/`\"` escapes are unescaped so embedded markdown + code render.
fn toon_to_readable(raw: &str) -> String {
    let mut out = String::new();
    let mut in_frontmatter = false;
    for (idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim_start();
        if idx == 0 && trimmed == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if trimmed == "---" {
                in_frontmatter = false;
            } else if let Some(d) = trimmed.strip_prefix("description:") {
                out.push_str("> ");
                out.push_str(d.trim());
                out.push_str("\n\n");
            }
            continue;
        }
        if trimmed.starts_with("@type:") {
            continue;
        }
        if let Some(h) = trimmed.strip_prefix(">>") {
            out.push_str(&format!("\n## {}\n", titlecase(&h.replace('_', " "))));
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    // Unescape the common TOON string escapes so code fences + lists render.
    out.replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\t", "    ")
}

/// Resolve a rule to readable Markdown text plus the canonical on-disk format.
/// Prefers the readable master in rules-md, then a live `.md` in rules, then the
/// Tokenizer backup, then a best-effort decode of the raw `.toon`.
fn rule_readable(stem: &str) -> Option<(String, RuleFormat)> {
    let master = rules_md_dir().join(format!("{stem}.md"));
    if master.exists() {
        return std::fs::read_to_string(&master)
            .ok()
            .map(|t| (t, RuleFormat::Md));
    }
    let dir = rules_dir();
    let md = dir.join(format!("{stem}.md"));
    if md.exists() {
        return std::fs::read_to_string(&md).ok().map(|t| (t, RuleFormat::Md));
    }
    let toon = dir.join(format!("{stem}.toon"));
    if toon.exists() {
        if let Some(backup) = manifest_backup_for(&toon) {
            if let Ok(text) = std::fs::read_to_string(&backup) {
                return Some((text, RuleFormat::Toon));
            }
        }
        if let Ok(raw) = std::fs::read_to_string(&toon) {
            let rendered = format!(
                "<!-- decoded from {stem}.toon (no original backup); edit + save to create a readable master -->\n\n{}",
                toon_to_readable(&raw)
            );
            return Some((rendered, RuleFormat::Toon));
        }
    }
    None
}

/// Ensure a readable master exists at `rules-md/<stem>.md`, returning its path.
/// Self-heals by materializing from `rule_readable` if the master is missing.
fn ensure_rule_master(stem: &str) -> Option<PathBuf> {
    let master = rules_md_dir().join(format!("{stem}.md"));
    if master.exists() {
        return Some(master);
    }
    let (text, _) = rule_readable(stem)?;
    std::fs::create_dir_all(rules_md_dir()).ok()?;
    std::fs::write(&master, text).ok()?;
    Some(master)
}

/// Compile the edited master `rules-md/<stem>.md` into `rules/<stem>.toon` and
/// refresh the symlink. On converter failure the previous `.toon` is preserved
/// (md_to_toon writes atomically via a temp file). Returns Ok(()) on success.
fn compile_rule_master(stem: &str) -> Result<()> {
    let master = rules_md_dir().join(format!("{stem}.md"));
    let toon = rules_dir().join(format!("{stem}.toon"));
    md_to_toon(&master, &toon)?;
    ensure_toon_symlink(stem);
    Ok(())
}

/// Open `path` in $EDITOR (fallback `vi`) with the TUI suspended, then restore.
fn edit_file_external<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
    path: &std::path::Path,
) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    suspend_tui(terminal)?;
    let _ = std::process::Command::new(&editor).arg(path).status();
    resume_tui(terminal)?;
    Ok(())
}

/// Render a scrollable plain-text panel (used by the CLAUDE.md tab).
fn render_text_tab(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    title: &str,
    body: &str,
    scroll: u16,
) {
    use ratatui::widgets::*;
    let p = Paragraph::new(body)
        .style(theme::row_normal())
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme::border())
                .title(format!(" {title} "))
                .title_style(theme::title()),
        );
    f.render_widget(p, area);
}

/// Render the RULES tab: list of rule stems on the left, readable preview right.
#[allow(clippy::too_many_arguments)]
fn render_rules_tab(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    stems: &[String],
    selected: usize,
    preview: &str,
    fmt: Option<RuleFormat>,
    scroll: u16,
) {
    use ratatui::prelude::*;
    use ratatui::widgets::*;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(20)])
        .split(area);

    let items: Vec<ListItem> = stems
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == selected {
                Style::default().bg(theme::accent()).fg(theme::bg())
            } else {
                theme::row_normal()
            };
            ListItem::new(s.clone()).style(style)
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border())
            .title(format!(" Rules ({}) ", stems.len()))
            .title_style(theme::title()),
    );
    f.render_widget(list, chunks[0]);

    let fmt_label = match fmt {
        Some(RuleFormat::Md) => "md",
        Some(RuleFormat::Toon) => "toon\u{2192}readable",
        None => "\u{2014}",
    };
    let title = match stems.get(selected) {
        Some(s) => format!(" {s} [{fmt_label}]  \u{2014}  e:edit  j/k:select  PgUp/PgDn:scroll "),
        None => " (no rules) ".to_string(),
    };
    let p = Paragraph::new(preview)
        .style(theme::row_normal())
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme::border())
                .title(title)
                .title_style(theme::title()),
        );
    f.render_widget(p, chunks[1]);
}

// ── Agents tab ───────────────────────────────────────────────────────

/// A flattened registry agent (mirrors the fields PROGRESS.html shows).
#[derive(Clone, Default)]
struct AgentEntry {
    id: i64,
    name: String,
    alias: String,
    industry: String,
    framework: String,
    skillset: String,
    use_case: String,
    source_url: String,
}

fn agents_registry_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude/agents/registry.json")
}

/// Load the agent catalogue from ~/.claude/agents/registry.json (a flat JSON
/// array). Fail-open: returns an empty vec if missing/unparseable.
fn load_agents() -> Vec<AgentEntry> {
    let content = match std::fs::read_to_string(agents_registry_path()) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };
    let arr = match json.as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };
    arr.iter()
        .map(|v| {
            let skillset = v["skillset"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            AgentEntry {
                id: v["id"].as_i64().unwrap_or(0),
                name: v["name"].as_str().unwrap_or("").to_string(),
                alias: v["alias"].as_str().unwrap_or("").to_string(),
                industry: v["industry"].as_str().unwrap_or("").to_string(),
                framework: v["framework"].as_str().unwrap_or("").to_string(),
                skillset,
                use_case: v["useCase"].as_str().unwrap_or("").to_string(),
                source_url: v["sourceUrl"].as_str().unwrap_or("").to_string(),
            }
        })
        .collect()
}

/// Render the Agents tab: scrollable agent list (left) + detail pane (right),
/// mirroring the PROGRESS.html Agents view.
fn render_agents_tab(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    agents: &[AgentEntry],
    selected: usize,
    scroll: u16,
) {
    use ratatui::prelude::*;
    use ratatui::widgets::*;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(20)])
        .split(area);

    // Left: list with a scroll window around the selection.
    let h = chunks[0].height.saturating_sub(2) as usize;
    let top = if h > 0 && selected >= h { selected + 1 - h } else { 0 };
    let items: Vec<ListItem> = agents
        .iter()
        .enumerate()
        .skip(top)
        .take(h.max(1))
        .map(|(i, a)| {
            let style = if i == selected {
                Style::default().bg(theme::accent()).fg(theme::bg())
            } else {
                theme::row_normal()
            };
            ListItem::new(format!("{} {}", a.id, a.name)).style(style)
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border())
            .title(format!(" Agents ({}) ", agents.len()))
            .title_style(theme::title()),
    );
    f.render_widget(list, chunks[0]);

    // Right: detail of the selected agent.
    let detail = match agents.get(selected) {
        Some(a) => format!(
            "{}  (#{} · {})\n\nAlias:     {}\nIndustry:  {}\nFramework: {}\n\nSkillset:\n  {}\n\nUse case:\n  {}\n\nSource:\n  {}",
            a.name, a.id, a.framework, a.alias, a.industry, a.framework, a.skillset, a.use_case, a.source_url
        ),
        None => "(no agents found in ~/.claude/agents/registry.json)".to_string(),
    };
    let title = match agents.get(selected) {
        Some(_) => " Agent detail  \u{2014}  j/k:select  PgUp/PgDn:scroll ".to_string(),
        None => " Agents ".to_string(),
    };
    let p = Paragraph::new(detail)
        .style(theme::row_normal())
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme::border())
                .title(title)
                .title_style(theme::title()),
        );
    f.render_widget(p, chunks[1]);
}

/// Full-screen in-TUI multiline editor modal. Returns Ok(true) if the user
/// saved (Ctrl+S), Ok(false) if they cancelled. Creates parent dirs on save.
fn run_editor<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
    title: &str,
    path: &std::path::Path,
) -> Result<bool> {
    use crossterm::event::{self, Event, KeyCode, KeyModifiers};
    let initial = std::fs::read_to_string(path).unwrap_or_default();
    let mut ed = editor::TextEditor::from_str(&initial);
    let mut status = format!("editing {}", path.display());
    let mut pending_discard = false;
    loop {
        terminal.draw(|f| {
            let area = f.area();
            let head = format!(
                "{title}  \u{2014}  Ctrl+S:save  Esc:cancel{}",
                if ed.dirty { "  \u{25cf} unsaved" } else { "" }
            );
            ed.render(
                f,
                area,
                &head,
                &status,
                theme::border(),
                theme::title(),
                theme::row_normal(),
            );
        })?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Char('s') if ctrl => {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        match std::fs::write(path, ed.to_string()) {
                            Ok(()) => return Ok(true),
                            Err(e) => status = format!("save failed: {e}"),
                        }
                    }
                    KeyCode::Esc => {
                        if !ed.dirty || pending_discard {
                            return Ok(false);
                        }
                        pending_discard = true;
                        status = "Unsaved changes — Esc again to discard, Ctrl+S to save".to_string();
                        continue;
                    }
                    KeyCode::Enter => ed.insert_newline(),
                    KeyCode::Backspace => ed.backspace(),
                    KeyCode::Delete => ed.delete(),
                    KeyCode::Left => ed.left(),
                    KeyCode::Right => ed.right(),
                    KeyCode::Up => ed.up(),
                    KeyCode::Down => ed.down(),
                    KeyCode::Home => ed.home(),
                    KeyCode::End => ed.end(),
                    KeyCode::PageUp => ed.page_up(10),
                    KeyCode::PageDown => ed.page_down(10),
                    KeyCode::Tab => {
                        for _ in 0..2 {
                            ed.insert_char(' ');
                        }
                    }
                    KeyCode::Char(c) if !ctrl => ed.insert_char(c),
                    _ => {}
                }
                pending_discard = false;
            }
        }
    }
}

// ── Overlay states ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Overlay {
    None,
    StatusPicker {
        row: usize,
        selected: usize,
    },
    CategoryPicker {
        row: usize,
        options: Vec<String>,
        selected: usize,
    },
    TextInput {
        row: usize,
        field: String,
        input: String,
        cursor: usize,
    },
    DeleteConfirm {
        row: usize,
    },
    RenameInput {
        row: usize,
        input: String,
        cursor: usize,
    },
}

/// Virtual rows that appear at top of list
const VIRTUAL_QUICK_SESSION: &str = "__QUICK_SESSION__";
const VIRTUAL_NEW_PROJECT: &str = "__NEW_PROJECT__";

fn run_list_ui<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
    mgr: &RegistryManager,
    home: &std::path::Path,
    mode: &str,
    select_mode: bool,
    mut projects: Vec<models::Project>,
    mut sizes: Vec<u64>,
) -> Result<Option<String>> {
    use crossterm::event::{self, Event, KeyCode, MouseButton, MouseEventKind};
    use ratatui::{prelude::*, widgets::*};

    // Load progress data from tasks.json once at startup
    let progress_data = load_progress_data(home);

    // Virtual rows: 2 items at top (Quick Session, New Project)
    let virtual_count: usize = if select_mode { 2 } else { 0 };

    let mut selected = 0usize;
    let mut focus = FocusPanel::Table;
    let mut active_tab = Tab::Projects;
    let mut overlay = Overlay::None;

    // CLAUDE.md tab state: true = global (~/.claude/CLAUDE.md), false = project.
    let mut claude_md_global = true;
    let mut claude_scroll: u16 = 0;
    // RULES tab state.
    let rule_stems = list_rule_stems();
    let mut rule_sel: usize = 0;
    let mut rule_scroll: u16 = 0;
    let mut rule_cache: Option<(String, Option<RuleFormat>)> = None;
    let mut rule_compile_status: Option<String> = None;
    // Agents tab state (registry loaded once).
    let agents = load_agents();
    let mut agent_sel: usize = 0;
    let mut agent_scroll: u16 = 0;
    // Intro-prompt tab state: true = global template, false = selected project.
    let mut intro_global = true;
    let mut intro_scroll: u16 = 0;

    // Stored panel areas for mouse hit-testing
    let mut table_area = Rect::default();
    let mut tree_area: Option<Rect> = None;
    let mut _info_area: Option<Rect> = None;

    // Build file tree for initially selected project
    let mut tree_state = {
        let real_idx = selected.checked_sub(virtual_count);
        real_idx
            .and_then(|i| projects.get(i))
            .map(|p| home.join(&p.folder_name))
            .filter(|dir| dir.exists())
            .map(|dir| filetree::FileTreeState::new(&dir))
    };

    let mode_parsed: ListMode = mode.parse().unwrap_or(ListMode::Quick);

    loop {
        // Rebuild flat list each frame (cheap for small trees)
        let flat = tree_state
            .as_ref()
            .map(|ts| filetree::flatten(&ts.root))
            .unwrap_or_default();

        let tokstat = tok_status();

        // CLAUDE.md tab body (read fresh each frame; files are small).
        let claude_path = if claude_md_global {
            claude_md_global_path()
        } else {
            claude_md_project_path()
        };
        let claude_body = std::fs::read_to_string(&claude_path).unwrap_or_else(|_| {
            format!("(no file at {})", claude_path.display())
        });
        let claude_title = if claude_md_global {
            "CLAUDE.md [global ~/.claude]  —  g/p:switch  e:edit (in-TUI)  j/k:scroll"
        } else {
            "CLAUDE.md [project ~/]  —  g/p:switch  e:edit (in-TUI)  j/k:scroll"
        };

        // Intro-prompt tab body (read fresh each frame). Global template, or the
        // currently-selected project's override; falls back to the default text.
        let intro_project_dir = selected
            .checked_sub(virtual_count)
            .and_then(|i| projects.get(i))
            .map(|p| home.join(&p.folder_name));
        let intro_path = if intro_global {
            intro_tmpl_global_path()
        } else {
            match &intro_project_dir {
                Some(d) => intro_tmpl_project_path(d),
                None => intro_tmpl_global_path(),
            }
        };
        let intro_body = match std::fs::read_to_string(&intro_path) {
            Ok(s) => s,
            Err(_) if intro_global => format!(
                "(no global template yet — press e to create it)\n\nDefault that ships in shell-init:\n\n{INTRO_PROMPT_DEFAULT}"
            ),
            Err(_) => format!(
                "(no per-project override at {})\n\nThis project falls back to the global template, then the built-in default. Press e to create a project-specific intro prompt.",
                intro_path.display()
            ),
        };
        let intro_title = if intro_global {
            "Intro prompt [global]  —  g/p:global/project  e:edit (in-TUI)  j/k:scroll".to_string()
        } else {
            match &intro_project_dir {
                Some(d) => format!(
                    "Intro prompt [project: {}]  —  g/p:global/project  e:edit  j/k:scroll",
                    d.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
                ),
                None => "Intro prompt [no project selected — showing global]".to_string(),
            }
        };

        // RULES tab preview (lazily cached; invalidated on selection change).
        if active_tab == Tab::Rules && rule_cache.is_none() {
            rule_cache = Some(match rule_stems.get(rule_sel) {
                Some(stem) => match rule_readable(stem) {
                    Some((text, fmt)) => (text, Some(fmt)),
                    None => ("(unreadable rule)".to_string(), None),
                },
                None => ("(no rules found)".to_string(), None),
            });
        }
        let (rule_preview_body, rule_fmt) = rule_cache
            .clone()
            .unwrap_or_else(|| (String::new(), None));
        // Prepend the last compile/save status banner, if any.
        let rule_preview = match &rule_compile_status {
            Some(s) => format!("» {s}\n\n{rule_preview_body}"),
            None => rule_preview_body,
        };

        terminal.draw(|f| {
            let area = f.area();
            let width = area.width;

            // Responsive layout
            let (show_tree, show_info) = if width < 80 {
                (false, false)
            } else if width < 120 {
                (true, false)
            } else {
                (true, true)
            };

            let vertical_chunks = if show_tree {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),  // title
                        Constraint::Percentage(55), // table
                        Constraint::Percentage(40), // bottom panels
                        Constraint::Length(1),  // footer
                    ])
                    .split(area)
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(1),
                    ])
                    .split(area)
            };

            // Title bar with tabs
            let tabs = Tabs::new(vec![
                Line::from(" 1 Projects "),
                Line::from(" 2 Tokenizer "),
                Line::from(" 3 CLAUDE.md "),
                Line::from(" 4 RULES "),
                Line::from(" 5 Agents "),
                Line::from(" 6 Intro "),
            ])
            .select(active_tab.index())
            .style(theme::dim())
            .highlight_style(
                Style::default()
                    .fg(theme::accent())
                    .add_modifier(Modifier::BOLD),
            )
            .divider("|")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme::border())
                    .border_type(BorderType::Rounded)
                    .title(format!(
                        " Projectwise \u{2500} {} ({} projects) ",
                        mode,
                        projects.len()
                    ))
                    .title_style(theme::title()),
            );
            f.render_widget(tabs, vertical_chunks[0]);

            // Content area spanning everything between the tab bar and footer
            // (used by non-Projects tabs).
            let tok_content_area = {
                let last = vertical_chunks.len() - 1;
                let top = vertical_chunks[0];
                let ftr = vertical_chunks[last];
                Rect::new(
                    area.x,
                    top.y + top.height,
                    area.width,
                    ftr.y.saturating_sub(top.y + top.height),
                )
            };

            if active_tab == Tab::Tokenizer {
                render_tokenizer_tab(f, tok_content_area, &tokstat);
            } else if active_tab == Tab::ClaudeMd {
                render_text_tab(f, tok_content_area, claude_title, &claude_body, claude_scroll);
            } else if active_tab == Tab::Rules {
                render_rules_tab(
                    f,
                    tok_content_area,
                    &rule_stems,
                    rule_sel,
                    &rule_preview,
                    rule_fmt,
                    rule_scroll,
                );
            } else if active_tab == Tab::Agents {
                render_agents_tab(f, tok_content_area, &agents, agent_sel, agent_scroll);
            } else if active_tab == Tab::IntroPrompt {
                render_text_tab(f, tok_content_area, &intro_title, &intro_body, intro_scroll);
            } else {
            // Table (panel index 1)
            table_area = vertical_chunks[1];
            let table_border_style = if focus == FocusPanel::Table {
                Style::default().fg(theme::accent())
            } else {
                theme::border()
            };

            let header_cells = ["", "Name", "Category", "Status", "Sessions", "Size", "Progress", "URL"]
                .iter()
                .map(|h| Cell::from(*h).style(theme::header()));
            let header = Row::new(header_cells).height(1);

            // Build rows: virtual rows first (if select_mode), then real projects
            let mut rows: Vec<Row> = Vec::with_capacity(virtual_count + projects.len());

            if select_mode {
                // Quick Session virtual row
                let qs_style = if selected == 0 {
                    Style::default().bg(theme::accent()).fg(theme::bg())
                } else {
                    Style::default().fg(theme::accent())
                };
                rows.push(Row::new(vec![
                    Cell::from("\u{1f4ac}").style(qs_style),
                    Cell::from("Quick Session").style(qs_style),
                    Cell::from("").style(qs_style),
                    Cell::from("").style(qs_style),
                    Cell::from("").style(qs_style),
                    Cell::from("").style(qs_style),
                    Cell::from("").style(qs_style),
                    Cell::from("").style(qs_style),
                ]));

                // New Project virtual row
                let np_style = if selected == 1 {
                    Style::default().bg(theme::accent()).fg(theme::bg())
                } else {
                    Style::default().fg(theme::accent())
                };
                rows.push(Row::new(vec![
                    Cell::from("\u{2795}").style(np_style),
                    Cell::from("New Project").style(np_style),
                    Cell::from("").style(np_style),
                    Cell::from("").style(np_style),
                    Cell::from("").style(np_style),
                    Cell::from("").style(np_style),
                    Cell::from("").style(np_style),
                    Cell::from("").style(np_style),
                ]));
            }

            for (i, p) in projects.iter().enumerate() {
                let row_idx = virtual_count + i;
                let fav = if p.favorite { "\u{2605}" } else { " " };
                let name = truncate_display(&p.display_name, 32);
                let status_str = p.status.to_string();
                let size_str = if i < sizes.len() { human_size(sizes[i]) } else { "\u{2014}".to_string() };

                let base = if row_idx.is_multiple_of(2) { theme::row_normal() } else { theme::row_alt() };

                let (prog_cell, prog_style, url_text) = match progress_data.get(&p.id) {
                    Some((pct, repo)) => {
                        let pct_str = format!("{:>3}%", pct);
                        let pstyle = if *pct >= 80 {
                            Style::default().fg(ratatui::style::Color::Green)
                        } else if *pct >= 40 {
                            Style::default().fg(ratatui::style::Color::Yellow)
                        } else {
                            Style::default().fg(ratatui::style::Color::Red)
                        };
                        let url = repo.as_deref()
                            .map(progress_url_cell)
                            .unwrap_or_else(|| "\u{2014}".to_string());
                        (pct_str, pstyle, url)
                    }
                    None => ("\u{2014}   ".to_string(), theme::dim(), "\u{2014}".to_string()),
                };

                rows.push(Row::new(vec![
                    Cell::from(fav).style(theme::favorite()),
                    Cell::from(name).style(base),
                    Cell::from(p.category.clone()).style(base),
                    Cell::from(status_str.clone()).style(theme::status_style(&status_str)),
                    Cell::from(format!("{:>4}", p.session_count)).style(base),
                    Cell::from(size_str).style(theme::dim()),
                    Cell::from(prog_cell).style(prog_style),
                    Cell::from(truncate_display(&url_text, 45)).style(theme::dim()),
                ]));
            }

            let widths = [
                Constraint::Length(2),
                Constraint::Min(20),
                Constraint::Length(14),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Min(10),
            ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(table_border_style)
                    .border_type(BorderType::Rounded)
                    .title(" Projects ")
                    .title_style(theme::title()))
                .row_highlight_style(Style::default().bg(theme::accent()).fg(theme::bg()))
                .highlight_symbol(" \u{25b6} ");

            let mut table_widget_state = TableState::default();
            if (virtual_count + projects.len()) > 0 { table_widget_state.select(Some(selected)); }
            f.render_stateful_widget(table, vertical_chunks[1], &mut table_widget_state);

            // Bottom panels
            if show_tree {
                let bottom_area = vertical_chunks[2];
                let bottom_chunks = if show_info {
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(bottom_area)
                } else {
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(100)])
                        .split(bottom_area)
                };

                tree_area = Some(bottom_chunks[0]);

                // Dir tree
                if let Some(ref ts) = tree_state {
                    filetree::render_tree(
                        ts,
                        &flat,
                        bottom_chunks[0],
                        f,
                        focus == FocusPanel::Tree,
                        "Directory Tree",
                    );
                } else {
                    let border_s = if focus == FocusPanel::Tree {
                        Style::default().fg(theme::accent())
                    } else {
                        theme::border()
                    };
                    let block = Block::default()
                        .title(" Directory Tree ")
                        .borders(Borders::ALL)
                        .border_style(border_s)
                        .border_type(BorderType::Rounded);
                    f.render_widget(block, bottom_chunks[0]);
                }

                // Info panel
                if show_info {
                    _info_area = Some(bottom_chunks[1]);
                    let info_border_style = if focus == FocusPanel::Info {
                        Style::default().fg(theme::accent())
                    } else {
                        theme::border()
                    };

                    let real_idx = selected.checked_sub(virtual_count);
                    let info_text = if let Some(p) = real_idx.and_then(|i| projects.get(i)) {
                        let ri = real_idx.unwrap_or(0);
                        let size_str = if ri < sizes.len() { human_size(sizes[ri]) } else { "\u{2014}".to_string() };
                        let tags_str = if p.tags.is_empty() { "\u{2014}".to_string() } else { p.tags.join(", ") };
                        let git_str = p.git_link.as_deref().unwrap_or("\u{2014}");
                        let last_str = relative_time(&p.last_accessed);
                        let (prog_info, url_info) = match progress_data.get(&p.id) {
                            Some((pct, repo)) => {
                                let ps = format!("{}%", pct);
                                let u = repo.as_deref()
                                    .map(|r| format!("file://{}/PROGRESS.html", r))
                                    .unwrap_or_else(|| "\u{2014}".to_string());
                                (ps, u)
                            }
                            None => ("\u{2014}".to_string(), "\u{2014}".to_string()),
                        };
                        vec![
                            Line::from(vec![
                                Span::styled("Desc:     ", theme::dim()),
                                Span::styled(truncate_display(&p.description, 30), theme::row_normal()),
                            ]),
                            Line::from(vec![
                                Span::styled("Tags:     ", theme::dim()),
                                Span::styled(tags_str, Style::default().fg(theme::accent())),
                            ]),
                            Line::from(vec![
                                Span::styled("Git:      ", theme::dim()),
                                Span::styled(truncate_display(git_str, 30), theme::dim()),
                            ]),
                            Line::from(vec![
                                Span::styled("Progress: ", theme::dim()),
                                Span::styled(prog_info, theme::row_normal()),
                            ]),
                            Line::from(vec![
                                Span::styled("URL:      ", theme::dim()),
                                Span::styled(truncate_display(&url_info, 35), theme::dim()),
                            ]),
                            Line::from(vec![
                                Span::styled("Created:  ", theme::dim()),
                                Span::styled(p.created.format("%Y-%m-%d").to_string(), theme::row_normal()),
                            ]),
                            Line::from(vec![
                                Span::styled("Last:     ", theme::dim()),
                                Span::styled(last_str, theme::row_normal()),
                            ]),
                            Line::from(vec![
                                Span::styled("Sessions: ", theme::dim()),
                                Span::styled(p.session_count.to_string(), theme::row_normal()),
                            ]),
                            Line::from(vec![
                                Span::styled("Size:     ", theme::dim()),
                                Span::styled(size_str, theme::row_normal()),
                            ]),
                        ]
                    } else {
                        vec![Line::from("No project selected")]
                    };

                    let info = Paragraph::new(info_text)
                        .block(Block::default()
                            .title(" Project Info ")
                            .borders(Borders::ALL)
                            .border_style(info_border_style)
                            .border_type(BorderType::Rounded));
                    f.render_widget(info, bottom_chunks[1]);
                }
            } else {
                tree_area = None;
                _info_area = None;
            }
            } // end Projects-tab content

            // Footer
            let footer_text: &str = if active_tab == Tab::Tokenizer {
                " q:Quit  [ ]:Switch tab  o:Optimize  T:Tokenizer-TUI  i:Install-Timer  h:Install-Hook"
            } else if active_tab == Tab::ClaudeMd {
                " q:Quit  [ ]:Switch tab  g/p:Global/Project  e:Edit  j/k:Scroll"
            } else if active_tab == Tab::Rules {
                " q:Quit  [ ]:Switch tab  j/k:Select  e:Edit  PgUp/PgDn:Scroll"
            } else if active_tab == Tab::Agents {
                " q:Quit  [ ]:Switch tab  j/k:Select  PgUp/PgDn:Scroll detail"
            } else if active_tab == Tab::IntroPrompt {
                " q:Quit  [ ]:Switch tab  g/p:Global/Project  e:Edit (in-TUI)  j/k:Scroll"
            } else if show_tree {
                " q:Quit  j/k:\u{2191}\u{2193}  /:Search  [ ]:Tab  Tab:Focus  Space:Expand  d:Dashboard  t:Theme  c:Category  x:Delete  r:Rename  Enter:Open"
            } else {
                " q:Quit  j/k:\u{2191}\u{2193}  /:Search  [ ]:Tab  d:Dashboard  t:Theme  c:Category  x:Delete  r:Rename  Enter:Open"
            };
            let footer_idx = if show_tree { vertical_chunks.len() - 1 } else { 2 };
            let footer = Paragraph::new(footer_text).style(theme::dim());
            f.render_widget(footer, vertical_chunks[footer_idx]);

            // ── Overlay rendering ───────────────────────────────────
            render_overlay(f, &overlay, &projects, area);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            let ev = event::read()?;

            // If overlay is active, route events to overlay handler
            if !matches!(overlay, Overlay::None) {
                match handle_overlay_event(
                    &ev,
                    &mut overlay,
                    &mut projects,
                    &mut sizes,
                    mgr,
                    home,
                    &mut selected,
                    mode_parsed,
                ) {
                    OverlayAction::Consumed => continue,
                    OverlayAction::Close => {
                        overlay = Overlay::None;
                        continue;
                    }
                    OverlayAction::PassThrough => {} // fall through to normal handling
                }
            }

            match ev {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                        KeyCode::Char(']') => active_tab = active_tab.next(),
                        KeyCode::Char('[') => active_tab = active_tab.prev(),
                        KeyCode::Char('1') => active_tab = Tab::Projects,
                        KeyCode::Char('2') => active_tab = Tab::Tokenizer,
                        KeyCode::Char('3') => active_tab = Tab::ClaudeMd,
                        KeyCode::Char('4') => active_tab = Tab::Rules,
                        KeyCode::Char('5') => active_tab = Tab::Agents,
                        KeyCode::Char('6') => active_tab = Tab::IntroPrompt,
                        KeyCode::Char('/') if active_tab == Tab::Projects => {
                            if let Some(folder) = run_project_search(terminal, home)? {
                                if let Some(idx) =
                                    projects.iter().position(|p| p.folder_name == folder)
                                {
                                    selected = virtual_count + idx;
                                    update_tree_for_selection(
                                        selected,
                                        virtual_count,
                                        &projects,
                                        home,
                                        &mut tree_state,
                                    );
                                }
                            }
                        }
                        KeyCode::Char('o') if active_tab == Tab::Tokenizer => {
                            spawn_tokenizer_optimize();
                        }
                        KeyCode::Char('T') if active_tab == Tab::Tokenizer => {
                            suspend_tui(terminal)?;
                            let _ = run_tokenizer_cmd(&["tui"]);
                            resume_tui(terminal)?;
                        }
                        KeyCode::Char('i') if active_tab == Tab::Tokenizer => {
                            suspend_tui(terminal)?;
                            let _ = run_tokenizer_cmd(&["install-timer"]);
                            resume_tui(terminal)?;
                        }
                        KeyCode::Char('h') if active_tab == Tab::Tokenizer => {
                            suspend_tui(terminal)?;
                            let _ = run_tokenizer_cmd(&["install-hook"]);
                            resume_tui(terminal)?;
                        }
                        // ── CLAUDE.md tab ──
                        KeyCode::Char('g') if active_tab == Tab::ClaudeMd => {
                            claude_md_global = true;
                            claude_scroll = 0;
                        }
                        KeyCode::Char('p') if active_tab == Tab::ClaudeMd => {
                            claude_md_global = false;
                            claude_scroll = 0;
                        }
                        KeyCode::Char('e') if active_tab == Tab::ClaudeMd => {
                            let path = if claude_md_global {
                                claude_md_global_path()
                            } else {
                                claude_md_project_path()
                            };
                            let title = if claude_md_global {
                                "CLAUDE.md (global ~/.claude)"
                            } else {
                                "CLAUDE.md (project ~/)"
                            };
                            run_editor(terminal, title, &path)?;
                            claude_scroll = 0;
                        }
                        KeyCode::Char('j') | KeyCode::Down if active_tab == Tab::ClaudeMd => {
                            claude_scroll = claude_scroll.saturating_add(1);
                        }
                        KeyCode::Char('k') | KeyCode::Up if active_tab == Tab::ClaudeMd => {
                            claude_scroll = claude_scroll.saturating_sub(1);
                        }
                        KeyCode::PageDown if active_tab == Tab::ClaudeMd => {
                            claude_scroll = claude_scroll.saturating_add(10);
                        }
                        KeyCode::PageUp if active_tab == Tab::ClaudeMd => {
                            claude_scroll = claude_scroll.saturating_sub(10);
                        }
                        // ── RULES tab ──
                        KeyCode::Char('e') if active_tab == Tab::Rules => {
                            if let Some(stem) = rule_stems.get(rule_sel).cloned() {
                                // Edit the persistent readable master in rules-md;
                                // on save, recompile it into the .toon Claude reads.
                                if let Some(master) = ensure_rule_master(&stem) {
                                    let before =
                                        std::fs::read_to_string(&master).unwrap_or_default();
                                    edit_file_external(terminal, &master)?;
                                    let after =
                                        std::fs::read_to_string(&master).unwrap_or_default();
                                    if after != before {
                                        rule_compile_status = Some(match compile_rule_master(&stem)
                                        {
                                            Ok(()) => format!("{stem}: saved + compiled to .toon"),
                                            Err(e) => format!(
                                                "{stem}: master saved; .toon NOT updated ({e})"
                                            ),
                                        });
                                    }
                                    rule_cache = None;
                                    rule_scroll = 0;
                                }
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down if active_tab == Tab::Rules => {
                            if !rule_stems.is_empty() {
                                rule_sel = (rule_sel + 1) % rule_stems.len();
                                rule_cache = None;
                                rule_scroll = 0;
                                rule_compile_status = None;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up if active_tab == Tab::Rules => {
                            if !rule_stems.is_empty() {
                                rule_sel =
                                    (rule_sel + rule_stems.len() - 1) % rule_stems.len();
                                rule_cache = None;
                                rule_scroll = 0;
                                rule_compile_status = None;
                            }
                        }
                        KeyCode::PageDown if active_tab == Tab::Rules => {
                            rule_scroll = rule_scroll.saturating_add(10);
                        }
                        KeyCode::PageUp if active_tab == Tab::Rules => {
                            rule_scroll = rule_scroll.saturating_sub(10);
                        }
                        // ── Agents tab ──
                        KeyCode::Char('j') | KeyCode::Down if active_tab == Tab::Agents => {
                            if !agents.is_empty() {
                                agent_sel = (agent_sel + 1) % agents.len();
                                agent_scroll = 0;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up if active_tab == Tab::Agents => {
                            if !agents.is_empty() {
                                agent_sel = (agent_sel + agents.len() - 1) % agents.len();
                                agent_scroll = 0;
                            }
                        }
                        KeyCode::PageDown if active_tab == Tab::Agents => {
                            agent_scroll = agent_scroll.saturating_add(10);
                        }
                        KeyCode::PageUp if active_tab == Tab::Agents => {
                            agent_scroll = agent_scroll.saturating_sub(10);
                        }
                        // ── Intro-prompt tab ──
                        KeyCode::Char('g') if active_tab == Tab::IntroPrompt => {
                            intro_global = true;
                            intro_scroll = 0;
                        }
                        KeyCode::Char('p') if active_tab == Tab::IntroPrompt => {
                            intro_global = false;
                            intro_scroll = 0;
                        }
                        KeyCode::Char('e') if active_tab == Tab::IntroPrompt => {
                            let path = if intro_global {
                                ensure_intro_global()
                            } else {
                                match selected
                                    .checked_sub(virtual_count)
                                    .and_then(|i| projects.get(i))
                                    .map(|p| home.join(&p.folder_name))
                                {
                                    Some(d) => intro_tmpl_project_path(&d),
                                    None => ensure_intro_global(),
                                }
                            };
                            let title = if intro_global {
                                "Intro prompt (global)".to_string()
                            } else {
                                "Intro prompt (project override)".to_string()
                            };
                            run_editor(terminal, &title, &path)?;
                            intro_scroll = 0;
                        }
                        KeyCode::Char('j') | KeyCode::Down if active_tab == Tab::IntroPrompt => {
                            intro_scroll = intro_scroll.saturating_add(1);
                        }
                        KeyCode::Char('k') | KeyCode::Up if active_tab == Tab::IntroPrompt => {
                            intro_scroll = intro_scroll.saturating_sub(1);
                        }
                        KeyCode::Tab => {
                            focus = focus.next();
                        }
                        KeyCode::Right => {
                            if focus == FocusPanel::Table {
                                focus = FocusPanel::Tree;
                            }
                        }
                        KeyCode::Left => {
                            if focus == FocusPanel::Tree {
                                focus = FocusPanel::Table;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => match focus {
                            FocusPanel::Table => {
                                if (virtual_count + projects.len()) > 0 {
                                    selected = (selected + 1) % (virtual_count + projects.len());
                                    update_tree_for_selection(
                                        selected,
                                        virtual_count,
                                        &projects,
                                        home,
                                        &mut tree_state,
                                    );
                                }
                            }
                            FocusPanel::Tree => {
                                let flat = tree_state
                                    .as_ref()
                                    .map(|ts| filetree::flatten(&ts.root))
                                    .unwrap_or_default();
                                if let Some(ref mut ts) = tree_state {
                                    ts.select_next(flat.len());
                                }
                            }
                            FocusPanel::Info => {}
                        },
                        KeyCode::Up | KeyCode::Char('k') => match focus {
                            FocusPanel::Table => {
                                if (virtual_count + projects.len()) > 0 {
                                    selected = (selected + (virtual_count + projects.len()) - 1)
                                        % (virtual_count + projects.len());
                                    update_tree_for_selection(
                                        selected,
                                        virtual_count,
                                        &projects,
                                        home,
                                        &mut tree_state,
                                    );
                                }
                            }
                            FocusPanel::Tree => {
                                let flat = tree_state
                                    .as_ref()
                                    .map(|ts| filetree::flatten(&ts.root))
                                    .unwrap_or_default();
                                if let Some(ref mut ts) = tree_state {
                                    ts.select_prev(flat.len());
                                }
                            }
                            FocusPanel::Info => {}
                        },
                        KeyCode::Char(' ') => {
                            if focus == FocusPanel::Tree {
                                let flat = tree_state
                                    .as_ref()
                                    .map(|ts| filetree::flatten(&ts.root))
                                    .unwrap_or_default();
                                if let Some(ref mut ts) = tree_state {
                                    ts.toggle_selected(&flat);
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            run_dashboard_ui(terminal)?;
                        }
                        KeyCode::Char('t') => {
                            if run_theme_picker(terminal)? {
                                // Theme changed, continue rendering with new colors
                            }
                        }
                        KeyCode::Char('x') | KeyCode::Delete if active_tab == Tab::Projects => {
                            if focus == FocusPanel::Table
                                && selected >= virtual_count
                                && selected - virtual_count < projects.len()
                            {
                                overlay = Overlay::DeleteConfirm {
                                    row: selected - virtual_count,
                                };
                            }
                        }
                        KeyCode::Char('r') if active_tab == Tab::Projects => {
                            if focus == FocusPanel::Table
                                && selected >= virtual_count
                                && selected - virtual_count < projects.len()
                            {
                                let ri = selected - virtual_count;
                                let current_name = projects[ri].display_name.clone();
                                let cursor = current_name.len();
                                overlay = Overlay::RenameInput {
                                    row: ri,
                                    input: current_name,
                                    cursor,
                                };
                            }
                        }
                        KeyCode::Char('s') if active_tab == Tab::Projects => {
                            // Status picker shortcut
                            if focus == FocusPanel::Table
                                && selected >= virtual_count
                                && selected - virtual_count < projects.len()
                            {
                                overlay = Overlay::StatusPicker {
                                    row: selected - virtual_count,
                                    selected: 0,
                                };
                            }
                        }
                        KeyCode::Char('c') if active_tab == Tab::Projects => {
                            // Category picker shortcut with auto-detection
                            if focus == FocusPanel::Table
                                && selected >= virtual_count
                                && selected - virtual_count < projects.len()
                            {
                                let ri = selected - virtual_count;
                                let project_dir = projects.get(ri)
                                    .map(|p| home.join(&p.folder_name));
                                let cats = collect_categories_with_detection(
                                    &projects,
                                    project_dir.as_deref(),
                                );
                                overlay = Overlay::CategoryPicker {
                                    row: ri,
                                    options: cats,
                                    selected: 0,
                                };
                            }
                        }
                        KeyCode::Enter if active_tab == Tab::Projects => {
                            if select_mode {
                                if selected == 0 {
                                    return Ok(Some(VIRTUAL_QUICK_SESSION.to_string()));
                                } else if selected == 1 {
                                    return Ok(Some(VIRTUAL_NEW_PROJECT.to_string()));
                                }
                            }
                            let real_idx = selected.checked_sub(virtual_count);
                            if let Some(p) = real_idx.and_then(|i| projects.get(i)) {
                                return Ok(Some(p.folder_name.clone()));
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse) if active_tab == Tab::Projects => {
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let pos = ratatui::layout::Position::new(mouse.column, mouse.row);
                            if table_area.contains(pos) {
                                focus = FocusPanel::Table;
                                // Header takes 2 rows (border + header row), plus block border
                                let table_row =
                                    (mouse.row as usize).saturating_sub(table_area.y as usize + 2);
                                let target = table_row;
                                if target < (virtual_count + projects.len()) {
                                    selected = target;
                                    update_tree_for_selection(
                                        selected,
                                        virtual_count,
                                        &projects,
                                        home,
                                        &mut tree_state,
                                    );

                                    // Check if click is on Status column (column offset ~36-46)
                                    // Widths: 2 + 20(min) + 14 + 10 + 10 + 8
                                    // Status column starts after fav(2) + name(~20) + category(14) = ~36
                                    let col_in_table = (mouse.column as usize)
                                        .saturating_sub(table_area.x as usize + 1);
                                    let inner_width = table_area.width.saturating_sub(2) as usize;
                                    let name_width =
                                        inner_width.saturating_sub(2 + 14 + 10 + 10 + 8);
                                    let status_col_start = 2 + name_width + 14;
                                    let status_col_end = status_col_start + 10;
                                    let cat_col_start = 2 + name_width;
                                    let cat_col_end = cat_col_start + 14;

                                    if selected >= virtual_count
                                        && selected - virtual_count < projects.len()
                                    {
                                        let ri = selected - virtual_count;
                                        if col_in_table >= status_col_start
                                            && col_in_table < status_col_end
                                        {
                                            overlay = Overlay::StatusPicker {
                                                row: ri,
                                                selected: 0,
                                            };
                                        } else if col_in_table >= cat_col_start
                                            && col_in_table < cat_col_end
                                        {
                                            let project_dir = projects.get(ri)
                                                .map(|p| home.join(&p.folder_name));
                                            let cats = collect_categories_with_detection(
                                                &projects,
                                                project_dir.as_deref(),
                                            );
                                            overlay = Overlay::CategoryPicker {
                                                row: ri,
                                                options: cats,
                                                selected: 0,
                                            };
                                        }
                                    }
                                }
                            } else if let Some(ta) = tree_area {
                                if ta.contains(pos) {
                                    focus = FocusPanel::Tree;
                                    let tree_row =
                                        (mouse.row as usize).saturating_sub(ta.y as usize + 1);
                                    if let Some(ref mut ts) = tree_state {
                                        let flat = filetree::flatten(&ts.root);
                                        if tree_row < flat.len() {
                                            ts.selected = tree_row;
                                        }
                                    }
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => match focus {
                            FocusPanel::Table => {
                                if (virtual_count + projects.len()) > 0 {
                                    selected = selected.saturating_sub(1);
                                    update_tree_for_selection(
                                        selected,
                                        virtual_count,
                                        &projects,
                                        home,
                                        &mut tree_state,
                                    );
                                }
                            }
                            FocusPanel::Tree => {
                                let flat = tree_state
                                    .as_ref()
                                    .map(|ts| filetree::flatten(&ts.root))
                                    .unwrap_or_default();
                                if let Some(ref mut ts) = tree_state {
                                    ts.select_prev(flat.len());
                                }
                            }
                            _ => {}
                        },
                        MouseEventKind::ScrollDown => match focus {
                            FocusPanel::Table => {
                                if (virtual_count + projects.len()) > 0 {
                                    selected =
                                        (selected + 1).min((virtual_count + projects.len()) - 1);
                                    update_tree_for_selection(
                                        selected,
                                        virtual_count,
                                        &projects,
                                        home,
                                        &mut tree_state,
                                    );
                                }
                            }
                            FocusPanel::Tree => {
                                let flat = tree_state
                                    .as_ref()
                                    .map(|ts| filetree::flatten(&ts.root))
                                    .unwrap_or_default();
                                if let Some(ref mut ts) = tree_state {
                                    ts.select_next(flat.len());
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn update_tree_for_selection(
    selected: usize,
    virtual_count: usize,
    projects: &[models::Project],
    home: &std::path::Path,
    tree_state: &mut Option<filetree::FileTreeState>,
) {
    let real_idx = selected.checked_sub(virtual_count);
    *tree_state = real_idx
        .and_then(|i| projects.get(i))
        .map(|p| home.join(&p.folder_name))
        .filter(|dir| dir.exists())
        .map(|dir| filetree::FileTreeState::new(&dir));
}

/// Detect project category by scanning directory contents for signature files.
fn detect_category(dir: &std::path::Path) -> Option<String> {
    if !dir.exists() {
        return None;
    }

    // Check for signature files (ordered by specificity)
    let signatures: &[(&[&str], &str)] = &[
        (&["Cargo.toml"], "Rust"),
        (&["go.mod"], "Go"),
        (&["Package.swift", "*.xcodeproj"], "Swift"),
        (&["pubspec.yaml"], "Flutter"),
        (&["mix.exs"], "Elixir"),
        (&["Gemfile", "*.gemspec"], "Ruby"),
        (&["pom.xml", "build.gradle", "build.gradle.kts"], "Java"),
        (&["*.csproj", "*.sln"], "C#/.NET"),
        (&["CMakeLists.txt", "Makefile.am"], "C/C++"),
        (&["pyproject.toml", "setup.py", "requirements.txt", "Pipfile"], "Python"),
        (&["package.json"], "Node"),
        (&["composer.json"], "PHP"),
        (&["Dockerfile", "docker-compose.yml", "docker-compose.yaml"], "DevOps"),
        (&["terraform.tf", "main.tf", "*.tfvars"], "Terraform"),
        (&["ansible.cfg", "playbook.yml"], "Ansible"),
        (&[".claude", "CLAUDE.md"], "Claude Code"),
        (&["Makefile"], "Build System"),
    ];

    let entries: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    for (patterns, category) in signatures {
        for pattern in *patterns {
            if let Some(suffix) = pattern.strip_prefix('*') {
                // Glob suffix match
                if entries.iter().any(|e| e.ends_with(suffix)) {
                    return Some(category.to_string());
                }
            } else if entries.iter().any(|e| e == *pattern) {
                return Some(category.to_string());
            }
        }
    }

    // Fallback: check for common project structure hints
    if entries.iter().any(|e| e == "src" || e == "lib") {
        return Some("Software".to_string());
    }
    if entries.iter().any(|e| e == "docs" || e == "content" || e.ends_with(".md")) {
        return Some("Documentation".to_string());
    }

    None
}

/// Build category options for the picker overlay, including auto-detected suggestion.
fn collect_categories_with_detection(
    projects: &[models::Project],
    project_dir: Option<&std::path::Path>,
) -> Vec<String> {
    let mut cats: Vec<String> = projects.iter().map(|p| p.category.clone()).collect();

    // Add standard categories that might not be in any project yet
    let standard = [
        "Research", "Software", "DevOps", "Documentation", "Experiment", "Archive",
    ];
    for s in standard {
        cats.push(s.to_string());
    }

    cats.sort();
    cats.dedup();

    // If we can detect a category, move it to the top
    if let Some(dir) = project_dir {
        if let Some(detected) = detect_category(dir) {
            // Remove it from its current position if present, and insert at top
            cats.retain(|c| c != &detected);
            cats.insert(0, format!("{detected} (detected)"));
        }
    }

    cats.push("Custom...".to_string());
    cats
}

// ── Overlay rendering ───────────────────────────────────────────────

fn render_overlay(
    f: &mut ratatui::Frame,
    overlay: &Overlay,
    projects: &[models::Project],
    area: ratatui::prelude::Rect,
) {
    use ratatui::{prelude::*, widgets::*};

    match overlay {
        Overlay::None => {}
        Overlay::StatusPicker { row, selected } => {
            let statuses = ["active", "paused", "archived"];
            let title = if let Some(p) = projects.get(*row) {
                format!(" Status: {} ", truncate_display(&p.display_name, 20))
            } else {
                " Status ".to_string()
            };
            let popup = centered_popup(area, 30, (statuses.len() + 2) as u16);
            render_shadow(f, popup);
            f.render_widget(Clear, popup);
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::accent()))
                .border_type(BorderType::Rounded);
            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let items: Vec<ListItem> = statuses
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let style = if i == *selected {
                        Style::default().bg(theme::accent()).fg(theme::bg())
                    } else {
                        theme::status_style(s)
                    };
                    let prefix = if i == *selected { "> " } else { "  " };
                    ListItem::new(format!("{prefix}{s}")).style(style)
                })
                .collect();
            f.render_widget(List::new(items), inner);
        }
        Overlay::CategoryPicker {
            row,
            options,
            selected,
        } => {
            let title = if let Some(p) = projects.get(*row) {
                format!(" Category: {} ", truncate_display(&p.display_name, 16))
            } else {
                " Category ".to_string()
            };
            let height = (options.len() + 2).min(15) as u16;
            let popup = centered_popup(area, 34, height);
            render_shadow(f, popup);
            f.render_widget(Clear, popup);
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::accent()))
                .border_type(BorderType::Rounded);
            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let items: Vec<ListItem> = options
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let style = if i == *selected {
                        Style::default().bg(theme::accent()).fg(theme::bg())
                    } else {
                        Style::default().fg(theme::fg())
                    };
                    let prefix = if i == *selected { "> " } else { "  " };
                    ListItem::new(format!("{prefix}{s}")).style(style)
                })
                .collect();
            f.render_widget(List::new(items), inner);
        }
        Overlay::TextInput {
            field,
            input,
            cursor,
            ..
        } => {
            let title = format!(" {} ", field);
            let popup = centered_popup(area, 40, 3);
            render_shadow(f, popup);
            f.render_widget(Clear, popup);
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::accent()))
                .border_type(BorderType::Rounded);
            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let display = format!("{}\u{2502}", input);
            let para = Paragraph::new(display).style(Style::default().fg(theme::fg()));
            f.render_widget(para, inner);
            // Show cursor position
            let cx = inner.x + (*cursor as u16).min(inner.width.saturating_sub(1));
            f.set_cursor_position(ratatui::layout::Position::new(cx, inner.y));
        }
        Overlay::DeleteConfirm { row } => {
            let name = projects
                .get(*row)
                .map(|p| p.display_name.as_str())
                .unwrap_or("?");
            let popup = centered_popup(area, 44, 5);
            render_shadow(f, popup);
            f.render_widget(Clear, popup);
            let block = Block::default()
                .title(" Confirm Delete ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(220, 60, 60)))
                .border_type(BorderType::Rounded);
            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let lines = vec![
                Line::from(format!(
                    "Remove \"{}\" from registry?",
                    truncate_display(name, 28)
                )),
                Line::from(""),
                Line::from(Span::styled("  y:Yes  n/Esc:Cancel", theme::dim())),
            ];
            f.render_widget(
                Paragraph::new(lines).style(Style::default().fg(theme::fg())),
                inner,
            );
        }
        Overlay::RenameInput { input, cursor, .. } => {
            let popup = centered_popup(area, 44, 3);
            render_shadow(f, popup);
            f.render_widget(Clear, popup);
            let block = Block::default()
                .title(" Rename Project ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::accent()))
                .border_type(BorderType::Rounded);
            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let para = Paragraph::new(input.as_str()).style(Style::default().fg(theme::fg()));
            f.render_widget(para, inner);
            let cx = inner.x + (*cursor as u16).min(inner.width.saturating_sub(1));
            f.set_cursor_position(ratatui::layout::Position::new(cx, inner.y));
        }
    }
}

fn centered_popup(area: ratatui::prelude::Rect, width: u16, height: u16) -> ratatui::prelude::Rect {
    let w = width.min(area.width.saturating_sub(4));
    let h = height.min(area.height.saturating_sub(4));
    ratatui::prelude::Rect::new(
        area.width.saturating_sub(w) / 2,
        area.height.saturating_sub(h) / 2,
        w,
        h,
    )
}

fn render_shadow(f: &mut ratatui::Frame, popup: ratatui::prelude::Rect) {
    use ratatui::{prelude::*, widgets::*};
    let shadow = Rect::new(
        popup
            .x
            .saturating_add(1)
            .min(f.area().width.saturating_sub(1)),
        popup
            .y
            .saturating_add(1)
            .min(f.area().height.saturating_sub(1)),
        popup
            .width
            .min(f.area().width.saturating_sub(popup.x.saturating_add(1))),
        popup
            .height
            .min(f.area().height.saturating_sub(popup.y.saturating_add(1))),
    );
    let shadow_block = Block::default().style(Style::default().bg(Color::Rgb(20, 20, 20)));
    f.render_widget(shadow_block, shadow);
}

// ── Overlay event handling ──────────────────────────────────────────

enum OverlayAction {
    Consumed,
    Close,
    PassThrough,
}

#[allow(clippy::too_many_arguments)]
fn handle_overlay_event(
    ev: &crossterm::event::Event,
    overlay: &mut Overlay,
    projects: &mut Vec<models::Project>,
    sizes: &mut Vec<u64>,
    mgr: &RegistryManager,
    home: &std::path::Path,
    table_selected: &mut usize,
    mode: ListMode,
) -> OverlayAction {
    use crossterm::event::{Event, KeyCode};

    let Event::Key(key) = ev else {
        return OverlayAction::Consumed;
    };

    match overlay {
        Overlay::None => OverlayAction::PassThrough,
        Overlay::StatusPicker { row, selected } => {
            let statuses = ["active", "paused", "archived"];
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => OverlayAction::Close,
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected = (*selected + 1) % statuses.len();
                    OverlayAction::Consumed
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.checked_sub(1).unwrap_or(statuses.len() - 1);
                    OverlayAction::Consumed
                }
                KeyCode::Enter => {
                    let row_val = *row;
                    let sel = *selected;
                    if let Some(p) = projects.get(row_val) {
                        let _ = mgr.set_field(&p.folder_name, "status", statuses[sel]);
                        reload_projects(projects, sizes, mgr, home, mode);
                    }
                    OverlayAction::Close
                }
                _ => OverlayAction::Consumed,
            }
        }
        Overlay::CategoryPicker {
            row,
            options,
            selected,
        } => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => OverlayAction::Close,
                KeyCode::Down | KeyCode::Char('j') => {
                    if !options.is_empty() {
                        *selected = (*selected + 1) % options.len();
                    }
                    OverlayAction::Consumed
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !options.is_empty() {
                        *selected = selected.checked_sub(1).unwrap_or(options.len() - 1);
                    }
                    OverlayAction::Consumed
                }
                KeyCode::Enter => {
                    let row_val = *row;
                    let sel = *selected;
                    let chosen = match options.get(sel) {
                        Some(c) => c.clone(),
                        None => return OverlayAction::Close,
                    };
                    if chosen == "Custom..." {
                        // Switch to text input overlay
                        *overlay = Overlay::TextInput {
                            row: row_val,
                            field: "Category".to_string(),
                            input: String::new(),
                            cursor: 0,
                        };
                        return OverlayAction::Consumed;
                    }
                    // Strip " (detected)" suffix if present
                    let category = chosen
                        .strip_suffix(" (detected)")
                        .unwrap_or(&chosen)
                        .to_string();
                    if let Some(p) = projects.get(row_val) {
                        let _ = mgr.set_field(&p.folder_name, "category", &category);
                        reload_projects(projects, sizes, mgr, home, mode);
                    }
                    OverlayAction::Close
                }
                _ => OverlayAction::Consumed,
            }
        }
        Overlay::TextInput {
            row,
            field,
            input,
            cursor,
        } => match key.code {
            KeyCode::Esc => OverlayAction::Close,
            KeyCode::Enter => {
                let row_val = *row;
                let field_name = field.to_lowercase();
                let value = input.clone();
                if let Some(p) = projects.get(row_val) {
                    let _ = mgr.set_field(&p.folder_name, &field_name, &value);
                    reload_projects(projects, sizes, mgr, home, mode);
                }
                OverlayAction::Close
            }
            KeyCode::Char(c) => {
                // cursor is a byte index; insert keeps it valid.
                input.insert(*cursor, c);
                *cursor += c.len_utf8();
                OverlayAction::Consumed
            }
            KeyCode::Backspace => {
                if *cursor > 0 {
                    // Step back by one char (not one byte).
                    let prev = input[..*cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    input.remove(prev);
                    *cursor = prev;
                }
                OverlayAction::Consumed
            }
            KeyCode::Left => {
                // Move cursor back by one char.
                *cursor = input[..*cursor]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                OverlayAction::Consumed
            }
            KeyCode::Right => {
                // Move cursor forward by one char.
                if *cursor < input.len() {
                    let next = input[*cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| *cursor + i)
                        .unwrap_or(input.len());
                    *cursor = next;
                }
                OverlayAction::Consumed
            }
            _ => OverlayAction::Consumed,
        },
        Overlay::DeleteConfirm { row } => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let row_val = *row;
                if let Some(p) = projects.get(row_val) {
                    let _ = mgr.remove(&p.folder_name);
                    reload_projects(projects, sizes, mgr, home, mode);
                    if *table_selected > 0 && *table_selected >= projects.len() {
                        *table_selected = table_selected.saturating_sub(1);
                    }
                }
                OverlayAction::Close
            }
            KeyCode::Char('n') | KeyCode::Esc => OverlayAction::Close,
            _ => OverlayAction::Consumed,
        },
        Overlay::RenameInput { row, input, cursor } => match key.code {
            KeyCode::Esc => OverlayAction::Close,
            KeyCode::Enter => {
                let row_val = *row;
                let value = input.clone();
                if !value.is_empty() {
                    if let Some(p) = projects.get(row_val) {
                        let _ = mgr.set_field(&p.folder_name, "display_name", &value);
                        reload_projects(projects, sizes, mgr, home, mode);
                    }
                }
                OverlayAction::Close
            }
            KeyCode::Char(c) => {
                // cursor is a byte index; insert keeps it valid.
                input.insert(*cursor, c);
                *cursor += c.len_utf8();
                OverlayAction::Consumed
            }
            KeyCode::Backspace => {
                if *cursor > 0 {
                    // Step back by one char (not one byte).
                    let prev = input[..*cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    input.remove(prev);
                    *cursor = prev;
                }
                OverlayAction::Consumed
            }
            KeyCode::Left => {
                // Move cursor back by one char.
                *cursor = input[..*cursor]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                OverlayAction::Consumed
            }
            KeyCode::Right => {
                // Move cursor forward by one char.
                if *cursor < input.len() {
                    let next = input[*cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| *cursor + i)
                        .unwrap_or(input.len());
                    *cursor = next;
                }
                OverlayAction::Consumed
            }
            _ => OverlayAction::Consumed,
        },
    }
}

fn reload_projects(
    projects: &mut Vec<models::Project>,
    sizes: &mut Vec<u64>,
    mgr: &RegistryManager,
    home: &std::path::Path,
    mode: ListMode,
) {
    if let Ok(new_projects) = mgr.list_sorted(mode) {
        let new_sizes = compute_sizes(&new_projects, home);
        *projects = new_projects;
        *sizes = new_sizes;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Dashboard Screen
// ═══════════════════════════════════════════════════════════════════════

fn run_dashboard_ui<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::{prelude::*, widgets::*};

    loop {
        // Load data
        let day_counts = sessions::aggregate_by_day().unwrap_or_default();
        let top = sessions::top_projects(5).unwrap_or_default();
        let weekday = sessions::activity_by_weekday().unwrap_or([0u64; 7]);
        let trend = sessions::last_n_days(30).unwrap_or_default();

        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // title
                    Constraint::Length(12), // calendar
                    Constraint::Min(8),     // bar charts
                    Constraint::Length(4),  // sparkline
                    Constraint::Length(1),  // footer
                ])
                .split(area);

            // Title
            let title = Paragraph::new(" Projectwise Usage Dashboard \u{2014} Last 52 Weeks")
                .style(theme::title())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(theme::border())
                        .border_type(BorderType::Rounded),
                );
            f.render_widget(title, chunks[0]);

            // Calendar heatmap
            {
                let today = chrono::Utc::now().date_naive();
                let start = today - chrono::Duration::weeks(52);

                let weeks = 52usize;
                let mut rows: Vec<String> = vec![String::new(); 7];
                for week_offset in 0..weeks {
                    let week_start = start + chrono::Duration::weeks(week_offset as i64);
                    for (day_offset, row) in rows.iter_mut().enumerate() {
                        let d = week_start + chrono::Duration::days(day_offset as i64);
                        let count = day_counts.get(&d).copied().unwrap_or(0);
                        let ch = match count {
                            0 => '\u{2591}',
                            1..=2 => '\u{2592}',
                            3..=5 => '\u{2593}',
                            _ => '\u{2588}',
                        };
                        row.push(ch);
                    }
                }
                let day_labels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                let lines: Vec<Line> = rows
                    .iter()
                    .enumerate()
                    .map(|(i, row)| Line::from(format!("{} {}", day_labels[i], row)))
                    .collect();

                let heatmap = Paragraph::new(lines)
                    .style(Style::default().fg(theme::accent()))
                    .block(
                        Block::default()
                            .title(" Activity Heatmap (52 weeks) ")
                            .borders(Borders::ALL)
                            .border_style(theme::border())
                            .border_type(BorderType::Rounded),
                    );
                f.render_widget(heatmap, chunks[1]);
            }

            // Bar charts row
            {
                let bar_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[2]);

                let proj_data: Vec<Bar> = top
                    .iter()
                    .map(|(name, count)| {
                        Bar::default()
                            .value(*count)
                            .label(Line::from(truncate_display(name, 14)))
                            .style(Style::default().fg(theme::accent()))
                    })
                    .collect();

                let proj_group = BarGroup::default().bars(&proj_data);
                let proj_chart = BarChart::default()
                    .block(
                        Block::default()
                            .title(" Top Projects ")
                            .borders(Borders::ALL)
                            .border_style(theme::border())
                            .border_type(BorderType::Rounded),
                    )
                    .data(proj_group)
                    .bar_width(3)
                    .bar_gap(1)
                    .value_style(Style::default().fg(theme::bg()).bg(theme::accent()))
                    .label_style(Style::default().fg(theme::dim_color()));
                f.render_widget(proj_chart, bar_chunks[0]);

                let day_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                let wd_bars: Vec<Bar> = weekday
                    .iter()
                    .enumerate()
                    .map(|(i, &count)| {
                        Bar::default()
                            .value(count)
                            .label(Line::from(day_names[i]))
                            .style(Style::default().fg(theme::accent()))
                    })
                    .collect();
                let wd_group = BarGroup::default().bars(&wd_bars);
                let wd_chart = BarChart::default()
                    .block(
                        Block::default()
                            .title(" Activity by Weekday ")
                            .borders(Borders::ALL)
                            .border_style(theme::border())
                            .border_type(BorderType::Rounded),
                    )
                    .data(wd_group)
                    .bar_width(3)
                    .bar_gap(1)
                    .value_style(Style::default().fg(theme::bg()).bg(theme::accent()))
                    .label_style(Style::default().fg(theme::dim_color()));
                f.render_widget(wd_chart, bar_chunks[1]);
            }

            // Sparkline 30-day trend
            {
                let spark = Sparkline::default()
                    .block(
                        Block::default()
                            .title(" 30-day trend ")
                            .borders(Borders::ALL)
                            .border_style(theme::border())
                            .border_type(BorderType::Rounded),
                    )
                    .data(&trend)
                    .style(Style::default().fg(theme::accent()));
                f.render_widget(spark, chunks[3]);
            }

            // Footer
            let footer = Paragraph::new(" q/Esc:Back  r:Refresh").style(theme::dim());
            f.render_widget(footer, chunks[4]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {} // just loop — data reloaded each draw
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Theme picker modal
// ═══════════════════════════════════════════════════════════════════════

/// Returns true if a theme was applied (caller should re-render)
fn run_theme_picker<W: std::io::Write>(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<W>>,
) -> Result<bool> {
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::{prelude::*, widgets::*};

    let themes = theme::available_themes();
    let current_theme = theme::current_theme_name();
    let mut selected = 0usize;
    let mut dark_only = true;

    // Build filtered list indices
    let filtered_indices = |dark_only: bool| -> Vec<usize> {
        themes
            .iter()
            .enumerate()
            .filter(|(_, t)| !dark_only || t.is_dark)
            .map(|(i, _)| i)
            .collect()
    };

    let mut indices = filtered_indices(dark_only);

    // Find current theme
    for (fi, &idx) in indices.iter().enumerate() {
        if themes[idx].name == current_theme {
            selected = fi;
            break;
        }
    }

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let popup_width = 52u16.min(area.width.saturating_sub(4));
            let popup_height = 20u16.min(area.height.saturating_sub(4));
            let popup_area = Rect::new(
                area.width.saturating_sub(popup_width) / 2,
                area.height.saturating_sub(popup_height) / 2,
                popup_width,
                popup_height,
            );

            // Dim background with Clear
            f.render_widget(Clear, popup_area);

            let block = Block::default()
                .title(format!(
                    " Select Theme {} ",
                    if dark_only { "(Dark)" } else { "(All)" }
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::accent()))
                .border_type(BorderType::Rounded);
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let mut lines = Vec::new();
            for (fi, &idx) in indices.iter().enumerate() {
                let entry = &themes[idx];
                let style = if fi == selected {
                    Style::default().bg(theme::accent()).fg(theme::bg())
                } else {
                    Style::default().fg(theme::fg())
                };
                let prefix = if fi == selected { "> " } else { "  " };
                let variant_tag = if entry.is_dark { "" } else { " [light]" };
                lines.push(Line::from(Span::styled(
                    format!("{}{}{}", prefix, entry.display_name, variant_tag),
                    style,
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " L:Toggle light  j/k:Nav  Enter:Apply  q:Cancel",
                theme::dim(),
            )));

            let para = Paragraph::new(lines);
            f.render_widget(para, inner);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                    KeyCode::Char('L') | KeyCode::Char('l') => {
                        dark_only = !dark_only;
                        indices = filtered_indices(dark_only);
                        selected = 0;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !indices.is_empty() {
                            selected = (selected + 1) % indices.len();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if !indices.is_empty() {
                            selected = selected.checked_sub(1).unwrap_or(indices.len() - 1);
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(&idx) = indices.get(selected) {
                            let name = themes[idx].name.clone();
                            let _ = theme::reload_theme(&name);
                            return Ok(true);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Preview — Styled terminal output (used by FZF --preview subprocess)
// ═══════════════════════════════════════════════════════════════════════

fn cmd_preview(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    use colored::*;
    if folder.starts_with("__") {
        return Ok(());
    }
    let Some(p) = mgr.get(folder)? else {
        eprintln!("not found: {folder}");
        return Ok(());
    };

    let fav = if p.favorite { " \u{2605}" } else { "" };
    let status_colored = match p.status.to_string().as_str() {
        "active" => "active".green().to_string(),
        "paused" => "paused".yellow().to_string(),
        "archived" => "archived".dimmed().to_string(),
        s => s.to_string(),
    };

    println!("{}{}", p.display_name.bold().cyan(), fav.yellow());
    println!("{}", "\u{2500}".repeat(40).dimmed());
    if p.description != "Project" && p.description != "\u{2014}" {
        println!("{}", p.description.dimmed());
        println!();
    }
    println!("  {:<14} {}", "Category".cyan(), p.category);
    println!("  {:<14} {}", "Status".cyan(), status_colored);
    if !p.tags.is_empty() {
        println!("  {:<14} {}", "Tags".cyan(), p.tags.join(", ").magenta());
    }
    println!(
        "  {:<14} {}",
        "Created".cyan(),
        p.created.format("%Y-%m-%d")
    );
    println!(
        "  {:<14} {}",
        "Last Active".cyan(),
        relative_time(&p.last_accessed)
    );
    println!("  {:<14} {}", "Sessions".cyan(), p.session_count);

    let dir = home.join(&p.folder_name);
    if dir.exists() {
        let count = walkdir::WalkDir::new(&dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();
        let size = dir_size(&dir);
        println!(
            "  {:<14} {} files  ({})",
            "Directory".cyan(),
            count,
            format_size(size)
        );
    } else {
        println!("  {:<14} {}", "Directory".cyan(), "MISSING".red().bold());
    }

    if let Some(ref url) = p.git_link {
        println!("  {:<14} {}", "Git".cyan(), url.green());
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Info — JSON detail
// ═══════════════════════════════════════════════════════════════════════

fn cmd_info(mgr: &RegistryManager, folder: &str) -> Result<()> {
    let project = mgr
        .get(folder)?
        .with_context(|| format!("project not found: {folder}"))?;
    println!("{}", serde_json::to_string_pretty(&project)?);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Edit — Interactive metadata editor
// ═══════════════════════════════════════════════════════════════════════

fn cmd_edit(mgr: &RegistryManager, folder: &str) -> Result<()> {
    let project = mgr
        .get(folder)?
        .with_context(|| format!("project not found: {folder}"))?;

    eprintln!("Editing: {}", project.display_name);
    eprintln!("(press Enter to keep current value)\n");

    let name: String = dialoguer::Input::new()
        .with_prompt("Display name")
        .default(project.display_name.clone())
        .interact_text()?;
    if name != project.display_name {
        mgr.set_field(folder, "display_name", &name)?;
    }

    let desc: String = dialoguer::Input::new()
        .with_prompt("Description")
        .default(project.description.clone())
        .interact_text()?;
    if desc != project.description {
        mgr.set_field(folder, "description", &desc)?;
    }

    let cat: String = dialoguer::Input::new()
        .with_prompt("Category")
        .default(project.category.clone())
        .interact_text()?;
    if cat != project.category {
        mgr.set_field(folder, "category", &cat)?;
    }

    let statuses = &["active", "paused", "archived"];
    let current_idx = statuses
        .iter()
        .position(|s| *s == project.status.to_string())
        .unwrap_or(0);
    let status_idx = dialoguer::Select::new()
        .with_prompt("Status")
        .items(statuses)
        .default(current_idx)
        .interact()?;
    let new_status = statuses[status_idx];
    if new_status != project.status.to_string() {
        mgr.set_field(folder, "status", new_status)?;
    }

    let current_tags = project.tags.join(", ");
    let tags_str: String = dialoguer::Input::new()
        .with_prompt("Tags (comma-separated)")
        .default(current_tags.clone())
        .allow_empty(true)
        .interact_text()?;
    if tags_str != current_tags {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        mgr.set_tags(folder, tags)?;
    }

    let current_git = project.git_link.clone().unwrap_or_default();
    let git: String = dialoguer::Input::new()
        .with_prompt("Git link")
        .default(current_git.clone())
        .allow_empty(true)
        .interact_text()?;
    if git != current_git {
        mgr.set_field(folder, "git_link", &git)?;
    }

    eprintln!("\n+ Updated: {name}");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Registry operations
// ═══════════════════════════════════════════════════════════════════════

fn cmd_registry(mgr: &RegistryManager, sub: RegistrySub) -> Result<()> {
    match sub {
        RegistrySub::Init => {
            mgr.init()?;
            eprintln!("+ Registry initialized");
        }
        RegistrySub::Add {
            folder,
            name,
            description,
            category,
        } => {
            validate_folder_name(&folder)?;
            let display = if name.is_empty() { &folder } else { &name };
            mgr.add(&folder, display, &description, &category)?;
            eprintln!("+ Added: {display}");
        }
        RegistrySub::Remove { folder } => {
            mgr.remove(&folder)?;
            eprintln!("+ Removed: {folder}");
        }
        RegistrySub::List => {
            for n in mgr.list_names()? {
                println!("{n}");
            }
        }
        RegistrySub::Get { folder } => {
            cmd_info(mgr, &folder)?;
        }
        RegistrySub::Touch { folder } => {
            mgr.touch(&folder)?;
        }
        RegistrySub::SetField {
            folder,
            field,
            value,
        } => {
            mgr.set_field(&folder, &field, &value)?;
        }
        RegistrySub::SetName { folder, name } => {
            mgr.set_field(&folder, "display_name", &name)?;
        }
        RegistrySub::SetStatus { folder, status } => {
            mgr.set_field(&folder, "status", &status)?;
        }
        RegistrySub::ToggleFav { folder } => {
            mgr.toggle_favorite(&folder)?;
        }
        RegistrySub::SetTags { folder, tags } => {
            let tags: Vec<String> = tags.split(',').map(|s| s.trim().to_string()).collect();
            mgr.set_tags(&folder, tags)?;
        }
        RegistrySub::AutoCategorize => {
            use colored::*;
            let home = get_home();
            let projects = mgr.list_sorted(ListMode::All)?;
            let mut updated = 0usize;
            for p in &projects {
                let dir = home.join(&p.folder_name);
                if let Some(detected) = detect_category(&dir) {
                    if p.category == "Research" || p.category == "Project" {
                        mgr.set_field(&p.folder_name, "category", &detected)?;
                        eprintln!(
                            "  {} {} -> {}",
                            "\u{2713}".green(),
                            truncate_display(&p.display_name, 30),
                            detected.cyan()
                        );
                        updated += 1;
                    } else {
                        eprintln!(
                            "  {} {} (kept: {}, detected: {})",
                            "\u{2014}".dimmed(),
                            truncate_display(&p.display_name, 30),
                            p.category.yellow(),
                            detected.dimmed()
                        );
                    }
                }
            }
            eprintln!(
                "\n{} Updated {updated}/{} projects",
                "\u{2713}".green(),
                projects.len()
            );
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Shell Init
// ═══════════════════════════════════════════════════════════════════════

fn cmd_shell_init() -> Result<()> {
    print!(
        r#"# Projectwise — shell integration
# Generated by cpm shell-init v3.8.0

# Ensure Tokenizer is installed (once) and fire a background boost so Claude
# starts with freshly-compressed .toon context. Fail-open: never blocks launch.
_projectwise_tokenizer() {{
  local tok="${{TOKENIZER_BIN:-$HOME/.cargo/bin/tokenizer}}"
  [[ -x "$tok" ]] || return 0
  case "$OSTYPE" in
    darwin*) [[ -f "$HOME/Library/LaunchAgents/com.tokenizer.plist" ]] || "$tok" install-timer >/dev/null 2>&1 ;;
    *)       [[ -f "$HOME/.config/systemd/user/tokenizer.timer" ]]      || "$tok" install-timer >/dev/null 2>&1 ;;
  esac
  [[ -f "$HOME/.claude/hooks/tokenizer-post-session.sh" ]] || "$tok" install-hook >/dev/null 2>&1
  ( "$tok" optimize --quiet >/dev/null 2>&1 & ) 2>/dev/null
}}

# Launch Claude with the session interview directive (always) plus the context
# digest as the initial prompt when present.
_projectwise_launch() {{
  local _dir="$1"; shift
  local _digest="$_dir/.projectwise/context-digest.md"
  # Intro prompt template: per-project override → global default → built-in.
  # Edit either via Projectwise's Intro tab (key 6).
  local _interview
  if [[ -s "$_dir/.projectwise/intro-prompt.tmpl" ]]; then
    _interview="$(cat "$_dir/.projectwise/intro-prompt.tmpl")"
  elif [[ -s "$HOME/.claude/.projectwise/intro-prompt.tmpl" ]]; then
    _interview="$(cat "$HOME/.claude/.projectwise/intro-prompt.tmpl")"
  else
    _interview="Interview me to find the real goal of this project. Bias toward small, compartmentalized specs. Make me verify key decisions explicitly so nothing is missed."
  fi
  if [[ -s "$_digest" ]]; then
    command claude --dangerously-skip-permissions "$_interview Then read the file $_digest -- it is the latest PROGRESS + ARCHITECTURE snapshot for this project; absorb it before interviewing me." "$@"
  else
    command claude --dangerously-skip-permissions "$_interview" "$@"
  fi
}}

projectwise() {{
  command -v claude &>/dev/null || {{ echo "Error: claude CLI not found" >&2; return 127; }}
  local _pd="${{CLAUDE_PROJECTS_DIR:-$HOME/.claude/projects}}"
  _projectwise_tokenizer
  ( cpm rules-sync >/dev/null 2>&1 & ) 2>/dev/null  # keep readable rule mirror fresh (idempotent)
  local _sel; _sel=$(cpm list --select) || return 1
  [[ -z "$_sel" ]] && return 1
  case "$_sel" in
    __QUICK_SESSION__) command claude --dangerously-skip-permissions "$@" ;;
    __NEW_PROJECT__)
      local _f; _f=$(cpm create) || return 1
      cd "$_pd/$_f" && cpm pre-launch "$_f" && _projectwise_launch "$_pd/$_f" "$@" ;;
    *) cd "$_pd/$_sel" && cpm pre-launch "$_sel" && _projectwise_launch "$_pd/$_sel" "$@" ;;
  esac
}}
clauded() {{ projectwise "$@"; }}
"#
    );
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// FZF internals
// ═══════════════════════════════════════════════════════════════════════

fn cmd_list_fzf(mgr: &RegistryManager, mode: &str) -> Result<()> {
    let mode: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    for p in mgr.list_sorted(mode)?.iter() {
        let fav = if p.favorite { "\u{2605} " } else { "  " };
        println!("{fav}{}\t{}", p.display_name, p.folder_name);
    }
    println!("  \u{2795} New Project\t__NEW_PROJECT__");
    println!("  \u{1f4ac} Quick Session\t__QUICK_SESSION__");
    Ok(())
}

fn cmd_prompt_input(label: &str) -> Result<()> {
    let input: String = dialoguer::Input::new().with_prompt(label).interact_text()?;
    print!("{input}");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Pre-launch hooks
// ═══════════════════════════════════════════════════════════════════════

fn cmd_pre_launch(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    use colored::*;
    validate_folder_name(folder)?;
    let dir = safe_join(home, folder)?;

    if !dir.exists() {
        eprintln!("{} Directory missing: {}", "!".yellow(), dir.display());
        return Ok(());
    }

    if cmd_exists("axon") {
        let d = dir.display().to_string();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("axon")
                .args(["analyze", &d])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        });
        eprintln!("{} axon analyze (background)", "\u{2713}".green());
    }

    if cmd_exists("tldr") {
        let d = dir.clone();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("tldr")
                .args(["warm", "."])
                .current_dir(&d)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        });
        eprintln!("{} tldr warm (background)", "\u{2713}".green());
    }

    if cmd_exists("claude-context") {
        let d = dir.clone();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("claude-context")
                .arg("index")
                .current_dir(&d)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        });
        eprintln!("{} claude-context index (background)", "\u{2713}".green());
    }

    let _ = mgr.touch(folder);
    // Log session for stats
    let _ = sessions::log_session(folder);

    // Build the context digest (PROGRESS + ARCHITECTURE) so Claude absorbs the
    // latest status on launch. The shell wrapper passes this file as Claude's
    // initial prompt. Non-interactive, fail-open.
    match build_context_digest(home, folder, &dir) {
        Some(path) => eprintln!(
            "{} context digest \u{2192} {}",
            "\u{2713}".green(),
            path.display()
        ),
        None => eprintln!(
            "{} no PROGRESS/ARCHITECTURE found \u{2014} skipping context digest",
            "i".blue()
        ),
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Create
// ═══════════════════════════════════════════════════════════════════════

fn cmd_create(mgr: &RegistryManager, home: &std::path::Path) -> Result<()> {
    let name: String = dialoguer::Input::new()
        .with_prompt("Project name")
        .interact_text()?;
    if name.is_empty() {
        anyhow::bail!("name required");
    }
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let folder = format!("{}_{}", sanitized, chrono::Utc::now().timestamp());
    validate_folder_name(&folder)?;
    let path = safe_join(home, &folder)?;
    std::fs::create_dir_all(path.join(".planning"))?;
    mgr.add(&folder, &name, "Project", "Research")?;
    eprintln!("+ Created: {name}");
    println!("{folder}");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Archive / Restore / Delete
// ═══════════════════════════════════════════════════════════════════════

fn cmd_archive(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    validate_folder_name(folder)?;
    if !dialoguer::Confirm::new()
        .with_prompt(format!("Archive '{folder}'?"))
        .default(false)
        .interact()?
    {
        return Ok(());
    }
    let archive_dir = std::env::var("CLAUDE_ARCHIVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".claude/archive")
        });
    std::fs::create_dir_all(&archive_dir)?;
    let src = safe_join(home, folder)?;
    if src.exists() {
        std::fs::rename(&src, archive_dir.join(folder))?;
    }
    mgr.set_field(folder, "status", "archived")?;
    eprintln!("+ Archived: {folder}");
    Ok(())
}

fn cmd_restore(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    validate_folder_name(folder)?;
    let archive_dir = std::env::var("CLAUDE_ARCHIVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".claude/archive")
        });
    let src = safe_join(&archive_dir, folder)?;
    if !src.exists() {
        anyhow::bail!("archive not found: {}", src.display());
    }
    let dest = safe_join(home, folder)?;
    std::fs::rename(&src, &dest)?;
    mgr.set_field(folder, "status", "active")?;
    eprintln!("+ Restored: {folder}");
    Ok(())
}

fn cmd_delete(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    validate_folder_name(folder)?;
    if !dialoguer::Confirm::new()
        .with_prompt(format!("PERMANENTLY delete '{folder}' from registry?"))
        .default(false)
        .interact()?
    {
        return Ok(());
    }
    mgr.remove(folder)?;
    let path = safe_join(home, folder)?;
    if path.exists() {
        if dialoguer::Confirm::new()
            .with_prompt(format!("Also delete directory {}?", path.display()))
            .default(false)
            .interact()?
        {
            std::fs::remove_dir_all(&path)?;
            eprintln!("+ Directory removed");
        } else {
            eprintln!("i Directory preserved");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Integrity
// ═══════════════════════════════════════════════════════════════════════

fn cmd_integrity(mgr: &RegistryManager, home: &std::path::Path, sub: IntegritySub) -> Result<()> {
    use colored::*;
    let reg = mgr.load()?;

    let mut missing = Vec::new();
    for p in &reg.projects {
        if !home.join(&p.folder_name).exists() {
            missing.push(p.folder_name.clone());
        }
    }

    let known: std::collections::HashSet<String> =
        reg.projects.iter().map(|p| p.folder_name.clone()).collect();
    let mut untracked = Vec::new();
    if home.exists() {
        for entry in std::fs::read_dir(home)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') && !known.contains(&name) {
                    untracked.push(name);
                }
            }
        }
    }

    match sub {
        IntegritySub::Check => {
            if missing.is_empty() && untracked.is_empty() {
                println!("{}", "\u{2713} Registry and filesystem in sync".green());
            } else {
                for m in &missing {
                    println!("{} MISSING (in registry, no directory): {m}", "!".yellow());
                }
                for u in &untracked {
                    println!(
                        "{} UNTRACKED (directory exists, not in registry): {u}",
                        "?".blue()
                    );
                }
            }
        }
        IntegritySub::Repair => {
            for m in &missing {
                mgr.set_field(m, "status", "archived")?;
                eprintln!("+ Marked as archived: {m}");
            }
            for u in &untracked {
                if validate_folder_name(u).is_err() {
                    eprintln!("! Skipping unsafe directory name: {u}");
                    continue;
                }
                mgr.add(u, u, "Auto-discovered project", "Research")?;
                eprintln!("+ Added to registry: {u}");
            }
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Cleanup — prune + report
// ═══════════════════════════════════════════════════════════════════════

fn cmd_cleanup(home: &std::path::Path, sub: CleanupSub) -> Result<()> {
    use colored::*;
    match sub {
        CleanupSub::Prune { days } => {
            let threshold = chrono::Utc::now() - chrono::Duration::days(days as i64);
            let mut removed = 0usize;
            let cache_dirs = [".axon", ".tldr", ".claude-context"];

            if home.exists() {
                for entry in std::fs::read_dir(home)? {
                    let entry = entry?;
                    if !entry.file_type()?.is_dir() {
                        continue;
                    }
                    let project_dir = entry.path();
                    for cache in &cache_dirs {
                        let cache_path = project_dir.join(cache);
                        if !cache_path.exists() {
                            continue;
                        }
                        let modified = cache_path.metadata()?.modified()?;
                        let modified_utc: chrono::DateTime<chrono::Utc> = modified.into();
                        if modified_utc < threshold {
                            std::fs::remove_dir_all(&cache_path)?;
                            eprintln!(
                                "  {} {}/{cache}",
                                "\u{2717}".red(),
                                project_dir
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                            );
                            removed += 1;
                        }
                    }
                }
            }

            let backup_dir = home.join(".backups");
            if backup_dir.exists() {
                for entry in std::fs::read_dir(&backup_dir)? {
                    let entry = entry?;
                    let modified = entry.metadata()?.modified()?;
                    let modified_utc: chrono::DateTime<chrono::Utc> = modified.into();
                    if modified_utc < threshold {
                        let _ = std::fs::remove_file(entry.path());
                        removed += 1;
                    }
                }
            }

            eprintln!(
                "{} Pruned {removed} stale items (older than {days} days)",
                "\u{2713}".green()
            );
        }
        CleanupSub::Report => {
            println!("{}", "Projectwise \u{2014} Size Report".cyan());
            println!("{}", "\u{2500}".repeat(60));
            println!("{:<40} {:>10} {:>8}", "Project", "Size", "Files");
            println!("{}", "\u{2500}".repeat(60));

            let mut total_size = 0u64;
            let mut total_files = 0usize;

            if home.exists() {
                let mut entries: Vec<_> = std::fs::read_dir(home)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .collect();
                entries.sort_by_key(|e| e.file_name());

                for entry in entries {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    let size = dir_size(&path);
                    let files = walkdir::WalkDir::new(&path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                        .count();
                    total_size += size;
                    total_files += files;
                    let display_name = truncate_display(&name, 38);
                    println!(
                        "{:<40} {:>10} {:>8}",
                        display_name,
                        format_size(size),
                        files
                    );
                }
            }

            println!("{}", "\u{2500}".repeat(60));
            println!(
                "{:<40} {:>10} {:>8}",
                "TOTAL".bold(),
                format_size(total_size).bold(),
                total_files.to_string().bold()
            );
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════

fn cmd_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn relative_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let secs = chrono::Utc::now().signed_duration_since(*dt).num_seconds();
    if secs < 60 {
        return "just now".to_string();
    }
    if secs < 3600 {
        return format!("{}m ago", secs / 60);
    }
    if secs < 86400 {
        return format!("{}h ago", secs / 3600);
    }
    if secs < 604800 {
        return format!("{}d ago", secs / 86400);
    }
    dt.format("%b %d").to_string()
}

fn dir_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    if bytes < 1024 * 1024 * 1024 {
        return format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0));
    }
    format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

#[cfg(test)]
mod tab_tests {
    use super::*;

    #[test]
    fn titlecase_basics() {
        assert_eq!(titlecase("agent spawning"), "Agent Spawning");
        assert_eq!(titlecase("when_to_suggest"), "When_to_suggest");
        assert_eq!(titlecase(""), "");
    }

    #[test]
    fn toon_to_readable_headers_and_unescape() {
        let raw = "---\ndescription: a rule\nglobs: *\n---\n@type:rule\n>>section_one\n- item with code:\\nline2";
        let out = toon_to_readable(raw);
        assert!(out.contains("> a rule"), "frontmatter desc → blockquote: {out}");
        assert!(out.contains("## Section One"), "section header: {out}");
        assert!(!out.contains("@type:rule"), "drops @type noise: {out}");
        assert!(out.contains("line2") && out.contains("item with code:\nline2"),
            "unescapes \\n: {out:?}");
    }

    #[test]
    fn tab_cycle_next_prev() {
        assert_eq!(Tab::Projects.next(), Tab::Tokenizer);
        assert_eq!(Tab::Rules.next(), Tab::Agents);
        assert_eq!(Tab::Agents.next(), Tab::IntroPrompt);
        assert_eq!(Tab::IntroPrompt.next(), Tab::Projects);
        assert_eq!(Tab::Projects.prev(), Tab::IntroPrompt);
        assert_eq!(Tab::Tokenizer.prev(), Tab::Projects);
        assert_eq!(Tab::ClaudeMd.index(), 2);
        assert_eq!(Tab::Agents.index(), 4);
        assert_eq!(Tab::IntroPrompt.index(), 5);
    }

    #[test]
    fn strip_html_drops_scripts_and_tags() {
        let html = "<html><head><style>x{}</style></head><body><script>bad()</script><p>Hello <b>world</b></p></body></html>";
        let out = strip_html(html);
        assert!(out.contains("Hello") && out.contains("world"), "keeps text: {out}");
        assert!(!out.contains("bad()"), "drops script body: {out}");
        assert!(!out.contains('<'), "drops tags: {out}");
    }

    // Single test for env-dependent converter logic (avoids set_var races that
    // would occur if these ran as separate parallel tests).
    #[test]
    fn converter_bins_and_safety_guard() {
        // (1) env override wins for both resolvers.
        std::env::set_var("TOON_BIN", "/tmp/custom-toon");
        assert_eq!(toon_bin(), std::path::PathBuf::from("/tmp/custom-toon"));

        // (2) a missing converter errors AND preserves the existing .toon.
        std::env::set_var("MD_TO_JSON_BIN", "/nonexistent/md2json-xyz");
        assert_eq!(
            md_to_json_bin(),
            std::path::PathBuf::from("/nonexistent/md2json-xyz")
        );
        let dir = std::env::temp_dir().join("cpm_md_to_toon_test");
        let _ = std::fs::create_dir_all(&dir);
        let md = dir.join("r.md");
        let toon = dir.join("r.toon");
        std::fs::write(&md, "# hi\n").unwrap();
        std::fs::write(&toon, "OLD_TOON").unwrap();
        let res = md_to_toon(&md, &toon);
        assert!(res.is_err(), "missing converter must error");
        assert_eq!(
            std::fs::read_to_string(&toon).unwrap(),
            "OLD_TOON",
            "previous .toon must be preserved on failure"
        );

        std::env::remove_var("MD_TO_JSON_BIN");
        std::env::remove_var("TOON_BIN");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_folder_name_accepts_valid() {
        assert!(validate_folder_name("hello-world").is_ok());
        assert!(validate_folder_name("Foo_Bar.123").is_ok());
        assert!(validate_folder_name("a").is_ok());
    }

    #[test]
    fn validate_folder_name_rejects_invalid() {
        assert!(validate_folder_name("").is_err());
        assert!(validate_folder_name(".").is_err());
        assert!(validate_folder_name("..").is_err());
        assert!(validate_folder_name("../evil").is_err());
        assert!(validate_folder_name("hello world").is_err()); // space
        assert!(validate_folder_name("a/b").is_err());         // slash
        assert!(validate_folder_name("a\0b").is_err());        // null byte
    }

    #[test]
    fn safe_join_prevents_traversal() {
        let tmp = std::env::temp_dir();
        // A name that passes validate but could theoretically escape — validate_folder_name
        // already rejects ".." and slashes, so safe_join is a second layer check.
        assert!(safe_join(&tmp, "legit-project").is_ok());
        // validate_folder_name rejects ".." before safe_join can even check.
        assert!(safe_join(&tmp, "..").is_err());
    }

    #[test]
    fn strip_html_with_multibyte_chars() {
        // Non-ASCII text outside of tags must survive strip_html without panic.
        let html = "<p>héllo</p><b>wörld</b>";
        let out = strip_html(html);
        assert!(out.contains("héllo"), "multibyte text preserved: {out}");
        assert!(out.contains("wörld"), "multibyte text preserved: {out}");
        assert!(!out.contains('<'), "tags removed: {out}");
    }

    #[test]
    fn strip_html_empty_and_no_tags() {
        assert_eq!(strip_html(""), "");
        assert_eq!(strip_html("plain text"), "plain text");
    }

    // Verify cursor byte-offset arithmetic for multibyte char input.
    // We simulate what the TextInput overlay key handler does.
    #[test]
    fn text_input_cursor_multibyte() {
        let mut input = String::new();
        let mut cursor: usize = 0;

        // Type 'é' (2 bytes in UTF-8)
        let c = 'é';
        input.insert(cursor, c);
        cursor += c.len_utf8();
        assert_eq!(cursor, 2);
        assert_eq!(input, "é");

        // Type 'a'
        let c = 'a';
        input.insert(cursor, c);
        cursor += c.len_utf8();
        assert_eq!(cursor, 3);
        assert_eq!(input, "éa");

        // Backspace (remove 'a', cursor back to 2)
        {
            let prev = input[..cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            input.remove(prev);
            cursor = prev;
        }
        assert_eq!(cursor, 2);
        assert_eq!(input, "é");

        // Left arrow (move back over 'é', cursor → 0)
        cursor = input[..cursor]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        assert_eq!(cursor, 0);

        // Right arrow (move forward over 'é', cursor → 2)
        {
            let next = input[cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| cursor + i)
                .unwrap_or(input.len());
            cursor = next;
        }
        assert_eq!(cursor, 2);
    }
}
