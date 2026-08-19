mod app;
mod diff;
mod event;
mod git;
mod log;
mod recent;
mod terminal;
mod tui;
mod ui;
mod watcher;

use anyhow::Result;
use clap::Parser;
use event::Action;

#[derive(Parser, Debug)]
#[command(name = "gitdiff-tui", version, about = "本地轻量 Git 终端客户端")]
struct Cli {
    /// 直接打开指定仓库路径
    #[arg(short = 'C', long)]
    directory: Option<String>,

    /// 提升日志级别到 debug
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    log::init(cli.verbose)?;
    let git_version = git::check_git()?;

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Action>();
    let mut app = app::App::new(tx.clone(), git_version);

    if let Some(dir) = cli.directory {
        app.request_open_repo(std::path::PathBuf::from(dir));
    }

    tui::run(app, rx).await
}
