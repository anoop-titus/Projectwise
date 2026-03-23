use ratatui::style::{Color, Modifier, Style};

// ── Palette ──────────────────────────────────────────────────────────
// Dark-mode theme with cyan accent, warm amber warnings, soft greens.

pub const BG: Color = Color::Rgb(22, 22, 30);         // near-black blue
pub const FG: Color = Color::Rgb(200, 200, 210);      // soft white
pub const DIM: Color = Color::Rgb(130, 130, 150);     // muted gray (WCAG AA)
pub const ACCENT: Color = Color::Rgb(0, 210, 210);    // cyan
pub const FAV: Color = Color::Rgb(255, 210, 60);      // gold star
pub const ROW_ALT: Color = Color::Rgb(28, 28, 40);    // subtle stripe
pub const BORDER: Color = Color::Rgb(70, 70, 95);     // border gray

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
