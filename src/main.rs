mod models;
mod registry;

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
#[command(name = "cpm", version = "3.0.0", about = "Claude Project Manager")]
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
    /// List projects in a table
    List {
        #[arg(default_value = "quick")]
        mode: String,
    },
    /// Preview a project
    Preview { folder: String },
    /// Show detailed project info
    Info { folder: String },
    /// Create a new project
    Create,
    /// Edit project metadata
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
    Prune {
        #[arg(long, default_value = "30")]
        days: u32,
    },
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
        Some(Commands::Version) => { println!("Claude Project Manager v3.0.0"); Ok(()) }
        Some(Commands::ListFzf { mode }) => cmd_list_fzf(&mgr, &mode),
        Some(Commands::PromptInput { label }) => cmd_prompt_input(&label),
        Some(Commands::PreLaunch { folder }) => cmd_pre_launch(&mgr, &home, &folder),
        Some(Commands::Create) => cmd_create(&mgr, &home),
        Some(Commands::Archive { folder }) => cmd_archive(&mgr, &home, &folder),
        Some(Commands::Restore { folder }) => cmd_restore(&mgr, &home, &folder),
        Some(Commands::Delete { folder }) => cmd_delete(&mgr, &home, &folder),
        Some(Commands::Integrity { sub }) => cmd_integrity(&mgr, &home, sub),
        _ => { eprintln!("command not yet implemented"); Ok(()) }
    }
}

// --- Select ---

fn cmd_select(mgr: &RegistryManager, _home: &std::path::Path, mode: &str) -> Result<()> {
    let mode_parsed: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    let projects = mgr.list_sorted(mode_parsed)?;

    let mut lines = String::new();
    for p in &projects {
        lines.push_str(&format!("{}\t{}\n", p.display_name, p.folder_name));
    }
    lines.push_str("➕ New Project\t__NEW_PROJECT__\n");
    lines.push_str("💬 Quick Session (no project)\t__QUICK_SESSION__\n");

    let cpm = std::env::current_exe()?.display().to_string();

    let output = std::process::Command::new("fzf")
        .args([
            "--ansi", "--delimiter", "\t", "--with-nth", "1",
            "--header", "R:Rename  F:Favorite  Ctrl-D:Archive  Enter:Select",
            "--preview", &format!("{cpm} preview {{2}}"),
            "--preview-window", "right:50%:wrap",
            "--bind", &format!("f:execute-silent({cpm} registry toggle-fav {{2}})+reload({cpm} _list-fzf {mode})"),
            "--bind", &format!("ctrl-d:execute-silent({cpm} registry set-status {{2}} archived)+reload({cpm} _list-fzf {mode})"),
            "--bind", &format!("r:execute-silent({cpm} registry set-name {{2}} $({cpm} _prompt-input Name))+reload({cpm} _list-fzf {mode})"),
            "--exit-0",
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

// --- List ---

fn cmd_list(mgr: &RegistryManager, mode: &str) -> Result<()> {
    use colored::*;
    let mode: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    let projects = mgr.list_sorted(mode)?;

    println!("{}\n", "Claude Project Manager — Projects".cyan());
    println!("{:<3} {:<35} {:<12} {:<8} {:>8} {:>12}", "", "Name", "Category", "Status", "Sessions", "Last Active");
    println!("{}", "━".repeat(82));

    for p in &projects {
        let fav = if p.favorite { "★ " } else { "  " };
        let rel = relative_time(&p.last_accessed);
        let name = if p.display_name.len() > 35 { &p.display_name[..35] } else { &p.display_name };
        println!("{:<3} {:<35} {:<12} {:<8} {:>8} {:>12}", fav.yellow(), name, p.category, p.status, p.session_count, rel.dimmed());
    }
    println!("\nTotal: {} projects", projects.len().to_string().cyan());
    Ok(())
}

// --- Preview ---

fn cmd_preview(mgr: &RegistryManager, home: &std::path::Path, folder: &str) -> Result<()> {
    use colored::*;
    if folder.starts_with("__") { return Ok(()); }
    let Some(p) = mgr.get(folder)? else { eprintln!("Project not found: {folder}"); return Ok(()) };

    let fav = if p.favorite { " ★" } else { "" };
    println!("{}{}", p.display_name.bold().cyan(), fav.yellow());
    if p.description != "Project" && p.description != "—" { println!("{}", p.description.dimmed()); }
    println!();
    println!("  {:<15} {}", "Category:".bold(), p.category);
    println!("  {:<15} {}", "Status:".bold(), p.status);
    if !p.tags.is_empty() { println!("  {:<15} {}", "Tags:".bold(), p.tags.join(", ")); }
    println!("  {:<15} {}", "Created:".bold(), p.created.format("%Y-%m-%d"));
    println!("  {:<15} {}", "Last Active:".bold(), relative_time(&p.last_accessed));
    println!("  {:<15} {}", "Sessions:".bold(), p.session_count);

    let dir = home.join(&p.folder_name);
    if dir.exists() {
        let count = walkdir::WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).count();
        println!("  {:<15} {}", "Files:".bold(), count);
    }
    if let Some(ref url) = p.git_link { println!("  {:<15} {}", "Git:".bold(), url.green()); }
    Ok(())
}

// --- Info ---

fn cmd_info(mgr: &RegistryManager, folder: &str) -> Result<()> {
    let project = mgr.get(folder)?.with_context(|| format!("project not found: {folder}"))?;
    println!("{}", serde_json::to_string_pretty(&project)?);
    Ok(())
}

// --- Registry ---

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

// --- Shell Init ---

fn cmd_shell_init() -> Result<()> {
    print!(r#"# Claude Project Manager — shell integration
# Generated by cpm shell-init
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

// --- FZF internal ---

fn cmd_list_fzf(mgr: &RegistryManager, mode: &str) -> Result<()> {
    let mode: ListMode = mode.parse().unwrap_or(ListMode::Quick);
    for p in mgr.list_sorted(mode)? {
        println!("{}\t{}", p.display_name, p.folder_name);
    }
    println!("➕ New Project\t__NEW_PROJECT__");
    println!("💬 Quick Session (no project)\t__QUICK_SESSION__");
    Ok(())
}

fn cmd_prompt_input(label: &str) -> Result<()> {
    let input: String = dialoguer::Input::new().with_prompt(label).interact_text()?;
    print!("{input}");
    Ok(())
}

// --- Pre-launch ---

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
            let _ = std::process::Command::new("axon").args(["analyze", &d])
                .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
        });
        eprintln!("{} axon analyze (background)", "i".blue());
    }

    // Refresh tldr (background)
    if cmd_exists("tldr") {
        let d = dir.clone();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("tldr").args(["warm", "."])
                .current_dir(&d).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
        });
        eprintln!("{} tldr warm (background)", "i".blue());
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

// --- Create ---

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

// --- Archive/Restore/Delete ---

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

// --- Integrity ---

fn cmd_integrity(mgr: &RegistryManager, home: &std::path::Path, sub: IntegritySub) -> Result<()> {
    use colored::*;
    let reg = mgr.load()?;

    // Registry entries with no directory
    let mut missing = Vec::new();
    for p in &reg.projects {
        if !home.join(&p.folder_name).exists() {
            missing.push(p.folder_name.clone());
        }
    }

    // Directories not in registry
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
                println!("{}", "+ Registry and filesystem in sync".green());
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

// --- Helpers ---

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
