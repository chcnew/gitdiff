use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use notify::RecommendedWatcher;
use ratatui::layout::{Position, Rect};
use tokio::sync::mpsc::UnboundedSender;

use crate::diff::{side_by_side, DiffRow, DiffTag};
use crate::event::Action;
use crate::git::types::{Area, DiffSides, DirEntry, FileChange};
use crate::git::{self, CommitFile};
use crate::recent::Recent;
use crate::terminal::CmdRunner;
use crate::watcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Welcome,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftTab {
    Changes,
    Files,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterView {
    Diff,
    Editor,
    HistoryDiff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Left,
    Center,
    History,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Path,
    Commit,
    Branch,
    Command,
    Edit,
}

#[derive(Debug, Default)]
pub struct ChangesState {
    pub items: Vec<FileChange>,
    pub selected: usize,
}

#[derive(Debug, Default)]
pub struct HistoryState {
    pub commits: Vec<git::CommitInfo>,
    pub selected: usize,
    pub in_files: bool,
    pub files: Vec<CommitFile>,
    pub files_selected: usize,
    pub selected_hash: Option<String>,
    pub detail: Option<(String, String)>,
    pub detail_scroll: usize,
}

#[derive(Debug, Default)]
pub struct BranchState {
    pub items: Vec<git::BranchInfo>,
    pub current: String,
    pub visible: bool,
    pub selected: usize,
}

#[derive(Debug, Clone)]
pub struct FlatRow {
    pub depth: usize,
    pub path: String,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Default)]
pub struct FileTreeState {
    pub root: Vec<DirEntry>,
    pub children: HashMap<String, Vec<DirEntry>>,
    pub open: HashSet<String>,
    pub selected: usize,
}

impl FileTreeState {
    pub fn flat(&self) -> Vec<FlatRow> {
        let mut rows = Vec::new();
        self.walk("", &self.root, 0, &mut rows);
        rows
    }

    fn walk(&self, dir: &str, entries: &[DirEntry], depth: usize, rows: &mut Vec<FlatRow>) {
        for e in entries {
            let rel = if dir.is_empty() {
                e.path.clone()
            } else {
                format!("{dir}/{}", e.path)
            };
            rows.push(FlatRow {
                depth,
                path: rel.clone(),
                name: e.name.clone(),
                is_dir: e.is_dir,
            });
            if e.is_dir && self.open.contains(&rel) {
                if let Some(children) = self.children.get(&rel) {
                    self.walk(&rel, children, depth + 1, rows);
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct EditorState {
    pub path: String,
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub dirty: bool,
    pub scroll: usize,
}

#[derive(Debug)]
pub struct DiffState {
    pub path: String,
    pub staged: bool,
    pub untracked: bool,
    pub sides: DiffSides,
    pub unified: String,
    pub rows: Vec<DiffRow>,
    pub hunks: Vec<usize>,
    pub hunk_idx: usize,
    pub side_by_side: bool,
    pub scroll: usize,
    pub loading: bool,
}

impl Default for DiffState {
    fn default() -> Self {
        DiffState {
            path: String::new(),
            staged: false,
            untracked: false,
            sides: DiffSides {
                left: String::new(),
                right: String::new(),
                left_label: String::new(),
                right_label: String::new(),
                binary: false,
            },
            unified: String::new(),
            rows: Vec::new(),
            hunks: Vec::new(),
            hunk_idx: 0,
            side_by_side: true,
            scroll: 0,
            loading: false,
        }
    }
}

/// 各面板区域（上一帧计算，供鼠标命中测试）。
#[derive(Debug, Clone, Copy)]
pub struct Rects {
    pub left: Rect,
    pub center: Rect,
    pub history: Rect,
    pub terminal: Rect,
    pub left_list: Rect,
    pub history_list: Rect,
    pub recent_list: Rect,
}

impl Default for Rects {
    fn default() -> Self {
        Rects {
            left: Rect::new(0, 0, 0, 0),
            center: Rect::new(0, 0, 0, 0),
            history: Rect::new(0, 0, 0, 0),
            terminal: Rect::new(0, 0, 0, 0),
            left_list: Rect::new(0, 0, 0, 0),
            history_list: Rect::new(0, 0, 0, 0),
            recent_list: Rect::new(0, 0, 0, 0),
        }
    }
}

pub struct App {
    pub tx: UnboundedSender<Action>,
    pub mode: Mode,
    pub repo_path: Option<PathBuf>,
    pub repo_name: String,
    pub current_branch: String,
    pub git_version: String,

    pub focus: Panel,
    pub left_tab: LeftTab,
    pub center_view: CenterView,
    pub input_mode: InputMode,
    pub input: String,

    pub changes: ChangesState,
    pub history: HistoryState,
    pub branches: BranchState,
    pub filetree: FileTreeState,
    pub editor: EditorState,
    pub diff: DiffState,
    pub runner: CmdRunner,

    pub recent: Recent,
    pub recent_list: Vec<git::RecentProject>,
    pub recent_selected: usize,

    pub status_msg: Option<String>,
    pub error_msg: Option<String>,
    pub help_open: bool,
    pub should_quit: bool,
    pub watcher: Option<RecommendedWatcher>,
    pub rects: Rects,
}

impl App {
    pub fn new(tx: UnboundedSender<Action>, git_version: String) -> Self {
        let recent = Recent::new();
        let recent_list = recent.load();
        App {
            tx,
            mode: Mode::Welcome,
            repo_path: None,
            repo_name: String::new(),
            current_branch: String::new(),
            git_version,
            focus: Panel::Left,
            left_tab: LeftTab::Changes,
            center_view: CenterView::Diff,
            input_mode: InputMode::Normal,
            input: String::new(),
            changes: ChangesState::default(),
            history: HistoryState::default(),
            branches: BranchState::default(),
            filetree: FileTreeState::default(),
            editor: EditorState::default(),
            diff: DiffState::default(),
            runner: CmdRunner::new(),
            recent,
            recent_list,
            recent_selected: 0,
            status_msg: None,
            error_msg: None,
            help_open: false,
            should_quit: false,
            watcher: None,
            rects: Rects::default(),
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Tick => self.runner.poll(),
            Action::Key(key) => self.handle_key(key),
            Action::Mouse(mouse) => self.handle_mouse(mouse),
            Action::Resize => {}
            Action::RepoOpened { path, name } => self.on_repo_opened(path, name),
            Action::StatusLoaded(items) => {
                self.changes.items = items;
                if self.changes.selected >= self.changes.items.len() {
                    self.changes.selected = self.changes.items.len().saturating_sub(1);
                }
            }
            Action::LogLoaded(commits) => {
                self.history.commits = commits;
                if self.history.selected >= self.history.commits.len() {
                    self.history.selected = self.history.commits.len().saturating_sub(1);
                }
            }
            Action::BranchesLoaded(items) => {
                self.branches.current = items
                    .iter()
                    .find(|b| b.is_current)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                self.branches.items = items;
                if self.branches.selected >= self.branches.items.len() {
                    self.branches.selected = self.branches.items.len().saturating_sub(1);
                }
            }
            Action::CurrentBranchLoaded(b) => self.current_branch = b,
            Action::DiffLoaded {
                path,
                staged,
                untracked,
                sides,
                unified,
            } => self.on_diff_loaded(path, staged, untracked, sides, unified),
            Action::CommitFilesLoaded { hash, files } => {
                if self.history.selected_hash.as_deref() == Some(hash.as_str()) {
                    self.history.files = files;
                    self.history.files_selected = 0;
                }
            }
            Action::HistoryDiffLoaded { path, content } => {
                self.history.detail = Some((path, content));
                self.history.detail_scroll = 0;
                self.center_view = CenterView::HistoryDiff;
                self.focus = Panel::Center;
            }
            Action::DirLoaded { path, entries } => {
                self.filetree.children.insert(path, entries);
            }
            Action::FileLoaded { path, content } => self.on_file_loaded(path, content),
            Action::FileSaved => {
                self.editor.dirty = false;
                self.reload_status();
                self.status_msg = Some("已保存".to_string());
            }
            Action::OpFinished(msg) => {
                self.status_msg = Some(msg);
                self.reload_status();
                self.reload_log();
                self.reload_branches();
                self.reload_current_branch();
            }
            Action::Error(msg) => self.error_msg = Some(msg),
            Action::RepoChanged => self.reload_status(),
            Action::CmdOutput { data } => self.runner.push_output(data),
        }
    }

    // ---- key 分发 ----

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        self.status_msg = None;
        self.error_msg = None;

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => {
                    if self.input_mode == InputMode::Command {
                        self.runner.interrupt();
                    } else {
                        self.should_quit = true;
                    }
                    return;
                }
                KeyCode::Char('s') => {
                    if self.input_mode == InputMode::Edit {
                        self.save_file();
                    }
                    return;
                }
                _ => return,
            }
        }

        match self.input_mode {
            InputMode::Path | InputMode::Commit | InputMode::Branch | InputMode::Command => {
                match key.code {
                    KeyCode::Enter => self.confirm_input(),
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.input.clear();
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    KeyCode::Up if self.input_mode == InputMode::Command => self.runner.hist_up(),
                    KeyCode::Down if self.input_mode == InputMode::Command => {
                        self.runner.hist_down()
                    }
                    KeyCode::Char(c) => self.input.push(c),
                    _ => {}
                }
                return;
            }
            InputMode::Edit => {
                self.handle_edit_key(key);
                return;
            }
            InputMode::Normal => {}
        }

        if self.help_open {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')
            ) {
                self.help_open = false;
            }
            return;
        }

        match self.mode {
            Mode::Welcome => self.handle_welcome_key(key),
            Mode::Workspace => self.handle_workspace_key(key),
        }
    }

    fn handle_welcome_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.help_open = true,
            KeyCode::Char('o') => {
                self.input_mode = InputMode::Path;
                self.input.clear();
            }
            KeyCode::Enter => {
                if let Some(r) = self.recent_list.get(self.recent_selected) {
                    let p = PathBuf::from(r.path.clone());
                    self.request_open_repo(p);
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.recent_selected = (self.recent_selected + 1)
                    .min(self.recent_list.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.recent_selected = self.recent_selected.saturating_sub(1);
            }
            KeyCode::Char('d') | KeyCode::Char('x') => {
                if let Some(r) = self.recent_list.get(self.recent_selected) {
                    let p = r.path.clone();
                    self.recent_list = self.recent.remove(&p);
                    if self.recent_selected >= self.recent_list.len() {
                        self.recent_selected = self.recent_list.len().saturating_sub(1);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_workspace_key(&mut self, key: KeyEvent) {
        if self.branches.visible {
            match key.code {
                KeyCode::Esc => {
                    self.branches.visible = false;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.branches.selected = (self.branches.selected + 1)
                        .min(self.branches.items.len().saturating_sub(1));
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.branches.selected = self.branches.selected.saturating_sub(1);
                }
                KeyCode::Enter => self.checkout_selected(),
                KeyCode::Char('c') => {
                    self.branches.visible = false;
                    self.input_mode = InputMode::Branch;
                    self.input.clear();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('?') => {
                self.help_open = true;
                return;
            }
            KeyCode::Char('t') => {
                self.runner.visible = !self.runner.visible;
                return;
            }
            KeyCode::Char('r') => {
                self.reload_all();
                return;
            }
            KeyCode::Char('x') => {
                self.close_repo();
                return;
            }
            KeyCode::Char('b') => {
                self.branches.visible = true;
                self.branches.selected = self
                    .branches
                    .items
                    .iter()
                    .position(|b| b.is_current)
                    .unwrap_or(0);
                return;
            }
            KeyCode::Char('P') => {
                self.push();
                return;
            }
            KeyCode::Char('F') => {
                self.pull();
                return;
            }
            KeyCode::Tab => {
                self.focus_next();
                return;
            }
            KeyCode::BackTab => {
                self.focus_prev();
                return;
            }
            KeyCode::Char('1') => {
                self.focus = Panel::Left;
                return;
            }
            KeyCode::Char('2') => {
                self.focus = Panel::Center;
                return;
            }
            KeyCode::Char('3') => {
                self.focus = Panel::History;
                return;
            }
            KeyCode::Char('4') => {
                self.focus = Panel::Terminal;
                return;
            }
            KeyCode::Char('[') => {
                self.left_tab = LeftTab::Changes;
                self.focus = Panel::Left;
                return;
            }
            KeyCode::Char(']') => {
                self.left_tab = LeftTab::Files;
                self.focus = Panel::Left;
                return;
            }
            _ => {}
        }

        match self.focus {
            Panel::Left => self.handle_left_key(key),
            Panel::Center => self.handle_center_key(key),
            Panel::History => self.handle_history_key(key),
            Panel::Terminal => self.handle_terminal_key(key),
        }
    }

    fn handle_left_key(&mut self, key: KeyEvent) {
        match self.left_tab {
            LeftTab::Changes => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.changes.selected = (self.changes.selected + 1)
                        .min(self.changes.items.len().saturating_sub(1));
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.changes.selected = self.changes.selected.saturating_sub(1);
                }
                KeyCode::Char('g') => self.changes.selected = 0,
                KeyCode::Char('G') => {
                    self.changes.selected = self.changes.items.len().saturating_sub(1);
                }
                KeyCode::Char(' ') => self.toggle_stage_selected(),
                KeyCode::Char('u') => self.unstage_selected(),
                KeyCode::Char('a') => self.stage_all(),
                KeyCode::Char('c') => {
                    self.input_mode = InputMode::Commit;
                    self.input.clear();
                }
                KeyCode::Enter => self.select_change(),
                _ => {}
            },
            LeftTab::Files => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    let len = self.filetree.flat().len().saturating_sub(1);
                    self.filetree.selected = (self.filetree.selected + 1).min(len);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.filetree.selected = self.filetree.selected.saturating_sub(1);
                }
                KeyCode::Char('g') => self.filetree.selected = 0,
                KeyCode::Char('G') => {
                    self.filetree.selected = self.filetree.flat().len().saturating_sub(1);
                }
                KeyCode::Enter | KeyCode::Char('l') => self.filetree_activate(),
                KeyCode::Char('h') => self.filetree_collapse(),
                KeyCode::Char('e') => self.filetree_edit(),
                _ => {}
            },
        }
    }

    fn handle_center_key(&mut self, key: KeyEvent) {
        match self.center_view {
            CenterView::Diff => match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.diff.scroll = self.diff.scroll.saturating_add(1),
                KeyCode::Char('k') | KeyCode::Up => self.diff.scroll = self.diff.scroll.saturating_sub(1),
                KeyCode::Char('g') => self.diff.scroll = 0,
                KeyCode::Char('G') => self.diff.scroll = usize::MAX / 2,
                KeyCode::Char('v') => self.diff.side_by_side = !self.diff.side_by_side,
                KeyCode::Char('n') => self.diff_next(),
                KeyCode::Char('N') => self.diff_prev(),
                _ => {}
            },
            CenterView::HistoryDiff => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.history.detail_scroll = self.history.detail_scroll.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.history.detail_scroll = self.history.detail_scroll.saturating_sub(1);
                }
                _ => {}
            },
            CenterView::Editor => {
                if key.code == KeyCode::Enter {
                    self.input_mode = InputMode::Edit;
                }
            }
        }
    }

    fn handle_history_key(&mut self, key: KeyEvent) {
        if self.history.in_files {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.history.files_selected = (self.history.files_selected + 1)
                        .min(self.history.files.len().saturating_sub(1));
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.history.files_selected = self.history.files_selected.saturating_sub(1);
                }
                KeyCode::Enter => self.open_history_file(),
                KeyCode::Esc => self.history.in_files = false,
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.history.selected = (self.history.selected + 1)
                        .min(self.history.commits.len().saturating_sub(1));
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.history.selected = self.history.selected.saturating_sub(1);
                }
                KeyCode::Char('g') => self.history.selected = 0,
                KeyCode::Char('G') => {
                    self.history.selected = self.history.commits.len().saturating_sub(1);
                }
                KeyCode::Enter => self.select_commit(),
                _ => {}
            }
        }
    }

    fn handle_terminal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('i') | KeyCode::Enter => {
                self.input_mode = InputMode::Command;
                self.input.clear();
                self.runner.visible = true;
            }
            _ => {}
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                return;
            }
            KeyCode::Char(c) => {
                let e = &mut self.editor;
                if let Some(line) = e.lines.get_mut(e.cursor_line) {
                    line.insert(e.cursor_col, c);
                    e.cursor_col = next_boundary(line, e.cursor_col);
                    e.dirty = true;
                }
            }
            KeyCode::Backspace => {
                let e = &mut self.editor;
                if e.cursor_col > 0 {
                    if let Some(line) = e.lines.get_mut(e.cursor_line) {
                        let prev = prev_boundary(line, e.cursor_col);
                        line.remove(prev);
                        e.cursor_col = prev;
                        e.dirty = true;
                    }
                } else if e.cursor_line > 0 {
                    let cur = e.lines.remove(e.cursor_line);
                    e.cursor_line -= 1;
                    let prev_len = e.lines[e.cursor_line].len();
                    e.lines[e.cursor_line].push_str(&cur);
                    e.cursor_col = prev_len;
                    e.dirty = true;
                }
            }
            KeyCode::Enter => {
                let e = &mut self.editor;
                let line = e.lines[e.cursor_line].clone();
                let (keep, rest) = line.split_at(e.cursor_col);
                e.lines[e.cursor_line] = keep.to_string();
                e.lines.insert(e.cursor_line + 1, rest.to_string());
                e.cursor_line += 1;
                e.cursor_col = 0;
                e.dirty = true;
            }
            KeyCode::Left => {
                let e = &mut self.editor;
                if let Some(line) = e.lines.get(e.cursor_line) {
                    e.cursor_col = prev_boundary(line, e.cursor_col);
                }
            }
            KeyCode::Right => {
                let e = &mut self.editor;
                if let Some(line) = e.lines.get(e.cursor_line) {
                    e.cursor_col = next_boundary(line, e.cursor_col);
                }
            }
            KeyCode::Up => {
                self.editor.cursor_line = self.editor.cursor_line.saturating_sub(1);
                self.clamp_editor_col();
            }
            KeyCode::Down => {
                let e = &mut self.editor;
                e.cursor_line = (e.cursor_line + 1).min(e.lines.len().saturating_sub(1));
                self.clamp_editor_col();
            }
            KeyCode::Home => self.editor.cursor_col = 0,
            KeyCode::End => {
                let e = &mut self.editor;
                e.cursor_col = e.lines[e.cursor_line].len();
            }
            _ => {}
        }
    }

    fn clamp_editor_col(&mut self) {
        let e = &mut self.editor;
        if let Some(line) = e.lines.get(e.cursor_line) {
            if e.cursor_col > line.len() {
                e.cursor_col = line.len();
            }
        }
    }

    // ---- 输入确认 ----

    fn confirm_input(&mut self) {
        match self.input_mode {
            InputMode::Path => {
                let p = std::mem::take(&mut self.input);
                self.input_mode = InputMode::Normal;
                self.request_open_repo(PathBuf::from(p));
            }
            InputMode::Commit => {
                let msg = std::mem::take(&mut self.input);
                self.input_mode = InputMode::Normal;
                self.do_commit(msg);
            }
            InputMode::Branch => {
                let name = std::mem::take(&mut self.input);
                self.input_mode = InputMode::Normal;
                self.do_create_branch(name);
            }
            InputMode::Command => {
                let line = std::mem::take(&mut self.input);
                self.input_mode = InputMode::Normal;
                if let Some(repo) = &self.repo_path {
                    self.runner.spawn(&line, repo, self.tx.clone());
                }
            }
            _ => {}
        }
    }

    // ---- 仓库 ----

    pub fn request_open_repo(&mut self, path: PathBuf) {
        let tx = self.tx.clone();
        let p = path.clone();
        tokio::spawn(async move {
            if !git::is_repo(&p).await {
                let _ = tx.send(Action::Error(format!("不是 Git 仓库: {}", p.display())));
                return;
            }
            let top = git::toplevel(&p).await.unwrap_or(p);
            let name = top
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| top.display().to_string());
            let _ = tx.send(Action::RepoOpened { path: top, name });
        });
    }

    fn on_repo_opened(&mut self, path: PathBuf, name: String) {
        self.repo_path = Some(path.clone());
        self.repo_name = name.clone();
        self.mode = Mode::Workspace;
        self.input_mode = InputMode::Normal;
        self.input.clear();
        self.recent_list = self.recent.add(path.display().to_string(), name);
        self.recent_selected = 0;
        self.start_watcher(path.clone());
        self.reload_all();
        self.status_msg = Some(format!("已打开 {}", path.display()));
    }

    fn close_repo(&mut self) {
        self.watcher = None;
        self.repo_path = None;
        self.repo_name.clear();
        self.current_branch.clear();
        self.mode = Mode::Welcome;
        self.changes = ChangesState::default();
        self.history = HistoryState::default();
        self.branches = BranchState::default();
        self.filetree = FileTreeState::default();
        self.editor = EditorState::default();
        self.diff = DiffState::default();
    }

    fn start_watcher(&mut self, path: PathBuf) {
        self.watcher = Some(watcher::start(&path, self.tx.clone()));
    }

    // ---- 变更操作 ----

    fn toggle_stage_selected(&mut self) {
        if let Some(fc) = self.changes.items.get(self.changes.selected).cloned() {
            if fc.area == Area::Staged {
                self.unstage_selected();
            } else {
                self.stage_paths(vec![fc.path]);
            }
        }
    }

    fn unstage_selected(&mut self) {
        if let Some(fc) = self.changes.items.get(self.changes.selected).cloned() {
            if fc.area == Area::Staged {
                let repo = self.repo_path.clone().unwrap();
                let tx = self.tx.clone();
                let paths = vec![fc.path];
                tokio::spawn(async move {
                    let out = git::unstage(&repo, &paths).await;
                    if out.ok() {
                        let _ = tx.send(Action::OpFinished("已取消暂存".to_string()));
                    } else {
                        let _ = tx.send(Action::Error(format!("取消暂存失败: {}", out.message())));
                    }
                });
            }
        }
    }

    fn stage_all(&mut self) {
        let paths: Vec<String> = self
            .changes
            .items
            .iter()
            .filter(|f| f.area != Area::Staged)
            .map(|f| f.path.clone())
            .collect();
        if !paths.is_empty() {
            self.stage_paths(paths);
        }
    }

    fn stage_paths(&mut self, paths: Vec<String>) {
        let repo = self.repo_path.clone().unwrap();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let out = git::stage(&repo, &paths).await;
            if out.ok() {
                let _ = tx.send(Action::OpFinished("已暂存".to_string()));
            } else {
                let _ = tx.send(Action::Error(format!("暂存失败: {}", out.message())));
            }
        });
    }

    fn do_commit(&mut self, msg: String) {
        if msg.trim().is_empty() {
            return;
        }
        let repo = self.repo_path.clone().unwrap();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let out = git::commit(&repo, &msg).await;
            if out.ok() {
                let _ = tx.send(Action::OpFinished("提交成功".to_string()));
            } else {
                let _ = tx.send(Action::Error(format!("提交失败: {}", out.message())));
            }
        });
    }

    // ---- Diff ----

    fn select_change(&mut self) {
        if let Some(fc) = self.changes.items.get(self.changes.selected).cloned() {
            let staged = fc.area == Area::Staged;
            let untracked = fc.area == Area::Untracked;
            self.load_diff(fc.path, staged, untracked);
        }
    }

    fn load_diff(&mut self, path: String, staged: bool, untracked: bool) {
        self.diff.loading = true;
        self.center_view = CenterView::Diff;
        self.focus = Panel::Center;
        let repo = self.repo_path.clone().unwrap();
        let tx = self.tx.clone();
        let p = path.clone();
        tokio::spawn(async move {
            let sides = git::diff_sides(&repo, &p, staged, untracked).await;
            let unified = git::unified_diff(&repo, &p, staged).await;
            let _ = tx.send(Action::DiffLoaded {
                path: p,
                staged,
                untracked,
                sides,
                unified,
            });
        });
    }

    fn on_diff_loaded(
        &mut self,
        path: String,
        staged: bool,
        untracked: bool,
        sides: DiffSides,
        unified: String,
    ) {
        let rows = if sides.binary {
            Vec::new()
        } else {
            side_by_side(&sides.left, &sides.right, 2000)
        };
        let hunks: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.tag != DiffTag::Equal)
            .map(|(i, _)| i)
            .collect();
        self.diff = DiffState {
            path,
            staged,
            untracked,
            sides,
            unified,
            rows,
            hunks,
            hunk_idx: 0,
            side_by_side: true,
            scroll: 0,
            loading: false,
        };
    }

    fn diff_next(&mut self) {
        if self.diff.hunks.is_empty() {
            return;
        }
        self.diff.hunk_idx = (self.diff.hunk_idx + 1) % self.diff.hunks.len();
        self.diff_center_active();
    }

    fn diff_prev(&mut self) {
        if self.diff.hunks.is_empty() {
            return;
        }
        self.diff.hunk_idx = (self.diff.hunk_idx + self.diff.hunks.len() - 1)
            % self.diff.hunks.len();
        self.diff_center_active();
    }

    fn diff_center_active(&mut self) {
        if let Some(&row) = self.diff.hunks.get(self.diff.hunk_idx) {
            self.diff.scroll = row.saturating_sub(4);
        }
    }

    // ---- 历史 ----

    fn select_commit(&mut self) {
        if let Some(c) = self.history.commits.get(self.history.selected) {
            let hash = c.hash.clone();
            self.history.selected_hash = Some(hash.clone());
            self.history.in_files = true;
            self.history.files_selected = 0;
            let repo = self.repo_path.clone().unwrap();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let files = git::commit_files(&repo, &hash).await;
                let _ = tx.send(Action::CommitFilesLoaded { hash, files });
            });
        }
    }

    fn open_history_file(&mut self) {
        let hash = self.history.selected_hash.clone();
        let file = self
            .history
            .files
            .get(self.history.files_selected)
            .cloned();
        if let (Some(hash), Some(f)) = (hash, file) {
            let path = f.path;
            let repo = self.repo_path.clone().unwrap();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let content = git::show_file_patch(&repo, &hash, &path).await;
                let _ = tx.send(Action::HistoryDiffLoaded { path, content });
            });
        }
    }

    // ---- 分支 ----

    fn checkout_selected(&mut self) {
        if let Some(b) = self.branches.items.get(self.branches.selected) {
            let branch = b.name.clone();
            self.branches.visible = false;
            let repo = self.repo_path.clone().unwrap();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let out = git::checkout(&repo, &branch).await;
                if out.ok() {
                    let _ = tx.send(Action::OpFinished(format!("已切换到 {branch}")));
                } else {
                    let _ = tx.send(Action::Error(format!("切换失败: {}", out.message())));
                }
            });
        }
    }

    fn do_create_branch(&mut self, name: String) {
        if name.trim().is_empty() {
            return;
        }
        let repo = self.repo_path.clone().unwrap();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let out = git::create_branch(&repo, &name).await;
            if out.ok() {
                let _ = tx.send(Action::OpFinished(format!("已创建并切换 {name}")));
            } else {
                let _ = tx.send(Action::Error(format!("创建失败: {}", out.message())));
            }
        });
    }

    // ---- 远程 ----

    pub fn push(&mut self) {
        self.git_remote("push");
    }

    pub fn pull(&mut self) {
        self.git_remote("pull");
    }

    fn git_remote(&mut self, kind: &str) {
        let repo = self.repo_path.clone().unwrap();
        let tx = self.tx.clone();
        let kind = kind.to_string();
        tokio::spawn(async move {
            let out = if kind == "push" {
                git::push(&repo).await
            } else {
                git::pull(&repo).await
            };
            if out.ok() {
                let _ = tx.send(Action::OpFinished(format!("{kind} 完成")));
            } else {
                let _ = tx.send(Action::Error(format!("{kind} 失败: {}", out.message())));
            }
        });
    }

    // ---- 文件树 / 编辑 ----

    fn filetree_activate(&mut self) {
        let flat = self.filetree.flat();
        if let Some(row) = flat.get(self.filetree.selected) {
            if row.is_dir {
                self.toggle_dir(row.path.clone());
            } else {
                self.open_file_in_editor(row.path.clone());
            }
        }
    }

    fn filetree_collapse(&mut self) {
        let flat = self.filetree.flat();
        if let Some(row) = flat.get(self.filetree.selected) {
            if row.is_dir {
                self.filetree.open.remove(&row.path);
            }
        }
    }

    fn filetree_edit(&mut self) {
        let flat = self.filetree.flat();
        if let Some(row) = flat.get(self.filetree.selected) {
            if !row.is_dir {
                self.open_file_in_editor(row.path.clone());
            }
        }
    }

    fn toggle_dir(&mut self, rel: String) {
        if self.filetree.open.contains(&rel) {
            self.filetree.open.remove(&rel);
        } else {
            self.filetree.open.insert(rel.clone());
            if !self.filetree.children.contains_key(&rel) {
                let repo = self.repo_path.clone().unwrap();
                let tx = self.tx.clone();
                let r = rel.clone();
                tokio::spawn(async move {
                    let entries = git::list_dir(&repo, &r).await;
                    let _ = tx.send(Action::DirLoaded {
                        path: r,
                        entries,
                    });
                });
            }
        }
    }

    fn open_file_in_editor(&mut self, rel: String) {
        let repo = self.repo_path.clone().unwrap();
        let tx = self.tx.clone();
        let r = rel.clone();
        tokio::spawn(async move {
            match git::read_file(&repo, &r).await {
                Ok(content) => {
                    let _ = tx.send(Action::FileLoaded { path: r, content });
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(e));
                }
            }
        });
    }

    fn on_file_loaded(&mut self, path: String, content: String) {
        let lines: Vec<String> = content.split('\n').map(|s| s.to_string()).collect();
        self.editor = EditorState {
            path,
            lines,
            cursor_line: 0,
            cursor_col: 0,
            dirty: false,
            scroll: 0,
        };
        self.center_view = CenterView::Editor;
        self.input_mode = InputMode::Edit;
        self.focus = Panel::Center;
    }

    fn save_file(&mut self) {
        if !self.editor.dirty {
            return;
        }
        let path = self.editor.path.clone();
        let content = self.editor.lines.join("\n");
        let repo = self.repo_path.clone().unwrap();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match git::write_file(&repo, &path, &content).await {
                Ok(()) => {
                    let _ = tx.send(Action::FileSaved);
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(format!("保存失败: {e}")));
                }
            }
        });
    }

    // ---- 焦点 ----

    fn focus_next(&mut self) {
        self.focus = match self.focus {
            Panel::Left => Panel::Center,
            Panel::Center => Panel::History,
            Panel::History => Panel::Terminal,
            Panel::Terminal => Panel::Left,
        };
    }

    fn focus_prev(&mut self) {
        self.focus = match self.focus {
            Panel::Left => Panel::Terminal,
            Panel::Center => Panel::Left,
            Panel::History => Panel::Center,
            Panel::Terminal => Panel::History,
        };
    }

    // ---- 鼠标 ----

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.help_open {
            self.help_open = false;
            return;
        }
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => self.click(mouse.column, mouse.row),
            MouseEventKind::ScrollUp => self.scroll(-1),
            MouseEventKind::ScrollDown => self.scroll(1),
            _ => {}
        }
    }

    fn click(&mut self, x: u16, y: u16) {
        if self.mode == Mode::Welcome {
            if self.rects.recent_list.contains(Position::new(x, y)) {
                let idx = (y - self.rects.recent_list.y) as usize;
                let start = crate::ui::window_start(
                    self.recent_list.len(),
                    self.recent_selected,
                    self.rects.recent_list.height as usize,
                );
                let target = start + idx;
                if target < self.recent_list.len() {
                    self.recent_selected = target;
                }
            }
            return;
        }

        if self.runner.visible && self.rects.terminal.contains(Position::new(x, y)) {
            self.focus = Panel::Terminal;
        } else if self.rects.left.contains(Position::new(x, y)) {
            self.focus = Panel::Left;
            self.click_left(x, y);
        } else if self.rects.center.contains(Position::new(x, y)) {
            self.focus = Panel::Center;
        } else if self.rects.history.contains(Position::new(x, y)) {
            self.focus = Panel::History;
            self.click_history(x, y);
        }
    }

    fn click_left(&mut self, x: u16, y: u16) {
        let list = self.rects.left_list;
        if !list.contains(Position::new(x, y)) {
            return;
        }
        let idx = (y - list.y) as usize;
        match self.left_tab {
            LeftTab::Changes => {
                let start =
                    crate::ui::window_start(self.changes.items.len(), self.changes.selected, list.height as usize);
                let target = start + idx;
                if target < self.changes.items.len() {
                    self.changes.selected = target;
                }
            }
            LeftTab::Files => {
                let flat = self.filetree.flat();
                let start = crate::ui::window_start(flat.len(), self.filetree.selected, list.height as usize);
                let target = start + idx;
                if target < flat.len() {
                    self.filetree.selected = target;
                }
            }
        }
    }

    fn click_history(&mut self, x: u16, y: u16) {
        let list = self.rects.history_list;
        if !list.contains(Position::new(x, y)) {
            return;
        }
        let row = (y - list.y) as usize;
        if self.history.in_files {
            let start =
                crate::ui::window_start(self.history.files.len(), self.history.files_selected, list.height as usize);
            let target = start + row;
            if target < self.history.files.len() {
                self.history.files_selected = target;
            }
        } else {
            let height = (list.height as usize) / 2;
            let start = crate::ui::window_start(self.history.commits.len(), self.history.selected, height);
            let target = start + row / 2;
            if target < self.history.commits.len() {
                self.history.selected = target;
            }
        }
    }

    fn scroll(&mut self, dir: i32) {
        match (self.mode, self.focus) {
            (Mode::Welcome, _) => {
                self.recent_selected = move_idx(self.recent_selected, dir, self.recent_list.len());
            }
            (Mode::Workspace, Panel::Left) => match self.left_tab {
                LeftTab::Changes => {
                    self.changes.selected = move_idx(self.changes.selected, dir, self.changes.items.len());
                }
                LeftTab::Files => {
                    let len = self.filetree.flat().len();
                    self.filetree.selected = move_idx(self.filetree.selected, dir, len);
                }
            },
            (Mode::Workspace, Panel::Center) => match self.center_view {
                CenterView::Diff => self.diff.scroll = add_scroll(self.diff.scroll, dir),
                CenterView::HistoryDiff => {
                    self.history.detail_scroll = add_scroll(self.history.detail_scroll, dir);
                }
                CenterView::Editor => {
                    let len = self.editor.lines.len();
                    self.editor.cursor_line = move_idx(self.editor.cursor_line, dir, len);
                    self.clamp_editor_col();
                }
            },
            (Mode::Workspace, Panel::History) => {
                if self.history.in_files {
                    self.history.files_selected = move_idx(self.history.files_selected, dir, self.history.files.len());
                } else {
                    self.history.selected = move_idx(self.history.selected, dir, self.history.commits.len());
                }
            }
            (Mode::Workspace, Panel::Terminal) => {
                self.runner.scroll = add_scroll(self.runner.scroll, dir);
            }
        }
    }

    // ---- 刷新 ----

    fn reload_all(&mut self) {
        self.reload_status();
        self.reload_log();
        self.reload_branches();
        self.reload_current_branch();
        self.reload_root_dir();
    }

    fn reload_status(&mut self) {
        let repo = match &self.repo_path {
            Some(r) => r.clone(),
            None => return,
        };
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let items = git::status(&repo).await;
            let _ = tx.send(Action::StatusLoaded(items));
        });
    }

    fn reload_log(&mut self) {
        let repo = match &self.repo_path {
            Some(r) => r.clone(),
            None => return,
        };
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let commits = git::log(&repo, 100).await;
            let _ = tx.send(Action::LogLoaded(commits));
        });
    }

    fn reload_branches(&mut self) {
        let repo = match &self.repo_path {
            Some(r) => r.clone(),
            None => return,
        };
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let items = git::branches(&repo).await;
            let _ = tx.send(Action::BranchesLoaded(items));
        });
    }

    fn reload_current_branch(&mut self) {
        let repo = match &self.repo_path {
            Some(r) => r.clone(),
            None => return,
        };
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let b = git::current_branch(&repo).await;
            let _ = tx.send(Action::CurrentBranchLoaded(b));
        });
    }

    fn reload_root_dir(&mut self) {
        let repo = match &self.repo_path {
            Some(r) => r.clone(),
            None => return,
        };
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let entries = git::list_dir(&repo, "").await;
            let _ = tx.send(Action::DirLoaded {
                path: String::new(),
                entries,
            });
        });
    }
}

fn move_idx(cur: usize, dir: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    if dir > 0 {
        (cur + 1).min(len - 1)
    } else {
        cur.saturating_sub(1)
    }
}

fn add_scroll(cur: usize, dir: i32) -> usize {
    if dir > 0 {
        cur + 1
    } else {
        cur.saturating_sub(1)
    }
}

fn next_boundary(s: &str, i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    let mut iter = s.char_indices();
    while let Some((idx, _)) = iter.next() {
        if idx == i {
            return match iter.next() {
                Some((idx2, _)) => idx2,
                None => s.len(),
            };
        }
        if idx > i {
            return idx;
        }
    }
    s.len()
}

fn prev_boundary(s: &str, i: usize) -> usize {
    if i == 0 {
        return 0;
    }
    let idx = i.min(s.len());
    s[..idx]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}
