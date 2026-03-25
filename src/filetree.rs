use ratatui::{
    prelude::*,
    widgets::{Block, Borders, BorderType, List, ListItem, ListState},
};
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git"];
const MAX_DEPTH: usize = 4;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn load(path: &Path, depth: usize) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        let is_dir = path.is_dir();
        let mut node = TreeNode {
            name,
            path: path.to_path_buf(),
            depth,
            is_dir,
            expanded: false,
            children: Vec::new(),
        };
        if is_dir && depth < MAX_DEPTH {
            node.children = load_children(path, depth + 1);
        }
        node
    }
}

fn load_children(dir: &Path, depth: usize) -> Vec<TreeNode> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut entries: Vec<_> = rd
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            // Skip hidden, skip known heavy dirs
            if s.starts_with('.') {
                return false;
            }
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                return !SKIP_DIRS.contains(&s.as_ref());
            }
            true
        })
        .collect();
    // Dirs first, then files, both alphabetical
    entries.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });
    entries
        .iter()
        .map(|e| TreeNode::load(&e.path(), depth))
        .collect()
}

pub struct FileTreeState {
    pub root: TreeNode,
    #[allow(dead_code)]
    pub flat: Vec<usize>, // reserved for future scroll tracking
    pub selected: usize,
    #[allow(dead_code)]
    pub scroll_offset: usize,
}

/// Flatten visible nodes into a list of (depth, name, is_dir, expanded, path)
pub fn flatten(node: &TreeNode) -> Vec<(usize, String, bool, bool, PathBuf)> {
    let mut result = Vec::new();
    flatten_inner(node, &mut result);
    result
}

fn flatten_inner(node: &TreeNode, out: &mut Vec<(usize, String, bool, bool, PathBuf)>) {
    // Skip root itself — we show its children at top level
    for child in &node.children {
        out.push((
            child.depth,
            child.name.clone(),
            child.is_dir,
            child.expanded,
            child.path.clone(),
        ));
        if child.is_dir && child.expanded {
            flatten_inner(child, out);
        }
    }
}

impl FileTreeState {
    pub fn new(path: &Path) -> Self {
        let root = TreeNode::load(path, 0);
        FileTreeState {
            root,
            flat: Vec::new(),
            selected: 0,
            scroll_offset: 0,
        }
    }

    pub fn select_next(&mut self, flat_len: usize) {
        if flat_len == 0 {
            return;
        }
        self.selected = (self.selected + 1) % flat_len;
    }

    pub fn select_prev(&mut self, flat_len: usize) {
        if flat_len == 0 {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
        if self.selected == 0 && flat_len > 0 {
            // stay at 0
        }
    }

    /// Toggle expand/collapse on the currently selected node.
    /// We walk the tree to find the node by path.
    pub fn toggle_selected(&mut self, flat: &[(usize, String, bool, bool, PathBuf)]) {
        if let Some(entry) = flat.get(self.selected) {
            if entry.2 {
                // is_dir
                toggle_node(&mut self.root, &entry.4);
            }
        }
    }
}

fn toggle_node(node: &mut TreeNode, target: &Path) -> bool {
    for child in &mut node.children {
        if child.path == target {
            child.expanded = !child.expanded;
            return true;
        }
        if toggle_node(child, target) {
            return true;
        }
    }
    false
}

pub fn render_tree(
    state: &FileTreeState,
    flat: &[(usize, String, bool, bool, PathBuf)],
    area: Rect,
    frame: &mut Frame,
    focused: bool,
    title: &str,
) {
    use crate::theme;

    let border_style = if focused {
        Style::default().fg(theme::ACCENT)
    } else {
        Style::default().fg(theme::BORDER)
    };

    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    let scroll = if state.selected >= visible_height {
        state.selected.saturating_sub(visible_height / 2)
    } else {
        0
    };

    let items: Vec<ListItem> = flat
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, (depth, name, is_dir, expanded, _path))| {
            let indent = "  ".repeat(*depth);
            let icon = if *is_dir {
                if *expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };
            let text = format!("{indent}{icon}{name}");
            let style = if i == state.selected {
                Style::default().bg(theme::ACCENT).fg(theme::BG)
            } else if *is_dir {
                Style::default().fg(theme::ACCENT)
            } else {
                Style::default().fg(theme::FG)
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    if !flat.is_empty() {
        let visible_sel = state.selected.saturating_sub(scroll);
        list_state.select(Some(visible_sel));
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}
