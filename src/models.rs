use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub folder_name: String,
    pub display_name: String,
    #[serde(default = "default_description")]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_status")]
    pub status: ProjectStatus,
    pub created: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    #[serde(default)]
    pub session_count: u32,
    #[serde(default)]
    pub git_link: Option<String>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub archive_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Paused,
    Archived,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for ProjectStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "archived" => Ok(Self::Archived),
            _ => anyhow::bail!("invalid status: {s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMeta {
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: String,
    pub projects: Vec<Project>,
    pub metadata: RegistryMeta,
}

impl Registry {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            version: "3.0.0".to_string(),
            projects: Vec::new(),
            metadata: RegistryMeta {
                created: now,
                updated: now,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListMode {
    Quick,
    Favorite,
    All,
}

impl std::str::FromStr for ListMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quick" => Ok(Self::Quick),
            "favorite" => Ok(Self::Favorite),
            "all" => Ok(Self::All),
            _ => anyhow::bail!("invalid mode: {s} (expected quick/favorite/all)"),
        }
    }
}

fn default_description() -> String {
    "Project".to_string()
}
fn default_category() -> String {
    "Research".to_string()
}
fn default_status() -> ProjectStatus {
    ProjectStatus::Active
}
