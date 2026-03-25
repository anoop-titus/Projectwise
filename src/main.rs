mod models;
mod registry;
mod theme;
mod sessions;
mod filetree;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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
    if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-') {
        anyhow::bail!(
            "folder name contains invalid characters (only [a-zA-Z0-9._-] allowed)"
        );
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
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/projects"))
}

#[derive(Parser)]
#[command(name = "cpm", version = "3.2.0", about = "Claude Project Manager")]
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
        Some(Commands::List { mode }) => cmd_list(&mgr, &home, &mode),
        Some(Commands::Preview { folder }) => cmd_preview(&mgr, &home, &folder),
        Some(Commands::Info { folder }) => cmd_info(&mgr, &folder),
        Some(Commands::Registry { sub }) => cmd_registry(&mgr, sub),
        Some(Commands::ShellInit) => cmd_shell_init(),
        Some(Commands::Version) => { println!("Claude Project Manager v3.2.0"); Ok(()) }
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

    if !output.status.success() { std::process::exit(1); }

    let selected = String::from_utf8_lossy(&output.stdout);
    let folder = selected.trim().split('\t').nth(1).unwrap_or("").trim();
    if !folder.is_empty() { println!("{folder}"); }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// List — Full Ratatui table (3-panel layout)
// ═══════════════════════════════════════════════════════════════════════

fn cmd_list(mgr: &RegistryManager, home: &std::path::Path, mode: &str) -> Result<()> {
    use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode}};
    use ratatui::prelude::*;
    use std::io;

    let mode_parsed: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    let projects = mgr.list_sorted(mode_parsed)?;

    // Compute sizes once at startup
    let sizes = compute_sizes(&projects, home);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_list_ui(&mut terminal, &projects, &sizes, home, mode);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

/// Compute sizes for all projects in parallel with home dir
pub fn compute_sizes(projects: &[models::Project], base: &std::path::Path) -> Vec<u64> {
    projects.iter().map(|p| {
        let dir = base.join(&p.folder_name);
        if dir.exists() { dir_size(&dir) } else { 0 }
    }).collect()
}

/// Format bytes as human-readable string
pub fn human_size(bytes: u64) -> String {
    format_size(bytes)
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

fn run_list_ui(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    projects: &[models::Project],
    sizes: &[u64],
    home: &std::path::Path,
    mode: &str,
) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::{prelude::*, widgets::*};

    let mut selected = 0usize;
    let mut focus = FocusPanel::Table;

    // Build file tree for initially selected project
    let mut tree_state = if let Some(p) = projects.first() {
        let dir = home.join(&p.folder_name);
        if dir.exists() {
            Some(filetree::FileTreeState::new(&dir))
        } else {
            None
        }
    } else {
        None
    };

    loop {
        // Rebuild flat list each frame (cheap for small trees)
        let flat = tree_state.as_ref().map(|ts| filetree::flatten(&ts.root)).unwrap_or_default();

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

            // Title bar
            let title = Paragraph::new(format!(
                " Claude Project Manager \u{2500} {} ({} projects)",
                mode, projects.len()
            ))
            .style(theme::title())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border())
                .border_type(BorderType::Rounded));
            f.render_widget(title, vertical_chunks[0]);

            // Table (panel index 1)
            let table_border_style = if focus == FocusPanel::Table {
                Style::default().fg(theme::ACCENT)
            } else {
                theme::border()
            };

            let header_cells = ["", "Name", "Category", "Status", "Sessions", "Size"]
                .iter()
                .map(|h| Cell::from(*h).style(theme::header()));
            let header = Row::new(header_cells).height(1);

            let rows: Vec<Row> = projects.iter().enumerate().map(|(i, p)| {
                let fav = if p.favorite { "\u{2605}" } else { " " };
                let name = truncate_display(&p.display_name, 32);
                let status_str = p.status.to_string();
                let size_str = if i < sizes.len() { human_size(sizes[i]) } else { "—".to_string() };

                let base = if i % 2 == 0 { theme::row_normal() } else { theme::row_alt() };

                Row::new(vec![
                    Cell::from(fav).style(theme::favorite()),
                    Cell::from(name).style(base),
                    Cell::from(p.category.clone()).style(base),
                    Cell::from(status_str.clone()).style(theme::status_style(&status_str)),
                    Cell::from(format!("{:>4}", p.session_count)).style(base),
                    Cell::from(size_str).style(theme::dim()),
                ])
            }).collect();

            let widths = [
                Constraint::Length(2),
                Constraint::Min(20),
                Constraint::Length(14),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(8),
            ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(table_border_style)
                    .border_type(BorderType::Rounded)
                    .title(" Projects ")
                    .title_style(theme::title()))
                .row_highlight_style(Style::default().bg(theme::ACCENT).fg(theme::BG))
                .highlight_symbol(" \u{25b6} ");

            let mut table_widget_state = TableState::default();
            if !projects.is_empty() { table_widget_state.select(Some(selected)); }
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
                    let border_style = if focus == FocusPanel::Tree {
                        Style::default().fg(theme::ACCENT)
                    } else {
                        theme::border()
                    };
                    let block = Block::default()
                        .title(" Directory Tree ")
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .border_type(BorderType::Rounded);
                    f.render_widget(block, bottom_chunks[0]);
                }

                // Info panel
                if show_info {
                    let info_border_style = if focus == FocusPanel::Info {
                        Style::default().fg(theme::ACCENT)
                    } else {
                        theme::border()
                    };

                    let info_text = if let Some(p) = projects.get(selected) {
                        let size_str = if selected < sizes.len() { human_size(sizes[selected]) } else { "—".to_string() };
                        let tags_str = if p.tags.is_empty() { "—".to_string() } else { p.tags.join(", ") };
                        let git_str = p.git_link.as_deref().unwrap_or("—");
                        let last_str = relative_time(&p.last_accessed);
                        vec![
                            Line::from(vec![
                                Span::styled("Desc:     ", theme::dim()),
                                Span::styled(truncate_display(&p.description, 30), theme::row_normal()),
                            ]),
                            Line::from(vec![
                                Span::styled("Tags:     ", theme::dim()),
                                Span::styled(tags_str, Style::default().fg(theme::ACCENT)),
                            ]),
                            Line::from(vec![
                                Span::styled("Git:      ", theme::dim()),
                                Span::styled(truncate_display(git_str, 30), theme::dim()),
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
            }

            // Footer
            let footer_text = if show_tree {
                " q:Quit  j/k:\u{2191}\u{2193}  Tab:Focus  Space:Expand  d:Dashboard  t:Theme  Enter:Open"
            } else {
                " q:Quit  j/k:\u{2191}\u{2193}  d:Dashboard  t:Theme  Enter:Open"
            };
            let footer_idx = if show_tree { vertical_chunks.len() - 1 } else { 2 };
            let footer = Paragraph::new(footer_text).style(theme::dim());
            f.render_widget(footer, vertical_chunks[footer_idx]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Tab => {
                        focus = focus.next();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        match focus {
                            FocusPanel::Table => {
                                if !projects.is_empty() {
                                    selected = (selected + 1) % projects.len();
                                    // Update tree for new selection
                                    let dir = home.join(&projects[selected].folder_name);
                                    tree_state = if dir.exists() {
                                        Some(filetree::FileTreeState::new(&dir))
                                    } else {
                                        None
                                    };
                                }
                            }
                            FocusPanel::Tree => {
                                let flat = tree_state.as_ref().map(|ts| filetree::flatten(&ts.root)).unwrap_or_default();
                                if let Some(ref mut ts) = tree_state {
                                    ts.select_next(flat.len());
                                }
                            }
                            FocusPanel::Info => {}
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        match focus {
                            FocusPanel::Table => {
                                if !projects.is_empty() {
                                    selected = (selected + projects.len() - 1) % projects.len();
                                    let dir = home.join(&projects[selected].folder_name);
                                    tree_state = if dir.exists() {
                                        Some(filetree::FileTreeState::new(&dir))
                                    } else {
                                        None
                                    };
                                }
                            }
                            FocusPanel::Tree => {
                                let flat = tree_state.as_ref().map(|ts| filetree::flatten(&ts.root)).unwrap_or_default();
                                if let Some(ref mut ts) = tree_state {
                                    ts.select_prev(flat.len());
                                }
                            }
                            FocusPanel::Info => {}
                        }
                    }
                    KeyCode::Char(' ') => {
                        if focus == FocusPanel::Tree {
                            let flat = tree_state.as_ref().map(|ts| filetree::flatten(&ts.root)).unwrap_or_default();
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
                            // Theme changed — restart list UI with same state
                            return run_list_ui(terminal, projects, sizes, home, mode);
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(p) = projects.get(selected) {
                            let json = serde_json::to_string_pretty(&p)?;
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
// Dashboard Screen
// ═══════════════════════════════════════════════════════════════════════

fn run_dashboard_ui(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
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
                    Constraint::Length(3),   // title
                    Constraint::Length(12),  // calendar
                    Constraint::Min(8),      // bar charts
                    Constraint::Length(4),   // sparkline
                    Constraint::Length(1),   // footer
                ])
                .split(area);

            // Title
            let title = Paragraph::new(" Claude Usage Dashboard — Last 52 Weeks")
                .style(theme::title())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme::border())
                    .border_type(BorderType::Rounded));
            f.render_widget(title, chunks[0]);

            // Calendar heatmap — 52 weeks × 7 days block character grid
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
                            0 => '░',
                            1..=2 => '▒',
                            3..=5 => '▓',
                            _ => '█',
                        };
                        row.push(ch);
                    }
                }
                let day_labels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                let lines: Vec<Line> = rows.iter().enumerate().map(|(i, row)| {
                    Line::from(format!("{} {}", day_labels[i], row))
                }).collect();

                let heatmap = Paragraph::new(lines)
                    .style(Style::default().fg(theme::ACCENT))
                    .block(Block::default()
                        .title(" Activity Heatmap (52 weeks) ")
                        .borders(Borders::ALL)
                        .border_style(theme::border())
                        .border_type(BorderType::Rounded));
                f.render_widget(heatmap, chunks[1]);
            }

            // Bar charts row
            {
                let bar_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[2]);

                // Top projects bar chart
                let proj_data: Vec<Bar> = top.iter().map(|(name, count)| {
                    Bar::default()
                        .value(*count)
                        .label(Line::from(truncate_display(name, 14)))
                        .style(Style::default().fg(theme::ACCENT))
                }).collect();

                let proj_group = BarGroup::default().bars(&proj_data);
                let proj_chart = BarChart::default()
                    .block(Block::default()
                        .title(" Top Projects ")
                        .borders(Borders::ALL)
                        .border_style(theme::border())
                        .border_type(BorderType::Rounded))
                    .data(proj_group)
                    .bar_width(3)
                    .bar_gap(1)
                    .value_style(Style::default().fg(theme::BG).bg(theme::ACCENT))
                    .label_style(Style::default().fg(theme::DIM));
                f.render_widget(proj_chart, bar_chunks[0]);

                // Weekday bar chart
                let day_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                let wd_bars: Vec<Bar> = weekday.iter().enumerate().map(|(i, &count)| {
                    Bar::default()
                        .value(count)
                        .label(Line::from(day_names[i]))
                        .style(Style::default().fg(theme::ACCENT))
                }).collect();
                let wd_group = BarGroup::default().bars(&wd_bars);
                let wd_chart = BarChart::default()
                    .block(Block::default()
                        .title(" Activity by Weekday ")
                        .borders(Borders::ALL)
                        .border_style(theme::border())
                        .border_type(BorderType::Rounded))
                    .data(wd_group)
                    .bar_width(3)
                    .bar_gap(1)
                    .value_style(Style::default().fg(theme::BG).bg(theme::ACCENT))
                    .label_style(Style::default().fg(theme::DIM));
                f.render_widget(wd_chart, bar_chunks[1]);
            }

            // Sparkline 30-day trend
            {
                let spark = Sparkline::default()
                    .block(Block::default()
                        .title(" 30-day trend ")
                        .borders(Borders::ALL)
                        .border_style(theme::border())
                        .border_type(BorderType::Rounded))
                    .data(&trend)
                    .style(Style::default().fg(theme::ACCENT));
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
fn run_theme_picker(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> Result<bool> {
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::{prelude::*, widgets::*};

    let themes = theme::available_themes();
    let current_theme = theme::current_theme_name();
    let mut selected = 0usize;
    let mut dark_only = true;

    for (i, &(id, _)) in themes.iter().enumerate() {
        if id == current_theme {
            selected = i;
            break;
        }
    }

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let popup_width = 52u16.min(area.width.saturating_sub(4));
            let popup_height = 16u16.min(area.height.saturating_sub(4));
            let popup_area = Rect::new(
                area.width.saturating_sub(popup_width) / 2,
                area.height.saturating_sub(popup_height) / 2,
                popup_width,
                popup_height,
            );

            // Dim background with Clear
            f.render_widget(Clear, popup_area);

            let block = Block::default()
                .title(format!(" Select Theme {} ", if dark_only { "(Dark)" } else { "(All)" }))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::ACCENT))
                .border_type(BorderType::Rounded);
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let mut lines = Vec::new();
            for (i, &(id, name)) in themes.iter().enumerate() {
                if dark_only && !theme::is_theme_dark(id) {
                    continue;
                }
                let style = if i == selected {
                    Style::default().bg(theme::ACCENT).fg(theme::BG)
                } else {
                    Style::default().fg(theme::FG)
                };
                let prefix = if i == selected { "> " } else { "  " };
                lines.push(Line::from(Span::styled(
                    format!("{}{}", prefix, name),
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
                        selected = 0;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let next = (selected + 1..themes.len())
                            .find(|&i| !dark_only || theme::is_theme_dark(themes[i].0));
                        if let Some(n) = next {
                            selected = n;
                        } else {
                            // Wrap around
                            if let Some(n) = (0..themes.len()).find(|&i| !dark_only || theme::is_theme_dark(themes[i].0)) {
                                selected = n;
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let prev = (0..selected).rev()
                            .find(|&i| !dark_only || theme::is_theme_dark(themes[i].0));
                        if let Some(p) = prev {
                            selected = p;
                        } else {
                            // Wrap around
                            if let Some(p) = (0..themes.len()).rev().find(|&i| !dark_only || theme::is_theme_dark(themes[i].0)) {
                                selected = p;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        let selected_theme = themes[selected].0.to_string();
                        let _ = theme::set_theme(&selected_theme);
                        return Ok(true);
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
            validate_folder_name(&folder)?;
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
# Generated by cpm shell-init v3.2.0
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
    // Task 1: log session for stats
    let _ = sessions::log_session(folder);

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
    let sanitized: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' { c } else { '-' }).collect();
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
    if !dialoguer::Confirm::new().with_prompt(format!("Archive '{folder}'?")).default(false).interact()? { return Ok(()) }
    let archive_dir = std::env::var("CLAUDE_ARCHIVE_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/archive"));
    std::fs::create_dir_all(&archive_dir)?;
    let src = safe_join(home, folder)?;
    if src.exists() { std::fs::rename(&src, archive_dir.join(folder))?; }
    mgr.set_field(folder, "status", "archived")?;
    eprintln!("+ Archived: {folder}");
    Ok(())
}

fn cmd_restore(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    validate_folder_name(folder)?;
    let archive_dir = std::env::var("CLAUDE_ARCHIVE_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/archive"));
    let src = safe_join(&archive_dir, folder)?;
    if !src.exists() { anyhow::bail!("archive not found: {}", src.display()); }
    let dest = safe_join(home, folder)?;
    std::fs::rename(&src, &dest)?;
    mgr.set_field(folder, "status", "active")?;
    eprintln!("+ Restored: {folder}");
    Ok(())
}

fn cmd_delete(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    validate_folder_name(folder)?;
    if !dialoguer::Confirm::new().with_prompt(format!("PERMANENTLY delete '{folder}' from registry?")).default(false).interact()? { return Ok(()) }
    mgr.remove(folder)?;
    let path = safe_join(home, folder)?;
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
                    let display_name = truncate_display(&name, 38);
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
