use std::path::PathBuf;

use crossterm::event::KeyEvent;

use crate::git::types::{
    BranchInfo, CommitFile, CommitInfo, DiffSides, DirEntry, FileChange,
};

/// 后台任务与输入事件统一归约为 `Action`，由 `App::update` 消费。
#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Key(KeyEvent),
    Resize,

    // 仓库
    RepoOpened { path: PathBuf, name: String },

    // Git 结果回投
    StatusLoaded(Vec<FileChange>),
    LogLoaded(Vec<CommitInfo>),
    BranchesLoaded(Vec<BranchInfo>),
    CurrentBranchLoaded(String),
    DiffLoaded {
        path: String,
        staged: bool,
        untracked: bool,
        sides: DiffSides,
        unified: String,
    },
    CommitFilesLoaded { hash: String, files: Vec<CommitFile> },
    HistoryDiffLoaded { path: String, content: String },

    // 文件
    DirLoaded { path: String, entries: Vec<DirEntry> },
    FileLoaded { path: String, content: String },
    FileSaved,

    // 操作反馈
    OpFinished(String),
    Error(String),

    // watcher
    RepoChanged,

    // 命令运行器输出
    CmdOutput { data: String },
}
