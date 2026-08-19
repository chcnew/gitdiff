use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::git::types::RecentProject;

/// 最近项目持久化（最多 20 条，最近打开的置顶）。
pub struct Recent {
    path: PathBuf,
}

impl Recent {
    pub fn new() -> Self {
        Recent {
            path: default_path(),
        }
    }

    pub fn load(&self) -> Vec<RecentProject> {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self, list: &[RecentProject]) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&self.path, serde_json::to_string_pretty(list).unwrap_or_default());
    }

    pub fn add(&self, path: String, name: String) -> Vec<RecentProject> {
        let mut list = self.load();
        list.retain(|p| p.path != path);
        list.insert(
            0,
            RecentProject {
                path,
                name,
                last_opened_at: now_millis(),
            },
        );
        list.truncate(20);
        self.save(&list);
        list
    }

    pub fn remove(&self, path: &str) -> Vec<RecentProject> {
        let mut list = self.load();
        list.retain(|p| p.path != path);
        self.save(&list);
        list
    }
}

fn default_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gitdiff-tui")
        .join("recent-projects.json")
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
