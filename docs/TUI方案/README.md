# GitDiff TUI 设计文档

本地 Windows 轻量 Git 终端客户端（TUI）的详细设计说明，与 [GUI 方案](../GUI方案/README.md) 同源，面向纯 Rust + 终端交互场景。

## 文档索引

| 文档                                   | 内容                                         |
| -------------------------------------- | -------------------------------------------- |
| [gitdiff-tui.md](./gitdiff-tui.md)     | 完整设计：概述、架构、模块、界面、API、构建  |

## 快速摘要

- **产品名**：GitDiff TUI `v0.1.0`
- **标识符**：`com.gitdiff.tui`
- **技术栈**：Rust + ratatui + crossterm + tokio
- **Git 实现**：调用本机 `git` CLI（不内嵌 libgit2，复用系统凭据）
- **平台**：优先 Windows 终端（Windows Terminal / ConHost），兼容 macOS / Linux

## 与 GUI 方案的关系

| 维度       | GUI 方案                                | TUI 方案                                    |
| ---------- | --------------------------------------- | ------------------------------------------- |
| 运行时     | Tauri 2 + WebView                       | 单二进制 + 终端（ratatui）                  |
| 前端       | Vue 3 + TypeScript + Pinia              | Rust 组件 + `App` 状态机                    |
| 渲染       | DOM / CSS                               | 帧缓冲（`Frame` + `Buffer`）                |
| 输入       | 鼠标 + 键盘                             | 键盘优先（Vim 风格）+ 可选鼠标              |
| 异步       | Tauri 主进程 + 事件总线                  | tokio 任务 + mpsc 通道回投 Action           |
| 终端内嵌   | `portable-pty` + `@xterm/xterm`         | 命令运行器（v1）/ PTY + termwiz（扩展）     |

## 相关入口

- GUI 方案：../GUI方案/gitdiff-gui.md
- 工程说明：../README.md
- 入口文件（规划）：`src/main.rs` / `src/app.rs`
