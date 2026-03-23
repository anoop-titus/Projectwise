use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

use crate::models::{ListMode, Project, ProjectStatus, Registry};

pub struct RegistryManager {
    path: PathBuf,
}

impl RegistryManager {
    pub fn new(home: &Path) -> Self {
        Self {
            path: home.join(".registry.json"),
        }
    }

    pub fn registry_path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn load(&self) -> Result<Registry> {
        let data = fs::read_to_string(&self.path)
            .with_context(|| format!("registry not found at {}. Run 'cpm registry init'", self.path.display()))?;
        serde_json::from_str(&data).context("failed to parse registry JSON")
    }

    pub fn save(&self, registry: &Registry) -> Result<()> {
        self.backup()?;
        let dir = self.path.parent().unwrap();
        let temp = NamedTempFile::new_in(dir).context("failed to create temp file")?;
        serde_json::to_writer_pretty(&temp, registry).context("failed to serialize registry")?;
        temp.persist(&self.path).context("failed to persist registry")?;
        Ok(())
    }

    pub fn init(&self) -> Result<()> {
        if self.exists() {
            anyhow::bail!("registry already exists at {}", self.path.display());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let registry = Registry::new();
        let data = serde_json::to_string_pretty(&registry)?;
        fs::write(&self.path, data)?;
        Ok(())
    }

    // --- CRUD ---

    pub fn add(&self, folder_name: &str, display_name: &str, description: &str, category: &str) -> Result<()> {
        let mut reg = self.load()?;
        let now = Utc::now();
        reg.projects.push(Project {
            id: folder_name.to_string(),
            folder_name: folder_name.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            tags: Vec::new(),
            category: category.to_string(),
            status: ProjectStatus::Active,
            created: now,
            last_accessed: now,
            session_count: 0,
            git_link: None,
            favorite: false,
            archive_path: None,
        });
        reg.metadata.updated = now;
        self.save(&reg)
    }

    pub fn remove(&self, folder_name: &str) -> Result<()> {
        let mut reg = self.load()?;
        reg.projects.retain(|p| p.folder_name != folder_name);
        reg.metadata.updated = Utc::now();
        self.save(&reg)
    }

    pub fn get(&self, folder_name: &str) -> Result<Option<Project>> {
        let reg = self.load()?;
        Ok(reg.projects.into_iter().find(|p| p.folder_name == folder_name))
    }

    pub fn touch(&self, folder_name: &str) -> Result<()> {
        let mut reg = self.load()?;
        if let Some(p) = reg.projects.iter_mut().find(|p| p.folder_name == folder_name) {
            p.last_accessed = Utc::now();
            p.session_count += 1;
        }
        reg.metadata.updated = Utc::now();
        self.save(&reg)
    }

    pub fn set_field(&self, folder_name: &str, field: &str, value: &str) -> Result<()> {
        let mut reg = self.load()?;
        let project = reg.projects.iter_mut()
            .find(|p| p.folder_name == folder_name)
            .with_context(|| format!("project not found: {folder_name}"))?;

        match field {
            "display_name" => project.display_name = value.to_string(),
            "description" => project.description = value.to_string(),
            "category" => project.category = value.to_string(),
            "status" => project.status = value.parse()?,
            "git_link" => project.git_link = if value.is_empty() { None } else { Some(value.to_string()) },
            _ => anyhow::bail!("unknown field: {field}"),
        }
        reg.metadata.updated = Utc::now();
        self.save(&reg)
    }

    pub fn toggle_favorite(&self, folder_name: &str) -> Result<()> {
        let mut reg = self.load()?;
        if let Some(p) = reg.projects.iter_mut().find(|p| p.folder_name == folder_name) {
            p.favorite = !p.favorite;
        }
        reg.metadata.updated = Utc::now();
        self.save(&reg)
    }

    pub fn set_tags(&self, folder_name: &str, tags: Vec<String>) -> Result<()> {
        let mut reg = self.load()?;
        if let Some(p) = reg.projects.iter_mut().find(|p| p.folder_name == folder_name) {
            p.tags = tags;
        }
        reg.metadata.updated = Utc::now();
        self.save(&reg)
    }

    pub fn list_sorted(&self, mode: ListMode) -> Result<Vec<Project>> {
        let reg = self.load()?;
        let mut projects: Vec<Project> = match mode {
            ListMode::Quick => reg.projects.into_iter().filter(|p| p.status == ProjectStatus::Active).collect(),
            ListMode::Favorite => reg.projects.into_iter().filter(|p| p.favorite).collect(),
            ListMode::All => reg.projects,
        };
        projects.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        Ok(projects)
    }

    pub fn list_names(&self) -> Result<Vec<String>> {
        let reg = self.load()?;
        Ok(reg.projects.iter().map(|p| p.folder_name.clone()).collect())
    }

    // --- Backup ---

    fn backup(&self) -> Result<()> {
        if !self.exists() { return Ok(()); }
        let backup_dir = self.path.parent().unwrap().join(".backups");
        fs::create_dir_all(&backup_dir)?;
        let ts = Utc::now().timestamp();
        fs::copy(&self.path, backup_dir.join(format!("registry.{ts}.backup")))?;
        // Prune to last 10
        let mut entries: Vec<_> = fs::read_dir(&backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("registry."))
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.file_name()));
        for entry in entries.iter().skip(10) {
            let _ = fs::remove_file(entry.path());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, RegistryManager) {
        let dir = TempDir::new().unwrap();
        let mgr = RegistryManager::new(dir.path());
        mgr.init().unwrap();
        (dir, mgr)
    }

    #[test]
    fn test_init_and_load() {
        let (_dir, mgr) = setup();
        let reg = mgr.load().unwrap();
        assert_eq!(reg.version, "3.0.0");
        assert!(reg.projects.is_empty());
    }

    #[test]
    fn test_add_and_get() {
        let (_dir, mgr) = setup();
        mgr.add("test-proj", "Test Project", "A test", "Research").unwrap();
        let proj = mgr.get("test-proj").unwrap().unwrap();
        assert_eq!(proj.display_name, "Test Project");
        assert_eq!(proj.status, ProjectStatus::Active);
    }

    #[test]
    fn test_remove() {
        let (_dir, mgr) = setup();
        mgr.add("a", "A", "desc", "Research").unwrap();
        mgr.add("b", "B", "desc", "Research").unwrap();
        mgr.remove("a").unwrap();
        assert!(mgr.get("a").unwrap().is_none());
        assert!(mgr.get("b").unwrap().is_some());
    }

    #[test]
    fn test_toggle_favorite() {
        let (_dir, mgr) = setup();
        mgr.add("x", "X", "d", "R").unwrap();
        assert!(!mgr.get("x").unwrap().unwrap().favorite);
        mgr.toggle_favorite("x").unwrap();
        assert!(mgr.get("x").unwrap().unwrap().favorite);
        mgr.toggle_favorite("x").unwrap();
        assert!(!mgr.get("x").unwrap().unwrap().favorite);
    }

    #[test]
    fn test_touch_increments() {
        let (_dir, mgr) = setup();
        mgr.add("p", "P", "d", "R").unwrap();
        mgr.touch("p").unwrap();
        mgr.touch("p").unwrap();
        assert_eq!(mgr.get("p").unwrap().unwrap().session_count, 2);
    }

    #[test]
    fn test_set_field() {
        let (_dir, mgr) = setup();
        mgr.add("p", "P", "d", "R").unwrap();
        mgr.set_field("p", "display_name", "New Name").unwrap();
        assert_eq!(mgr.get("p").unwrap().unwrap().display_name, "New Name");
    }

    #[test]
    fn test_list_sorted_modes() {
        let (_dir, mgr) = setup();
        mgr.add("a", "A", "d", "R").unwrap();
        mgr.add("b", "B", "d", "R").unwrap();
        mgr.toggle_favorite("b").unwrap();
        mgr.set_field("a", "status", "archived").unwrap();

        let quick = mgr.list_sorted(ListMode::Quick).unwrap();
        assert_eq!(quick.len(), 1);
        assert_eq!(quick[0].folder_name, "b");

        let favs = mgr.list_sorted(ListMode::Favorite).unwrap();
        assert_eq!(favs.len(), 1);

        let all = mgr.list_sorted(ListMode::All).unwrap();
        assert_eq!(all.len(), 2);
    }
}
