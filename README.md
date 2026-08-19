# GitDiff TUI

本地轻量 Git 终端客户端，纯 Rust 实现，键盘优先、启动快、复用系统 Git 凭据。

> 与 [GUI 版](./docs/GUI方案/gitdiff-gui.md) 同源，面向终端环境，基于 [ratatui](https://ratatui.rs/) + [crossterm](https://github.com/crossterm-rs/crossterm)。

## 功能

- 打开仓库 / 最近项目（本地 JSON 持久化，最多 20 条）
- 变更列表：staged / unstaged / untracked，暂存、取消暂存、全部暂存、提交
- Diff：左右对比 + 统一视图，增删改着色，差异点跳转（`n`/`N`）
- 历史：最近 100 条提交，提交文件列表，中栏查看 patch
- 分支：列表、切换、创建
- 文件树：浏览、展开目录、编辑保存
- 远程：push / pull（复用系统凭据助手 / SSH agent）
- 终端：底部命令运行器（`cmd /c` / `sh -c`），输出回显、命令历史、`Ctrl+C` 中断
- 鼠标：点击聚焦面板、点击选中列表项、滚轮滚动
- 文件监听：工作区变更 900ms 防抖后自动刷新变更列表

## 环境要求

- Rust（stable）
- 本机 `git` 可用（在 `PATH` 中）
- 支持 ANSI/VT 的终端（Windows 推荐 Windows Terminal）

## 构建与运行

```bash
# 直接运行
cargo run

# 直接打开指定仓库
cargo run -- -C <repo-path>

# 调试日志（debug 级别）
cargo run -- -v

# 发布构建
cargo build --release
```

产物为单二进制 `target/release/gitdiff-tui.exe`，复制即可运行。

## 快捷键

### 全局 / 欢迎页

| 键 | 动作 |
| --- | --- |
| `o` | 打开仓库（输入路径） |
| `Enter` | 打开选中的最近项目 |
| `d` / `x` | 移除最近项目 |
| `?` | 帮助 |
| `q` / `Ctrl+C` | 退出 |

### 工作区

| 键 | 动作 |
| --- | --- |
| `Tab` / `Shift+Tab` | 切换焦点面板 |
| `1` `2` `3` `4` | 焦点直达：左 / 中 / 右 / 终端 |
| `[` / `]` | 左栏切换：变更 / 文件 |
| `b` | 分支面板 |
| `P` / `F` | 推送 / 拉取 |
| `t` | 折叠/展开终端 |
| `r` | 刷新 |
| `x` | 关闭仓库（返回欢迎页） |

### 变更列表（左栏）

| 键 | 动作 |
| --- | --- |
| `j` / `k`（或方向键） | 移动 |
| `space` | 暂存 / 取消暂存 |
| `u` | 取消暂存 |
| `a` | 全部暂存 |
| `c` | 提交 |
| `Enter` | 查看差异 |

### 差异（中栏）

| 键 | 动作 |
| --- | --- |
| `v` | 左右视图 ↔ 统一视图 |
| `n` / `N` | 上一个/下一个差异点 |
| `j` / `k` | 滚动 |

### 文件树 / 编辑

| 键 | 动作 |
| --- | --- |
| `Enter` / `l` | 展开目录 / 打开文件 |
| `h` | 收起目录 |
| `e` | 编辑文件 |
| `Ctrl+S` | 保存 |
| `Esc` | 退出编辑 |

### 终端

| 键 | 动作 |
| --- | --- |
| `i` / `Enter` | 聚焦输入 |
| `↑` / `↓` | 命令历史 |
| `Ctrl+C` | 中断运行中的命令 |

## 目录结构

```text
src/
├── main.rs          # 入口：clap 参数 + 启动
├── tui.rs           # 终端初始化 + 主循环（poll/tick + Action 回投）
├── app.rs           # App 状态机 + update(Action) 归约 + 鼠标命中
├── event.rs         # Action 定义
├── git/             # git CLI 薄封装（run_git）与 porcelain 解析
├── diff.rs          # side-by-side 行对齐算法（similar）
├── watcher.rs       # 文件监听 + 900ms 防抖
├── recent.rs        # 最近项目持久化
├── terminal.rs      # 命令运行器
├── log.rs           # 文件日志（tracing）
└── ui/              # 各面板渲染（纯函数，只读 App）
```

## 设计文档

- [TUI 方案](./docs/TUI方案/gitdiff-tui.md)
- [GUI 方案](./docs/GUI方案/gitdiff-gui.md)

## 运行时数据位置

| 数据 | 位置 |
| --- | --- |
| 最近项目 | `<data_dir>/gitdiff-tui/recent-projects.json` |
| 文件日志 | `<exe_dir>/logs/gitdiff.log`（失败回退 `<data_dir>/GitDiffTUI/logs/`） |

## 已知限制

- 首版无冲突可视化解决；超大文件 Diff 会截断显示行数
- 命令运行器为 `cmd /c` / `sh -c`，非完整 PTY 终端模拟（交互式 TUI 程序在其中不可用）
- 编辑器为简版行编辑，复杂编辑建议用外部 `$EDITOR`
