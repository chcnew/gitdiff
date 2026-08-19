use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Staged,
    Unstaged,
    Untracked,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub index_status: char,
    pub worktree_status: char,
    pub area: Area,
}

#[derive(Debug, Clone)]
pub struct DiffSides {
    pub left: String,
    pub right: String,
    pub left_label: String,
    pub right_label: String,
    pub binary: bool,
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub subject: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct CommitFile {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened_at: i64,
}
