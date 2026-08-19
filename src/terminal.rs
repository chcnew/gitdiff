use std::path::Path;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc::UnboundedSender;

use crate::event::Action;

const MAX_LINES: usize = 2000;

/// 底部命令运行器：在仓库 cwd 启动 shell 命令并回显输出。
pub struct CmdRunner {
    pub visible: bool,
    pub height_percent: u16,
    pub output: Vec<String>,
    pub input: String,
    pub history: Vec<String>,
    pub hist_idx: Option<usize>,
    pub running: bool,
    child: Option<Child>,
}

impl CmdRunner {
    pub fn new() -> Self {
        CmdRunner {
            visible: true,
            height_percent: 25,
            output: vec![String::from("输入命令并回车执行（Ctrl+C 中断，Esc 收起）")],
            input: String::new(),
            history: Vec::new(),
            hist_idx: None,
            running: false,
            child: None,
        }
    }

    pub fn spawn(&mut self, cmdline: &str, cwd: &Path, tx: UnboundedSender<Action>) {
        let cmdline = cmdline.trim();
        if cmdline.is_empty() {
            return;
        }
        self.push_output(format!("$ {cmdline}"));
        self.history.push(cmdline.to_string());
        self.hist_idx = None;
        self.input.clear();

        #[cfg(windows)]
        let mut cmd = {
            let mut c = Command::new("cmd");
            c.args(["/c", cmdline]);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.args(["-c", cmdline]);
            c
        };

        cmd.current_dir(cwd)
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        match cmd.spawn() {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    tokio::spawn(read_stream(stdout, tx.clone()));
                }
                if let Some(stderr) = child.stderr.take() {
                    tokio::spawn(read_stream(stderr, tx));
                }
                self.child = Some(child);
                self.running = true;
            }
            Err(e) => {
                self.push_output(format!("[启动失败] {e}"));
            }
        }
    }

    /// 非阻塞轮询子进程是否退出。
    pub fn poll(&mut self) {
        let mut done = false;
        let mut code = String::new();
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    done = true;
                    code = status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "signal".to_string());
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }
        if done {
            self.child = None;
            self.running = false;
            self.push_output(format!("[退出: {code}]"));
        }
    }

    pub fn interrupt(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.start_kill();
        }
        self.running = false;
        self.child = None;
        self.push_output("[已中断]".to_string());
    }

    pub fn push_output(&mut self, line: String) {
        for l in line.lines() {
            self.output.push(l.to_string());
        }
        if self.output.len() > MAX_LINES {
            let excess = self.output.len() - MAX_LINES;
            self.output.drain(0..excess);
        }
    }

    /// 历史导航：向上。
    pub fn hist_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.hist_idx {
            None => self.history.len().saturating_sub(1),
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.hist_idx = Some(idx);
        self.input = self.history[idx].clone();
    }

    /// 历史导航：向下。
    pub fn hist_down(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.hist_idx {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.hist_idx = None;
                self.input.clear();
            }
            Some(i) => {
                self.hist_idx = Some(i + 1);
                self.input = self.history[i + 1].clone();
            }
        }
    }
}

async fn read_stream<R>(r: R, tx: UnboundedSender<Action>)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(r).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let _ = tx.send(Action::CmdOutput { data: line });
    }
}
