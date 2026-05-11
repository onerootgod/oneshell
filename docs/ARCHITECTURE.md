# 🏗️ OneShell 架构说明

## 🎯 总目标

OneShell 要做成一个围绕 macOS 体验设计的终端与远程工作台。

核心关键词：

- 😀 Emoji 正确渲染
- 🌍 代理友好
- 🔐 本地加密
- 📂 稳定 SFTP
- 🧰 脚本联动
- 👑 本地高级功能门禁

## 🧱 分层结构

### ⚛️ 前端层

目录：`src/`

职责：

- 桌面工作台 UI
- 终端主舞台
- 连接管理界面
- 脚本工作站界面
- SFTP 浏览界面
- license / 功能门禁展示

### 🔌 Tauri 命令桥

目录：`src-tauri/src/commands/`

职责：

- 暴露给前端的 invoke 接口
- 转发命令到 Rust 模块
- 维持轻量 command handler 边界
- 后续承接终端、SFTP、脚本工作站的事件桥

### 🦀 Rust 模块层

目录：`src-tauri/src/modules/`

当前模块：

- `db.rs`
  本地 SQLCipher 数据库初始化与基础 CRUD
- `crypto.rs`
  `master.key` 生成、密钥派生、AES-GCM 加密解密
- `models.rs`
  前后端共用 DTO

后续模块：

- `ssh.rs`
  SSH runtime、SOCKS5、keep-alive、PTY、stdin/stdout
- `sftp.rs`
  文件浏览、上传、下载、删除、文件树数据结构
- `scripts.rs`
  本地脚本目录扫描、执行与远端注入
- `license.rs`
  机器码、license 校验、高级功能解锁

## 🔐 存储与安全策略

OneShell 不是只做“存库”，而是做双层保护：

1. 🧱 数据库层：SQLCipher 加密整个 SQLite 文件
2. 🛡️ 字段层：服务器密码额外走 AES-256-GCM

流程：

- 应用第一次启动时生成本地 `master.key`
- 从 `master.key` 派生两类用途不同的 key：
  - SQLCipher 数据库 key
  - 密码字段加密 key

这样即使以后调整库结构，密钥边界也还能保持清晰。

## 🖥️ 终端渲染策略

终端渲染必须围绕 macOS 的痛点来设计。

当前固定方案：

- `xterm.js`
- `xterm-addon-unicode11`
- `xterm-addon-webgl`
- `xterm-addon-fit`

当前原则：

- 😀 Emoji 必须按双宽正确占位
- 🇨🇳 中文与 Emoji 混排不能挤压错位
- 🎮 WebGL 优先，context 丢失时自动回退 canvas
- 🍎 字体回退必须带 `Apple Color Emoji`

当前字体回退顺序：

`JetBrainsMono Nerd Font -> Apple Color Emoji -> Menlo -> monospace`

## 🌍 SSH / 代理方向

下一阶段的 SSH runtime，必须满足：

- 支持用户名 + 密码认证
- 支持 `SOCKS5` 代理拨号
- 支持 keep-alive
- 支持 PTY resize
- 支持终端 stdin / stdout 实时桥接

前端和 SSH runtime 的交互方式：

- 前端通过 Tauri command 发起连接、输入、resize
- Rust 后端通过事件把 stdout、stderr、lifecycle 推回前端

## 📂 SFTP 方向

SFTP 不是附属功能，而是 OneShell 的主工作流之一。

目标要求：

- 支持目录浏览
- 支持上传 / 下载 / 删除
- 支持 UTF-8、中文、Emoji 文件名
- 支持大文件异步传输
- 前端能收到标准化文件树 JSON

## 🧰 脚本工作站方向

脚本工作站是 OneShell 和普通终端工具拉开差距的重要部分。

目标：

- 扫描本地脚本目录
- 展示 `.py` / `.sh` 脚本
- 支持本地执行
- 支持注入到当前 SSH 会话执行

## 👑 本地高级功能方向

后续高级功能不会先依赖云服务，而是本地优先：

- 获取机器码
- 校验本地 license
- 解锁高级功能

预期高级功能包括：

- 多线程 SFTP
- 更多脚本存储能力
- 更多工作台高级能力

## 🚫 明确不做的事

当前架构里明确不再做：

- 不恢复旧 SwiftUI 主线
- 不恢复旧 `core/` 运行时
- 不再做双架构并存
- 不为了“保留旧历史”牺牲新的结构清晰度
