mod filetree;
mod models;
mod registry;
mod sessions;
mod theme;

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
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/projects"))
}

#[derive(Parser)]
#[command(
    name = "cpm",
    version = "3.3.0",
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
        Some(Commands::Version) => {
            println!("Projectwise v3.3.0");
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

    // Virtual rows: 2 items at top (Quick Session, New Project)
    let virtual_count: usize = if select_mode { 2 } else { 0 };

    let mut selected = 0usize;
    let mut focus = FocusPanel::Table;
    let mut overlay = Overlay::None;

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
                " Projectwise \u{2500} {} ({} projects)",
                mode, projects.len()
            ))
            .style(theme::title())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border())
                .border_type(BorderType::Rounded));
            f.render_widget(title, vertical_chunks[0]);

            // Table (panel index 1)
            table_area = vertical_chunks[1];
            let table_border_style = if focus == FocusPanel::Table {
                Style::default().fg(theme::accent())
            } else {
                theme::border()
            };

            let header_cells = ["", "Name", "Category", "Status", "Sessions", "Size"]
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
                ]));
            }

            for (i, p) in projects.iter().enumerate() {
                let row_idx = virtual_count + i;
                let fav = if p.favorite { "\u{2605}" } else { " " };
                let name = truncate_display(&p.display_name, 32);
                let status_str = p.status.to_string();
                let size_str = if i < sizes.len() { human_size(sizes[i]) } else { "\u{2014}".to_string() };

                let base = if row_idx.is_multiple_of(2) { theme::row_normal() } else { theme::row_alt() };

                rows.push(Row::new(vec![
                    Cell::from(fav).style(theme::favorite()),
                    Cell::from(name).style(base),
                    Cell::from(p.category.clone()).style(base),
                    Cell::from(status_str.clone()).style(theme::status_style(&status_str)),
                    Cell::from(format!("{:>4}", p.session_count)).style(base),
                    Cell::from(size_str).style(theme::dim()),
                ]));
            }

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

            // Footer
            let footer_text = if show_tree {
                " q:Quit  j/k:\u{2191}\u{2193}  Tab:Focus  Space:Expand  d:Dashboard  t:Theme  x:Delete  r:Rename  Enter:Open"
            } else {
                " q:Quit  j/k:\u{2191}\u{2193}  d:Dashboard  t:Theme  x:Delete  r:Rename  Enter:Open"
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
                        KeyCode::Char('x') | KeyCode::Delete => {
                            if focus == FocusPanel::Table
                                && selected >= virtual_count
                                && selected - virtual_count < projects.len()
                            {
                                overlay = Overlay::DeleteConfirm {
                                    row: selected - virtual_count,
                                };
                            }
                        }
                        KeyCode::Char('r') => {
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
                        KeyCode::Char('s') => {
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
                        KeyCode::Char('c') => {
                            // Category picker shortcut
                            if focus == FocusPanel::Table
                                && selected >= virtual_count
                                && selected - virtual_count < projects.len()
                            {
                                let cats = collect_categories(&projects);
                                overlay = Overlay::CategoryPicker {
                                    row: selected - virtual_count,
                                    options: cats,
                                    selected: 0,
                                };
                            }
                        }
                        KeyCode::Enter => {
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
                Event::Mouse(mouse) => {
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
                                            let cats = collect_categories(&projects);
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

fn collect_categories(projects: &[models::Project]) -> Vec<String> {
    let mut cats: Vec<String> = projects.iter().map(|p| p.category.clone()).collect();
    cats.sort();
    cats.dedup();
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
                    *selected = (*selected + 1) % options.len();
                    OverlayAction::Consumed
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    *selected = selected.checked_sub(1).unwrap_or(options.len() - 1);
                    OverlayAction::Consumed
                }
                KeyCode::Enter => {
                    let row_val = *row;
                    let sel = *selected;
                    let chosen = options[sel].clone();
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
                    if let Some(p) = projects.get(row_val) {
                        let _ = mgr.set_field(&p.folder_name, "category", &chosen);
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
                input.insert(*cursor, c);
                *cursor += 1;
                OverlayAction::Consumed
            }
            KeyCode::Backspace => {
                if *cursor > 0 {
                    *cursor -= 1;
                    input.remove(*cursor);
                }
                OverlayAction::Consumed
            }
            KeyCode::Left => {
                *cursor = cursor.saturating_sub(1);
                OverlayAction::Consumed
            }
            KeyCode::Right => {
                *cursor = (*cursor + 1).min(input.len());
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
                input.insert(*cursor, c);
                *cursor += 1;
                OverlayAction::Consumed
            }
            KeyCode::Backspace => {
                if *cursor > 0 {
                    *cursor -= 1;
                    input.remove(*cursor);
                }
                OverlayAction::Consumed
            }
            KeyCode::Left => {
                *cursor = cursor.saturating_sub(1);
                OverlayAction::Consumed
            }
            KeyCode::Right => {
                *cursor = (*cursor + 1).min(input.len());
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
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Shell Init
// ═══════════════════════════════════════════════════════════════════════

fn cmd_shell_init() -> Result<()> {
    print!(
        r#"# Projectwise — shell integration
# Generated by cpm shell-init v3.3.0
projectwise() {{
  command -v claude &>/dev/null || {{ echo "Error: claude CLI not found" >&2; return 127; }}
  local _pd="${{CLAUDE_PROJECTS_DIR:-$HOME/.claude/projects}}"
  local _sel; _sel=$(cpm list --select) || return 1
  [[ -z "$_sel" ]] && return 1
  case "$_sel" in
    __QUICK_SESSION__) command claude --dangerously-skip-permissions "$@" ;;
    __NEW_PROJECT__)
      local _f; _f=$(cpm create) || return 1
      cd "$_pd/$_f" && cpm pre-launch "$_f" && command claude --dangerously-skip-permissions "$@" ;;
    *) cd "$_pd/$_sel" && cpm pre-launch "$_sel" && command claude --dangerously-skip-permissions "$@" ;;
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

    let docs: Vec<&str> = ["PROJECT.md", "README.md", "PLAN.md", "PROGRESS.md"]
        .iter()
        .filter(|f| dir.join(f).exists())
        .copied()
        .collect();
    if !docs.is_empty() {
        let prompt = format!("Review docs? ({})", docs.join(", "));
        if dialoguer::Confirm::new()
            .with_prompt(&prompt)
            .default(false)
            .interact()?
        {
            let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".to_string());
            let paths: Vec<_> = docs
                .iter()
                .map(|f| dir.join(f).display().to_string())
                .collect();
            let _ = std::process::Command::new(&pager).args(&paths).status();
        }
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
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/archive"));
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
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".claude/archive"));
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
