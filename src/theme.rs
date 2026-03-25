use ratatui::style::{Color, Modifier, Style};
use std::sync::OnceLock;

// ── Palette ──────────────────────────────────────────────────────────
// Dark-mode theme with cyan accent, warm amber warnings, soft greens.
pub const BG: Color = Color::Rgb(22, 22, 30); // near-black blue
pub const FG: Color = Color::Rgb(200, 200, 210); // soft white
pub const DIM: Color = Color::Rgb(130, 130, 150); // muted gray (WCAG AA)
pub const ACCENT: Color = Color::Rgb(0, 210, 210); // cyan
pub const FAV: Color = Color::Rgb(255, 210, 60); // gold star
pub const ROW_ALT: Color = Color::Rgb(28, 28, 40); // subtle stripe
pub const BORDER: Color = Color::Rgb(70, 70, 95); // border gray

static THEME_NAME: OnceLock<String> = OnceLock::new();

/// Initialize theme (loads from config)
pub fn init_theme() {
    let config_path = dirs::home_dir().unwrap().join(".claude").join("cpm.toml");
    let theme_name = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                content.lines()
                    .find(|l| l.starts_with("theme"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| "default".to_string())
            }
            Err(_) => "default".to_string(),
        }
    } else {
        "default".to_string()
    };
    let _ = THEME_NAME.get_or_init(|| theme_name);
}

/// Get current theme name
pub fn current_theme_name() -> &'static str {
    THEME_NAME.get().map(|s| s.as_str()).unwrap_or("default")
}

// ── Composed styles ──────────────────────────────────────────────────
pub fn title() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn header() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

pub fn row_normal() -> Style {
    Style::default().fg(FG).bg(BG)
}

pub fn row_alt() -> Style {
    Style::default().fg(FG).bg(ROW_ALT)
}

pub fn dim() -> Style {
    Style::default().fg(DIM)
}

pub fn favorite() -> Style {
    Style::default().fg(FAV)
}

pub fn border() -> Style {
    Style::default().fg(BORDER)
}

pub fn status_style(status: &str) -> Style {
    match status {
        "active" => Style::default().fg(Color::Rgb(80, 220, 120)),
        "paused" => Style::default().fg(Color::Rgb(230, 150, 40)),
        "archived" => Style::default().fg(DIM),
        _ => Style::default().fg(FG),
    }
}

/// Available themes (id, name)
pub fn available_themes() -> Vec<(&'static str, &'static str)> {
    vec![
        ("default", "Default"),
        ("tokyo-night", "Tokyo Night"),
        ("catppuccin-mocha", "Catppuccin Mocha"),
        ("dracula", "Dracula"),
        ("gruvbox-dark", "Gruvbox Dark"),
        ("nord", "Nord"),
        ("rose-pine", "Rose Pine"),
        ("one-dark", "One Dark"),
        ("kanagawa-dragon", "Kanagawa Dragon"),
        ("monokai-pro", "Monokai Pro"),
        ("night-owl", "Night Owl"),
    ]
}

/// Check if theme is dark
pub fn is_theme_dark(name: &str) -> bool {
    !name.contains("light")
}

/// Set theme by name and save to config
pub fn set_theme(name: &str) -> anyhow::Result<()> {
    // Save to config
    let config_path = dirs::home_dir().unwrap().join(".claude").join("cpm.toml");
    let config = format!("theme = \"{}\"\n", name);
    std::fs::create_dir_all(config_path.parent().unwrap())?;
    std::fs::write(config_path, config)?;
    Ok(())
}
