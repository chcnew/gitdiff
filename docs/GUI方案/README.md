# GitDiff 设计文档

本地 Windows 轻量 Git 图形客户端的详细设计说明。

## 文档索引

| 文档                               | 内容                                         |
| ---------------------------------- | -------------------------------------------- |
| [产品概述](./01-overview.md)       | 目标、范围、技术选型、约束                   |
| [整体架构](./02-architecture.md)   | 分层架构、进程模型、关键数据流               |
| [模块设计](./03-modules.md)        | 目录结构、前后端模块职责                     |
| [后端设计](./04-backend.md)        | Rust / Tauri Commands、Git、监听、终端、日志 |
| [前端设计](./05-frontend.md)       | Vue3 / Pinia、页面、组件、状态               |
| [界面与交互](./06-ui-ux.md)        | 布局、主题、Diff、文件树、终端               |
| [命令与事件 API](./07-api.md)      | invoke 命令、事件、数据结构                  |
| [构建与发布](./08-build-deploy.md) | 开发、打包、图标、产物路径                   |

## 快速摘要

- **产品名**：GitDiff `v0.1.0`
- **标识符**：`com.gitdiff.app`
- **技术栈**：Tauri 2 + Vue 3 + TypeScript + Pinia
- **Git 实现**：调用本机 `git` CLI（不内嵌 libgit2）
- **平台**：优先 Windows 桌面客户端

## 相关入口

- 工程说明：../README.md
- 前端入口：`src/main.ts` / `src/App.vue`
- 后端入口：`src-tauri/src/lib.rs`