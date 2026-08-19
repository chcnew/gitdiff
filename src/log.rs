use std::path::PathBuf;

use anyhow::Result;

/// 初始化文件日志。仅写文件（不写 stderr，避免破坏 TUI 画面）。
pub fn init(verbose: bool) -> Result<()> {
    let dir = log_dir();
    std::fs::create_dir_all(&dir)?;
    let appender = tracing_appender::rolling::never(&dir, "gitdiff.log");
    let level = if verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(format!("gitdiff_tui={level},info"))
        .with_ansi(false)
        .with_writer(appender)
        .init();

    Ok(())
}

fn log_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let d = parent.join("logs");
            if d.is_dir() || std::fs::create_dir_all(&d).is_ok() {
                return d;
            }
        }
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("GitDiffTUI")
        .join("logs")
}
