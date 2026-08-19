use std::path::{Component, Path};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::UnboundedSender;

use crate::event::Action;

/// 启动递归监听，900ms 尾沿防抖后回投 `RepoChanged`。
pub fn start(repo: &Path, tx: UnboundedSender<Action>) -> RecommendedWatcher {
    let (stx, srx) = std::sync::mpsc::channel::<notify::Result<Event>>();
    let mut watcher =
        notify::recommended_watcher(move |res| {
            let _ = stx.send(res);
        })
        .expect("创建 watcher 失败");

    let _ = watcher.watch(repo, RecursiveMode::Recursive);

    std::thread::spawn(move || {
        let mut pending = false;
        loop {
            match srx.recv_timeout(Duration::from_millis(900)) {
                Ok(Ok(event)) => {
                    if should_ignore(&event) {
                        continue;
                    }
                    pending = true;
                }
                Ok(Err(_)) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if pending {
                        pending = false;
                        let _ = tx.send(Action::RepoChanged);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    watcher
}

fn should_ignore(event: &Event) -> bool {
    if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
        return true;
    }
    event.paths.iter().any(|p| {
        p.components().any(|c| {
            if let Component::Normal(s) = c {
                matches!(
                    s.to_str(),
                    Some(".git" | "node_modules" | "target" | "dist" | "build")
                )
            } else {
                false
            }
        })
    })
}
