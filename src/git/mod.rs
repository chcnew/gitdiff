pub mod parse;
pub mod types;

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::Result;
use tokio::process::Command;

pub use types::*;

pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
}

impl GitOutput {
    /// 合并 stdout + stderr 作为展示消息（trim 后）。
    pub fn message(&self) -> String {
        let mut s = String::new();
        if !self.stdout.trim().is_empty() {
            s.push_str(self.stdout.trim());
        }
        if !self.stderr.trim().is_empty() {
            if !s.is_empty() {
                s.push('\n');
            }
            s.push_str(self.stderr.trim());
        }
        s
    }

    pub fn ok(&self) -> bool {
        self.code == Some(0)
    }
}

pub fn check_git() -> Result<String> {
    let out = std::process::Command::new("git").arg("--version").output()?;
    if !out.status.success() {
        anyhow::bail!("未找到可用 git，请安装后加入 PATH");
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// 统一 `git` 子进程封装。工作目录为仓库路径，禁用交互提示。
pub async fn run_git(cwd: &Path, args: &[String]) -> GitOutput {
    let mut cmd = Command::new("git");
    cmd.args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    match cmd.output().await {
        Ok(o) => GitOutput {
            stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
            code: o.status.code(),
        },
        Err(e) => GitOutput {
            stdout: String::new(),
            stderr: e.to_string(),
            code: Some(-1),
        },
    }
}

pub async fn is_repo(path: &Path) -> bool {
    let out = run_git(
        path,
        &["rev-parse".into(), "--is-inside-work-tree".into()],
    )
    .await;
    out.code == Some(0) && out.stdout.trim() == "true"
}

pub async fn toplevel(path: &Path) -> Option<PathBuf> {
    let out = run_git(path, &["rev-parse".into(), "--show-toplevel".into()]).await;
    if out.code == Some(0) {
        Some(PathBuf::from(out.stdout.trim()))
    } else {
        None
    }
}

pub async fn status(repo: &Path) -> Vec<FileChange> {
    let out = run_git(
        repo,
        &["status".into(), "--porcelain=v1".into(), "-uall".into()],
    )
    .await;
    parse::parse_status(&out.stdout)
}

pub async fn log(repo: &Path, limit: usize) -> Vec<CommitInfo> {
    let fmt = format!(
        "--pretty=format:%H%x1f%s%x1f%an%x1f%ad",
    );
    let out = run_git(
        repo,
        &[
            "log".into(),
            fmt,
            "--date=short".into(),
            format!("-{limit}"),
        ],
    )
    .await;
    parse::parse_log(&out.stdout)
}

pub async fn branches(repo: &Path) -> Vec<BranchInfo> {
    let out = run_git(repo, &["branch".into(), "--list".into()]).await;
    parse::parse_branches(&out.stdout)
}

pub async fn current_branch(repo: &Path) -> String {
    let out = run_git(
        repo,
        &["rev-parse".into(), "--abbrev-ref".into(), "HEAD".into()],
    )
    .await;
    if out.code == Some(0) {
        out.stdout.trim().to_string()
    } else {
        String::new()
    }
}

pub async fn stage(repo: &Path, paths: &[String]) -> GitOutput {
    let mut args: Vec<String> = vec!["add".into(), "--".into()];
    args.extend(paths.iter().cloned());
    run_git(repo, &args).await
}

pub async fn unstage(repo: &Path, paths: &[String]) -> GitOutput {
    let mut args: Vec<String> = vec!["reset".into(), "HEAD".into(), "--".into()];
    args.extend(paths.iter().cloned());
    run_git(repo, &args).await
}

pub async fn commit(repo: &Path, message: &str) -> GitOutput {
    run_git(repo, &["commit".into(), "-m".into(), message.to_string()]).await
}

pub async fn push(repo: &Path) -> GitOutput {
    run_git(repo, &["push".into()]).await
}

pub async fn pull(repo: &Path) -> GitOutput {
    run_git(repo, &["pull".into(), "--rebase=false".into()]).await
}

pub async fn checkout(repo: &Path, branch: &str) -> GitOutput {
    run_git(repo, &["checkout".into(), branch.to_string()]).await
}

pub async fn create_branch(repo: &Path, name: &str) -> GitOutput {
    run_git(repo, &["checkout".into(), "-b".into(), name.to_string()]).await
}

pub async fn unified_diff(repo: &Path, path: &str, staged: bool) -> String {
    let mut args: Vec<String> = vec!["diff".into()];
    if staged {
        args.push("--cached".into());
    }
    args.push("--".into());
    args.push(path.to_string());
    let out = run_git(repo, &args).await;
    out.stdout
}

pub async fn diff_sides(
    repo: &Path,
    path: &str,
    staged: bool,
    is_untracked: bool,
) -> DiffSides {
    let (left, right, left_label, right_label) = if staged {
        let l = show_blob(repo, &format!("HEAD:{path}")).await;
        let r = show_blob(repo, &format!(":{path}")).await;
        (l, r, "HEAD".to_string(), "Index".to_string())
    } else if is_untracked {
        (
            String::new(),
            read_worktree(repo, path).await,
            "空".to_string(),
            "工作区".to_string(),
        )
    } else {
        let l = show_blob(repo, &format!(":{path}")).await;
        let r = read_worktree(repo, path).await;
        (l, r, "Index".to_string(), "工作区".to_string())
    };

    let binary = is_binary(&left) || is_binary(&right);
    DiffSides {
        left,
        right,
        left_label,
        right_label,
        binary,
    }
}

pub async fn commit_files(repo: &Path, hash: &str) -> Vec<CommitFile> {
    let out = run_git(
        repo,
        &[
            "show".into(),
            "--name-status".into(),
            "--pretty=format:".into(),
            hash.to_string(),
        ],
    )
    .await;
    parse::parse_name_status(&out.stdout)
}

pub async fn show_file_patch(repo: &Path, commit: &str, path: &str) -> String {
    let out = run_git(
        repo,
        &[
            "show".into(),
            commit.to_string(),
            "--".into(),
            path.to_string(),
        ],
    )
    .await;
    if out.stdout.is_empty() {
        out.stderr
    } else {
        out.stdout
    }
}

pub async fn list_dir(repo: &Path, rel: &str) -> Vec<DirEntry> {
    let dir = if rel.is_empty() {
        repo.to_path_buf()
    } else {
        repo.join(rel)
    };
    let mut entries = Vec::new();
    let mut rd = match tokio::fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    while let Ok(Some(entry)) = rd.next_entry().await {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if is_noise_dir(&name) {
            continue;
        }
        let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
        let rel_path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        entries.push(DirEntry {
            name,
            path: rel_path,
            is_dir,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

pub async fn read_file(repo: &Path, rel: &str) -> Result<String, String> {
    let bytes = tokio::fs::read(repo.join(rel))
        .await
        .map_err(|e| format!("读取失败: {e}"))?;
    if is_binary_bytes(&bytes) {
        return Err("二进制文件，拒绝编辑".to_string());
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub async fn write_file(repo: &Path, rel: &str, content: &str) -> Result<(), String> {
    let path = repo.join(rel);
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    tokio::fs::write(&path, content)
        .await
        .map_err(|e| e.to_string())
}

async fn show_blob(repo: &Path, spec: &str) -> String {
    let out = run_git(repo, &["show".into(), spec.to_string()]).await;
    if out.code == Some(0) {
        out.stdout
    } else {
        String::new()
    }
}

async fn read_worktree(repo: &Path, rel: &str) -> String {
    match tokio::fs::read(repo.join(rel)).await {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(_) => String::new(),
    }
}

fn is_binary(s: &str) -> bool {
    is_binary_bytes(s.as_bytes())
}

fn is_binary_bytes(b: &[u8]) -> bool {
    b.iter().take(8192).any(|&x| x == 0)
}

fn is_noise_dir(name: &str) -> bool {
    matches!(name, ".git" | "node_modules" | "target" | "dist" | "build")
}
