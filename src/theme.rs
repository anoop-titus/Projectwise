use ratatui::style::{Color, Modifier, Style};
use std::sync::Mutex;

// ── Dynamic color system backed by opaline ──────────────────────────

#[derive(Debug, Clone)]
pub struct AppColors {
    pub bg: Color,
    pub fg: Color,
    pub dim: Color,
    pub accent: Color,
    pub fav: Color,
    pub row_alt: Color,
    pub border: Color,
    pub status_active: Color,
    pub status_paused: Color,
}

static APP_COLORS: Mutex<Option<AppColors>> = Mutex::new(None);
static THEME_NAME: Mutex<Option<String>> = Mutex::new(None);

fn default_colors() -> AppColors {
    AppColors {
        bg: Color::Rgb(22, 22, 30),
        fg: Color::Rgb(200, 200, 210),
        dim: Color::Rgb(130, 130, 150),
        accent: Color::Rgb(0, 210, 210),
        fav: Color::Rgb(255, 210, 60),
        row_alt: Color::Rgb(28, 28, 40),
        border: Color::Rgb(70, 70, 95),
        status_active: Color::Rgb(80, 220, 120),
        status_paused: Color::Rgb(230, 150, 40),
    }
}

fn load_opaline_colors(name: &str) -> AppColors {
    let defaults = default_colors();
    let Some(theme) = opaline::load_by_name(name) else {
        return defaults;
    };

    let extract = |token: &str, fallback: Color| -> Color {
        theme
            .try_color(token)
            .map(|c| Color::Rgb(c.r, c.g, c.b))
            .unwrap_or(fallback)
    };

    AppColors {
        bg: extract("bg.base", defaults.bg),
        fg: extract("text.primary", defaults.fg),
        dim: extract("text.muted", defaults.dim),
        accent: extract("accent.primary", defaults.accent),
        fav: extract("warning", defaults.fav),
        row_alt: extract("bg.panel", defaults.row_alt),
        border: extract("border.unfocused", defaults.border),
        status_active: extract("success", defaults.status_active),
        status_paused: extract("warning", defaults.status_paused),
    }
}

/// Initialize theme from cpm.toml config
pub fn init_theme() {
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("cpm.toml");
    let theme_name = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|l| l.starts_with("theme"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|s| s.trim().trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "default".to_string())
    } else {
        "default".to_string()
    };

    let colors = if theme_name == "default" {
        default_colors()
    } else {
        load_opaline_colors(&theme_name)
    };

    if let Ok(mut guard) = APP_COLORS.lock() {
        *guard = Some(colors);
    }
    if let Ok(mut guard) = THEME_NAME.lock() {
        *guard = Some(theme_name);
    }
}

/// Reload theme by name: swap colors in memory and persist to cpm.toml
pub fn reload_theme(name: &str) -> anyhow::Result<()> {
    let colors = if name == "default" {
        default_colors()
    } else {
        load_opaline_colors(name)
    };

    if let Ok(mut guard) = APP_COLORS.lock() {
        *guard = Some(colors);
    }
    if let Ok(mut guard) = THEME_NAME.lock() {
        *guard = Some(name.to_string());
    }

    // Persist to config
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("cpm.toml");
    let config = format!("theme = \"{}\"\n", name);
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(config_path, config)?;
    Ok(())
}

/// Get current theme name
pub fn current_theme_name() -> String {
    THEME_NAME
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| "default".to_string())
}

fn colors() -> AppColors {
    APP_COLORS
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(default_colors)
}

// ── Color accessors ─────────────────────────────────────────────────

pub fn bg() -> Color {
    colors().bg
}
pub fn fg() -> Color {
    colors().fg
}
pub fn accent() -> Color {
    colors().accent
}
pub fn fav() -> Color {
    colors().fav
}
pub fn row_alt_color() -> Color {
    colors().row_alt
}
pub fn border_color() -> Color {
    colors().border
}
pub fn dim_color() -> Color {
    colors().dim
}

// ── Composed styles ─────────────────────────────────────────────────

pub fn title() -> Style {
    Style::default().fg(accent()).add_modifier(Modifier::BOLD)
}

pub fn header() -> Style {
    Style::default()
        .fg(accent())
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

pub fn row_normal() -> Style {
    Style::default().fg(fg()).bg(bg())
}

pub fn row_alt() -> Style {
    Style::default().fg(fg()).bg(row_alt_color())
}

pub fn dim() -> Style {
    Style::default().fg(dim_color())
}

pub fn favorite() -> Style {
    Style::default().fg(fav())
}

pub fn border() -> Style {
    Style::default().fg(border_color())
}

pub fn status_style(status: &str) -> Style {
    let c = colors();
    match status.to_lowercase().as_str() {
        "active" => Style::default().fg(c.status_active),
        "paused" => Style::default().fg(c.status_paused),
        "archived" => dim(),
        _ => Style::default().fg(c.fg),
    }
}

/// List available themes from opaline + a "default" entry
pub fn available_themes() -> Vec<ThemeEntry> {
    let mut entries = vec![ThemeEntry {
        name: "default".to_string(),
        display_name: "Default (Built-in)".to_string(),
        is_dark: true,
    }];

    for info in opaline::list_available_themes() {
        entries.push(ThemeEntry {
            name: info.name.clone(),
            display_name: info.display_name.clone(),
            is_dark: matches!(info.variant, opaline::ThemeVariant::Dark),
        });
    }
    entries
}

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub name: String,
    pub display_name: String,
    pub is_dark: bool,
}
