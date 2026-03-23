mod models;
mod registry;
mod theme;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use models::ListMode;
use registry::RegistryManager;

fn get_home() -> PathBuf {
    std::env::var("CLAUDE_PROJECTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/projects"))
}

#[derive(Parser)]
#[command(name = "cpm", version = "3.1.0", about = "Claude Project Manager")]
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
    Remove { folder: String },
    List,
    Get { folder: String },
    Touch { folder: String },
    #[command(name = "set-name")]
    SetName { folder: String, name: String },
    #[command(name = "set-status")]
    SetStatus { folder: String, status: String },
    #[command(name = "set-field")]
    SetField { folder: String, field: String, value: String },
    #[command(name = "toggle-fav")]
    ToggleFav { folder: String },
    #[command(name = "set-tags")]
    SetTags { folder: String, tags: String },
}

#[derive(Subcommand)]
enum IntegritySub {
    Check,
    Repair,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let home = get_home();
    let mgr = RegistryManager::new(&home);

    match cli.command {
        None | Some(Commands::Select { .. }) => {
            let mode_str = match &cli.command {
                Some(Commands::Select { mode }) => mode.as_str(),
                _ => "quick",
            };
            cmd_select(&mgr, &home, mode_str)
        }
        Some(Commands::List { mode }) => cmd_list(&mgr, &mode),
        Some(Commands::Preview { folder }) => cmd_preview(&mgr, &home, &folder),
        Some(Commands::Info { folder }) => cmd_info(&mgr, &folder),
        Some(Commands::Registry { sub }) => cmd_registry(&mgr, sub),
        Some(Commands::ShellInit) => cmd_shell_init(),
        Some(Commands::Version) => { println!("Claude Project Manager v3.1.0"); Ok(()) }
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
    for p in &projects {
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

    if !output.status.success() { std::process::exit(1); }

    let selected = String::from_utf8_lossy(&output.stdout);
    let folder = selected.trim().split('\t').nth(1).unwrap_or("").trim();
    if !folder.is_empty() { println!("{folder}"); }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// List — Full Ratatui table
// ═══════════════════════════════════════════════════════════════════════

fn cmd_list(mgr: &RegistryManager, mode: &str) -> Result<()> {
    use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}};
    use ratatui::prelude::*;
    use std::io;

    let mode_parsed: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    let projects = mgr.list_sorted(mode_parsed)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_list_ui(&mut terminal, &projects, mode);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

fn run_list_ui(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    projects: &[models::Project],
    mode: &str,
) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::{prelude::*, widgets::*};

    let mut selected = 0usize;

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(1),
                ])
                .split(area);

            // Title bar
            let title = Paragraph::new(format!(" Claude Project Manager \u{2500} {} ({} projects)", mode, projects.len()))
                .style(theme::title())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme::border())
                    .border_type(BorderType::Rounded));
            f.render_widget(title, chunks[0]);

            // Table
            let header_cells = ["", "Name", "Category", "Status", "Sessions", "Last Active"]
                .iter()
                .map(|h| Cell::from(*h).style(theme::header()));
            let header = Row::new(header_cells).height(1);

            let rows: Vec<Row> = projects.iter().enumerate().map(|(i, p)| {
                let fav = if p.favorite { "\u{2605}" } else { " " };
                let name = if p.display_name.len() > 32 {
                    format!("{}...", &p.display_name[..29])
                } else {
                    p.display_name.clone()
                };
                let rel = relative_time(&p.last_accessed);
                let status_str = p.status.to_string();

                let base = if i % 2 == 0 { theme::row_normal() } else { theme::row_alt() };

                Row::new(vec![
                    Cell::from(fav).style(theme::favorite()),
                    Cell::from(name).style(base),
                    Cell::from(p.category.clone()).style(base),
                    Cell::from(status_str.clone()).style(theme::status_style(&status_str)),
                    Cell::from(format!("{:>4}", p.session_count)).style(base),
                    Cell::from(rel).style(theme::dim()),
                ])
            }).collect();

            let widths = [
                Constraint::Length(2),
                Constraint::Min(20),
                Constraint::Length(14),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(14),
            ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme::border())
                    .border_type(BorderType::Rounded)
                    .title(" Projects ")
                    .title_style(theme::title()))
                .row_highlight_style(Style::default().bg(theme::ACCENT).fg(theme::BG))
                .highlight_symbol(" \u{25b6} ");

            let mut state = TableState::default();
            if !projects.is_empty() { state.select(Some(selected)); }
            f.render_stateful_widget(table, chunks[1], &mut state);

            // Footer
            let footer = Paragraph::new(" q:Quit  j/k:\u{2191}\u{2193}  Enter:Info")
                .style(theme::dim());
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !projects.is_empty() { selected = (selected + 1) % projects.len(); }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if !projects.is_empty() { selected = (selected + projects.len() - 1) % projects.len(); }
                    }
                    KeyCode::Enter => {
                        if let Some(p) = projects.get(selected) {
                            let json = serde_json::to_string_pretty(&p)?;
                            // Will be printed after TUI cleanup in cmd_list
                            eprintln!("{json}");
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Preview — Styled terminal output (used by FZF --preview subprocess)
// ═══════════════════════════════════════════════════════════════════════

fn cmd_preview(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    use colored::*;
    if folder.starts_with("__") { return Ok(()); }
    let Some(p) = mgr.get(folder)? else { eprintln!("not found: {folder}"); return Ok(()) };

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
    println!("  {:<14} {}", "Created".cyan(), p.created.format("%Y-%m-%d"));
    println!("  {:<14} {}", "Last Active".cyan(), relative_time(&p.last_accessed));
    println!("  {:<14} {}", "Sessions".cyan(), p.session_count);

    let dir = home.join(&p.folder_name);
    if dir.exists() {
        let count = walkdir::WalkDir::new(&dir).max_depth(3)
            .into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).count();
        let size = dir_size(&dir);
        println!("  {:<14} {} files  ({})", "Directory".cyan(), count, format_size(size));
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
    let project = mgr.get(folder)?.with_context(|| format!("project not found: {folder}"))?;
    println!("{}", serde_json::to_string_pretty(&project)?);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Edit — Interactive metadata editor
// ═══════════════════════════════════════════════════════════════════════

fn cmd_edit(mgr: &RegistryManager, folder: &str) -> Result<()> {
    let project = mgr.get(folder)?.with_context(|| format!("project not found: {folder}"))?;

    eprintln!("Editing: {}", project.display_name);
    eprintln!("(press Enter to keep current value)\n");

    // Display name
    let name: String = dialoguer::Input::new()
        .with_prompt("Display name")
        .default(project.display_name.clone())
        .interact_text()?;
    if name != project.display_name {
        mgr.set_field(folder, "display_name", &name)?;
    }

    // Description
    let desc: String = dialoguer::Input::new()
        .with_prompt("Description")
        .default(project.description.clone())
        .interact_text()?;
    if desc != project.description {
        mgr.set_field(folder, "description", &desc)?;
    }

    // Category
    let cat: String = dialoguer::Input::new()
        .with_prompt("Category")
        .default(project.category.clone())
        .interact_text()?;
    if cat != project.category {
        mgr.set_field(folder, "category", &cat)?;
    }

    // Status
    let statuses = &["active", "paused", "archived"];
    let current_idx = statuses.iter().position(|s| *s == project.status.to_string()).unwrap_or(0);
    let status_idx = dialoguer::Select::new()
        .with_prompt("Status")
        .items(statuses)
        .default(current_idx)
        .interact()?;
    let new_status = statuses[status_idx];
    if new_status != project.status.to_string() {
        mgr.set_field(folder, "status", new_status)?;
    }

    // Tags
    let current_tags = project.tags.join(", ");
    let tags_str: String = dialoguer::Input::new()
        .with_prompt("Tags (comma-separated)")
        .default(current_tags.clone())
        .allow_empty(true)
        .interact_text()?;
    if tags_str != current_tags {
        let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        mgr.set_tags(folder, tags)?;
    }

    // Git link
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
        RegistrySub::Init => { mgr.init()?; eprintln!("+ Registry initialized"); }
        RegistrySub::Add { folder, name, description, category } => {
            let display = if name.is_empty() { &folder } else { &name };
            mgr.add(&folder, display, &description, &category)?;
            eprintln!("+ Added: {display}");
        }
        RegistrySub::Remove { folder } => { mgr.remove(&folder)?; eprintln!("+ Removed: {folder}"); }
        RegistrySub::List => { for n in mgr.list_names()? { println!("{n}"); } }
        RegistrySub::Get { folder } => { cmd_info(mgr, &folder)?; }
        RegistrySub::Touch { folder } => { mgr.touch(&folder)?; }
        RegistrySub::SetField { folder, field, value } => { mgr.set_field(&folder, &field, &value)?; }
        RegistrySub::SetName { folder, name } => { mgr.set_field(&folder, "display_name", &name)?; }
        RegistrySub::SetStatus { folder, status } => { mgr.set_field(&folder, "status", &status)?; }
        RegistrySub::ToggleFav { folder } => { mgr.toggle_favorite(&folder)?; }
        RegistrySub::SetTags { folder, tags } => {
            let tags: Vec<String> = tags.split(',').map(|s| s.trim().to_string()).collect();
            mgr.set_tags(&folder, tags)?;
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Shell Init
// ═══════════════════════════════════════════════════════════════════════

fn cmd_shell_init() -> Result<()> {
    print!(r#"# Claude Project Manager — shell integration
# Generated by cpm shell-init v3.1.0
claude() {{
  command -v claude &>/dev/null || {{ echo "Error: claude CLI not found" >&2; return 127; }}
  [[ "$*" =~ (--help|--version|-h|-v) ]] && {{ command claude "$@"; return $?; }}
  local _pd="${{CLAUDE_PROJECTS_DIR:-$HOME/.claude/projects}}"
  [[ "$(pwd)" == "$_pd/"* ]] && {{ command claude "$@"; return $?; }}
  (
    local _sel; _sel=$(cpm select) || return 1
    [[ -z "$_sel" ]] && return 1
    case "$_sel" in
      __QUICK_SESSION__) command claude "$@" ;;
      __NEW_PROJECT__)
        local _f; _f=$(cpm create) || return 1
        cd "$_pd/$_f" && cpm pre-launch "$_f" && command claude "$@" ;;
      *) cd "$_pd/$_sel" && cpm pre-launch "$_sel" && command claude "$@" ;;
    esac
  )
}}
"#);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// FZF internals
// ═══════════════════════════════════════════════════════════════════════

fn cmd_list_fzf(mgr: &RegistryManager, mode: &str) -> Result<()> {
    let mode: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    for p in mgr.list_sorted(mode)? {
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
    let dir = home.join(folder);

    if !dir.exists() {
        eprintln!("{} Directory missing: {}", "!".yellow(), dir.display());
        return Ok(());
    }

    // Refresh axon (background)
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

    // Refresh tldr (background)
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

    // Refresh claude-context (background)
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

    // Doc review
    let docs: Vec<&str> = ["PROJECT.md", "README.md", "PLAN.md", "PROGRESS.md"]
        .iter().filter(|f| dir.join(f).exists()).copied().collect();
    if !docs.is_empty() {
        let prompt = format!("Review docs? ({})", docs.join(", "));
        if dialoguer::Confirm::new().with_prompt(&prompt).default(false).interact()? {
            let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
            let paths: Vec<_> = docs.iter().map(|f| dir.join(f).display().to_string()).collect();
            let _ = std::process::Command::new(&pager).args(&paths).status();
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Create
// ═══════════════════════════════════════════════════════════════════════

fn cmd_create(mgr: &RegistryManager, home: &std::path::Path) -> Result<()> {
    let name: String = dialoguer::Input::new().with_prompt("Project name").interact_text()?;
    if name.is_empty() { anyhow::bail!("name required"); }
    let folder = format!("{}_{}", name, chrono::Utc::now().timestamp());
    let path = home.join(&folder);
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
    if !dialoguer::Confirm::new().with_prompt(format!("Archive '{folder}'?")).default(false).interact()? { return Ok(()) }
    let archive_dir = std::env::var("CLAUDE_ARCHIVE_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/archive"));
    std::fs::create_dir_all(&archive_dir)?;
    let src = home.join(folder);
    if src.exists() { std::fs::rename(&src, archive_dir.join(folder))?; }
    mgr.set_field(folder, "status", "archived")?;
    eprintln!("+ Archived: {folder}");
    Ok(())
}

fn cmd_restore(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    let archive_dir = std::env::var("CLAUDE_ARCHIVE_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/archive"));
    let src = archive_dir.join(folder);
    if !src.exists() { anyhow::bail!("archive not found: {}", src.display()); }
    std::fs::rename(&src, home.join(folder))?;
    mgr.set_field(folder, "status", "active")?;
    eprintln!("+ Restored: {folder}");
    Ok(())
}

fn cmd_delete(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    if !dialoguer::Confirm::new().with_prompt(format!("PERMANENTLY delete '{folder}' from registry?")).default(false).interact()? { return Ok(()) }
    mgr.remove(folder)?;
    let path = home.join(folder);
    if path.exists() {
        if dialoguer::Confirm::new().with_prompt(format!("Also delete directory {}?", path.display())).default(false).interact()? {
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

    let known: std::collections::HashSet<String> = reg.projects.iter().map(|p| p.folder_name.clone()).collect();
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
                for m in &missing { println!("{} MISSING (in registry, no directory): {m}", "!".yellow()); }
                for u in &untracked { println!("{} UNTRACKED (directory exists, not in registry): {u}", "?".blue()); }
            }
        }
        IntegritySub::Repair => {
            for m in &missing {
                mgr.set_field(m, "status", "archived")?;
                eprintln!("+ Marked as archived: {m}");
            }
            for u in &untracked {
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
                    if !entry.file_type()?.is_dir() { continue; }
                    let project_dir = entry.path();
                    for cache in &cache_dirs {
                        let cache_path = project_dir.join(cache);
                        if !cache_path.exists() { continue; }
                        let modified = cache_path.metadata()?.modified()?;
                        let modified_utc: chrono::DateTime<chrono::Utc> = modified.into();
                        if modified_utc < threshold {
                            std::fs::remove_dir_all(&cache_path)?;
                            eprintln!("  {} {}/{cache}", "\u{2717}".red(), project_dir.file_name().unwrap_or_default().to_string_lossy());
                            removed += 1;
                        }
                    }
                }
            }

            // Also prune registry backups
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

            eprintln!("{} Pruned {removed} stale items (older than {days} days)", "\u{2713}".green());
        }
        CleanupSub::Report => {
            println!("{}", "Claude Project Manager — Size Report".cyan());
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
                    let files = walkdir::WalkDir::new(&path).into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                        .count();
                    total_size += size;
                    total_files += files;

                    let display_name = if name.len() > 38 { format!("{}...", &name[..35]) } else { name };
                    println!("{:<40} {:>10} {:>8}", display_name, format_size(size), files);
                }
            }

            println!("{}", "\u{2500}".repeat(60));
            println!("{:<40} {:>10} {:>8}", "TOTAL".bold(), format_size(total_size).bold(), total_files.to_string().bold());
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════

fn cmd_exists(cmd: &str) -> bool {
    std::process::Command::new("which").arg(cmd)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false)
}

fn relative_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
    let secs = chrono::Utc::now().signed_duration_since(*dt).num_seconds();
    if secs < 60 { return "just now".to_string() }
    if secs < 3600 { return format!("{}m ago", secs / 60) }
    if secs < 86400 { return format!("{}h ago", secs / 3600) }
    if secs < 604800 { return format!("{}d ago", secs / 86400) }
    dt.format("%b %d").to_string()
}

fn dir_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path).into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { return format!("{bytes} B"); }
    if bytes < 1024 * 1024 { return format!("{:.1} KB", bytes as f64 / 1024.0); }
    if bytes < 1024 * 1024 * 1024 { return format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)); }
    format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}
