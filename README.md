# 🚀 OneShell

OneShell 是一个面向 macOS 的新一代终端与远程工作台，目标不是“再做一个能 SSH 的壳”，而是要在终端渲染、Emoji 显示、代理连接、SFTP、脚本联动和本地高级功能上，做出一套比 FinalShell 更现代、更稳定、更适合 macOS 的产品。

这次仓库已经完成一次**彻底清库重建**：

- ✅ 旧的混合 SwiftUI / 旧 runtime 历史已经切断
- ✅ GitHub 仓库本体已删除并按同名重建
- ✅ 本地 git 历史已重置为新的根提交
- ✅ 当前仓库只保留新的 `Tauri + Rust + React` 主线

## 🧭 产品定位

OneShell 要解决的核心问题：

- 😀 终端里 Emoji、中文、双宽字符不能错位、重叠、乱码
- 🌍 海外机器连接要原生支持 SOCKS5 代理
- 📂 SFTP 文件管理要稳定支持 UTF-8、中文、Emoji 文件名
- 🧪 本地脚本工作站要能直接联动远端终端
- 🔐 连接资料必须本地加密保存，不依赖云端
- 👑 后续高级功能要支持本地 license 校验

## 🧱 当前技术栈

- 🖥️ 桌面壳：`Tauri 2`
- 🦀 后端：`Rust`
- ⚛️ 前端：`React 18`
- 🎨 样式：`Tailwind CSS`
- 🧵 终端：`xterm.js`
- 🔐 数据库：`SQLite + SQLCipher`
- 🛡️ 密码字段加密：`AES-256-GCM`

## ✅ 当前已经落下来的东西

- ✅ 新的 Tauri 工程骨架
- ✅ React + Tailwind 前端骨架
- ✅ `src-tauri/tauri.conf.json` 基础桌面配置
- ✅ 本地 SQLCipher 数据库初始化
- ✅ 服务器资料的基础存储模型
- ✅ 本地 `master.key` 生成逻辑
- ✅ 密码字段 AES-GCM 二次加密
- ✅ `xterm.js` 终端表面组件
- ✅ `Unicode11` 宽度处理
- ✅ `WebGL -> canvas` 渲染回退策略
- ✅ macOS 字体回退顺序：
  `JetBrainsMono Nerd Font -> Apple Color Emoji -> Menlo -> monospace`
- ✅ macOS 通用构建链路骨架（`Intel + Apple Silicon`）

## 🎯 当前主线怎么推

当前只认这条新主线，后续 agent 不要再把仓库往旧 SwiftUI / 旧 Rust Core 的方向拉回去。

优先顺序：

1. 🖥️ 终端主渲染链路继续打磨
2. 🔌 Rust SSH 连接管理器与 Tauri 事件桥
3. 🌍 SOCKS5 代理连接
4. 🧰 脚本工作站
5. 📂 SFTP 文件管理
6. 👑 本地 license 与高级功能门禁

## 📁 当前仓库结构

```text
oneshell/
├─ src/                      # React 前端
│  ├─ components/terminal/   # 终端组件
│  ├─ App.tsx
│  ├─ main.tsx
│  └─ index.css
├─ src-tauri/                # Tauri + Rust 后端
│  ├─ src/
│  │  ├─ commands/
│  │  ├─ modules/
│  │  ├─ app_state.rs
│  │  └─ lib.rs
│  ├─ Cargo.toml
│  └─ tauri.conf.json
├─ docs/
│  ├─ ARCHITECTURE.md
│  └─ ROADMAP.md
├─ package.json
└─ vite.config.ts
```

## 🧭 文档入口

- 📘 [AGENTS.md](./AGENTS.md)：给其他 agent 的统一作战说明
- 🏗️ [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)：系统架构、边界和模块分层
- 🗺️ [docs/ROADMAP.md](./docs/ROADMAP.md)：阶段路线图、优先级与交付口径
- 🔌 [docs/SSH_RUNTIME.md](./docs/SSH_RUNTIME.md)：SSH 前后端事件桥与 command 契约
- 🍎 [docs/BUILD_MACOS_UNIVERSAL.md](./docs/BUILD_MACOS_UNIVERSAL.md)：Intel + M 芯片通用构建说明

## ⚠️ 硬性原则

- ❌ 不要恢复旧的 `core/`、`ui/`、旧 `.github/` 或旧 SwiftUI 代码线
- ❌ 不要再引入“混合双主线架构”
- ❌ 不要把旧仓库的历史包袱带回主线
- ✅ 所有新文档默认使用 `Emoji + 中文`
- ✅ 新 agent 进入仓库后，先看 `AGENTS.md`
- ✅ 所有实现必须围绕当前 Tauri 重建路线收口

## 🛠️ 本地开发

```bash
npm install
npm run tauri:dev
```

## 🍎 通用打包

```bash
npm run build:macos:universal
```

这条链路会生成支持：

- `arm64`
- `x86_64`

的通用 macOS `.app / .dmg`。

## 📝 说明

- 当前仓库已经是新的“纯净重建仓库”
- 删除前的老仓库历史，已在本地做只读 bundle 备份
- 后续所有推进，以这个新仓库为唯一主线
