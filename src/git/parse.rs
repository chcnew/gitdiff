use super::types::{
    Area, BranchInfo, CommitFile, CommitInfo, FileChange,
};

/// 解析 `git status --porcelain=v1 -uall` 输出。
pub fn parse_status(s: &str) -> Vec<FileChange> {
    let mut out = Vec::new();
    for line in s.lines() {
        if line.len() < 4 {
            continue;
        }
        let b = line.as_bytes();
        let x = b[0] as char;
        let y = b[1] as char;
        let rest = line[3..].trim_end();

        let (path, old_path) = parse_path(rest);

        if x == '?' && y == '?' {
            out.push(FileChange {
                path,
                old_path: None,
                index_status: '?',
                worktree_status: '?',
                area: Area::Untracked,
            });
            continue;
        }

        if x != ' ' {
            out.push(FileChange {
                path: path.clone(),
                old_path: old_path.clone(),
                index_status: x,
                worktree_status: y,
                area: Area::Staged,
            });
        }
        if y != ' ' {
            out.push(FileChange {
                path,
                old_path,
                index_status: x,
                worktree_status: y,
                area: Area::Unstaged,
            });
        }
    }
    out
}

fn parse_path(rest: &str) -> (String, Option<String>) {
    if let Some(pos) = rest.find(" -> ") {
        let old = rest[..pos].trim().trim_matches('"').to_string();
        let new = rest[pos + 4..].trim().trim_matches('"').to_string();
        (new, Some(old))
    } else {
        (rest.trim_matches('"').to_string(), None)
    }
}

/// 解析 `git log --pretty=format:%H%x1f%s%x1f%an%x1f%ad` 输出。
pub fn parse_log(s: &str) -> Vec<CommitInfo> {
    s.lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.split('\u{1f}').collect();
            if parts.len() < 4 {
                return None;
            }
            Some(CommitInfo {
                hash: parts[0].to_string(),
                subject: parts[1].to_string(),
                author: parts[2].to_string(),
                date: parts[3].to_string(),
            })
        })
        .collect()
}

/// 解析 `git branch --list` 输出。
pub fn parse_branches(s: &str) -> Vec<BranchInfo> {
    s.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() {
                return None;
            }
            let (is_current, name) = match l.strip_prefix("* ") {
                Some(rest) => (true, rest),
                None => (false, l),
            };
            Some(BranchInfo {
                name: name.to_string(),
                is_current,
            })
        })
        .collect()
}

/// 解析 `git show --name-status --pretty=format:` 输出。
pub fn parse_name_status(s: &str) -> Vec<CommitFile> {
    s.lines()
        .filter_map(|l| {
            let l = l.trim_end();
            if l.is_empty() {
                return None;
            }
            let first = l.as_bytes()[0] as char;
            if !first.is_ascii_alphabetic() {
                return None;
            }
            let mut parts = l.split('\t');
            let status_full = parts.next().unwrap_or("");
            let status = status_full.chars().next().unwrap_or(' ').to_string();
            let path = parts.last().unwrap_or("").to_string();
            if path.is_empty() {
                return None;
            }
            Some(CommitFile { path, status })
        })
        .collect()
}
