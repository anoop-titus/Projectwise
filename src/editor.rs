//! Minimal, self-contained multiline text editor for the Projectwise TUI.
//!
//! Used by the CLAUDE.md tab and the session intro-prompt tab for true in-TUI
//! editing (no external $EDITOR). UTF-8 aware: the cursor column is a CHAR
//! index, converted to a byte index only at the moment a `String` is mutated.
//! `lines` is never empty — there is always at least one (possibly empty) line.

/// In-memory editor state: the buffer, cursor, vertical scroll, dirty flag.
#[derive(Debug, Clone)]
pub struct TextEditor {
    pub lines: Vec<String>,
    pub cy: usize, // cursor line index
    pub cx: usize, // cursor column as a CHAR index within lines[cy]
    pub top: usize, // first visible line (vertical scroll offset)
    pub dirty: bool,
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Byte offset of the `n`-th char (or the string length if n >= char count).
fn byte_at(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len())
}

impl TextEditor {
    pub fn from_str(s: &str) -> Self {
        let mut lines: Vec<String> = s.split('\n').map(|l| l.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        TextEditor { lines, cy: 0, cx: 0, top: 0, dirty: false }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    fn cur_len(&self) -> usize {
        char_len(&self.lines[self.cy])
    }

    /// Keep cx within the current line after vertical moves.
    fn clamp_cx(&mut self) {
        let l = self.cur_len();
        if self.cx > l {
            self.cx = l;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let b = byte_at(&self.lines[self.cy], self.cx);
        self.lines[self.cy].insert(b, c);
        self.cx += 1;
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let b = byte_at(&self.lines[self.cy], self.cx);
        let rest = self.lines[self.cy].split_off(b);
        self.lines.insert(self.cy + 1, rest);
        self.cy += 1;
        self.cx = 0;
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cx > 0 {
            let start = byte_at(&self.lines[self.cy], self.cx - 1);
            let end = byte_at(&self.lines[self.cy], self.cx);
            self.lines[self.cy].replace_range(start..end, "");
            self.cx -= 1;
            self.dirty = true;
        } else if self.cy > 0 {
            // Join this line onto the end of the previous line.
            let cur = self.lines.remove(self.cy);
            self.cy -= 1;
            self.cx = self.cur_len();
            self.lines[self.cy].push_str(&cur);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self) {
        if self.cx < self.cur_len() {
            let start = byte_at(&self.lines[self.cy], self.cx);
            let end = byte_at(&self.lines[self.cy], self.cx + 1);
            self.lines[self.cy].replace_range(start..end, "");
            self.dirty = true;
        } else if self.cy + 1 < self.lines.len() {
            let next = self.lines.remove(self.cy + 1);
            self.lines[self.cy].push_str(&next);
            self.dirty = true;
        }
    }

    pub fn left(&mut self) {
        if self.cx > 0 {
            self.cx -= 1;
        } else if self.cy > 0 {
            self.cy -= 1;
            self.cx = self.cur_len();
        }
    }

    pub fn right(&mut self) {
        if self.cx < self.cur_len() {
            self.cx += 1;
        } else if self.cy + 1 < self.lines.len() {
            self.cy += 1;
            self.cx = 0;
        }
    }

    pub fn up(&mut self) {
        if self.cy > 0 {
            self.cy -= 1;
            self.clamp_cx();
        }
    }

    pub fn down(&mut self) {
        if self.cy + 1 < self.lines.len() {
            self.cy += 1;
            self.clamp_cx();
        }
    }

    pub fn home(&mut self) {
        self.cx = 0;
    }

    pub fn end(&mut self) {
        self.cx = self.cur_len();
    }

    pub fn page_up(&mut self, rows: usize) {
        let n = rows.max(1);
        self.cy = self.cy.saturating_sub(n);
        self.clamp_cx();
    }

    pub fn page_down(&mut self, rows: usize) {
        let n = rows.max(1);
        self.cy = (self.cy + n).min(self.lines.len().saturating_sub(1));
        self.clamp_cx();
    }

    /// Adjust `top` so the cursor line is within a window of `view_rows`.
    pub fn clamp_scroll(&mut self, view_rows: usize) {
        let h = view_rows.max(1);
        if self.cy < self.top {
            self.top = self.cy;
        } else if self.cy >= self.top + h {
            self.top = self.cy + 1 - h;
        }
    }

    /// Render into `area` with a bordered block titled `title`; `status` is shown
    /// on the last inner row. Places the terminal cursor at the editing point.
    pub fn render(
        &mut self,
        f: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        title: &str,
        status: &str,
        theme_border: ratatui::style::Style,
        theme_title: ratatui::style::Style,
        theme_text: ratatui::style::Style,
    ) {
        use ratatui::layout::Position;
        use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme_border)
            .title(format!(" {title} "))
            .title_style(theme_title);
        let inner = block.inner(area);
        f.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Reserve the last inner row for the status line.
        let text_rows = inner.height.saturating_sub(1) as usize;
        self.clamp_scroll(text_rows);

        // Horizontal offset so the cursor column stays visible.
        let width = inner.width as usize;
        let hoff = if self.cx >= width { self.cx + 1 - width } else { 0 };

        let mut body = String::new();
        for i in 0..text_rows {
            let li = self.top + i;
            if li >= self.lines.len() {
                break;
            }
            let line = &self.lines[li];
            let slice: String = line.chars().skip(hoff).take(width).collect();
            body.push_str(&slice);
            body.push('\n');
        }
        let para = Paragraph::new(body).style(theme_text);
        f.render_widget(para, inner);

        // Status line on the last inner row.
        let status_area = ratatui::layout::Rect::new(
            inner.x,
            inner.y + inner.height - 1,
            inner.width,
            1,
        );
        let st: String = status.chars().take(width).collect();
        f.render_widget(Paragraph::new(st).style(theme_title), status_area);

        // Cursor position (only if within the visible text window).
        if self.cy >= self.top && self.cy < self.top + text_rows {
            let scr_y = inner.y + (self.cy - self.top) as u16;
            let scr_x = inner.x + (self.cx - hoff) as u16;
            f.set_cursor_position(Position::new(scr_x, scr_y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_text() {
        let s = "line one\nline two\nthird";
        assert_eq!(TextEditor::from_str(s).to_string(), s);
    }

    #[test]
    fn empty_buffer_has_one_line() {
        let e = TextEditor::from_str("");
        assert_eq!(e.lines.len(), 1);
        assert_eq!(e.to_string(), "");
    }

    #[test]
    fn insert_and_newline() {
        let mut e = TextEditor::from_str("ab");
        e.end();
        e.insert_char('c'); // abc
        e.insert_newline(); // abc\n
        e.insert_char('d'); // abc\nd
        assert_eq!(e.to_string(), "abc\nd");
        assert!(e.dirty);
    }

    #[test]
    fn backspace_joins_lines() {
        let mut e = TextEditor::from_str("ab\ncd");
        e.cy = 1;
        e.cx = 0;
        e.backspace(); // join -> "abcd", cursor after b
        assert_eq!(e.to_string(), "abcd");
        assert_eq!((e.cy, e.cx), (0, 2));
    }

    #[test]
    fn delete_at_eol_joins_next() {
        let mut e = TextEditor::from_str("ab\ncd");
        e.cy = 0;
        e.end(); // cx=2
        e.delete();
        assert_eq!(e.to_string(), "abcd");
    }

    #[test]
    fn utf8_cursor_is_char_indexed() {
        let mut e = TextEditor::from_str("héllo");
        e.home();
        e.right(); // after h
        e.right(); // after é
        e.insert_char('X'); // hé X llo -> "héXllo"
        assert_eq!(e.to_string(), "héXllo");
    }

    #[test]
    fn vertical_move_clamps_column() {
        let mut e = TextEditor::from_str("longline\nhi");
        e.cy = 0;
        e.end(); // cx = 8
        e.down(); // line "hi" len 2 -> cx clamps to 2
        assert_eq!(e.cx, 2);
    }

    #[test]
    fn scroll_keeps_cursor_visible() {
        let mut e = TextEditor::from_str(&(0..50).map(|i| i.to_string()).collect::<Vec<_>>().join("\n"));
        e.cy = 40;
        e.clamp_scroll(10);
        assert!(e.top <= 40 && 40 < e.top + 10);
    }
}
