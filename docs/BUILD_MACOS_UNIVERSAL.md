# 🍎 macOS 通用构建

## 🎯 目标

OneShell 的 macOS 交付必须同时覆盖：

- `Apple Silicon (arm64)`
- `Intel (x86_64)`

当前仓库统一采用 **Universal Binary** 路线，而不是分发两套不同安装包。

## ✅ 当前工程约束

- 最低支持版本：`macOS 12.0`
- Tauri 构建目标：`universal-apple-darwin`
- 交付物：`.app + .dmg`

## 🛠️ 本地构建

```bash
npm install
npm run build:macos:universal
```

这个脚本会自动：

1. 安装 `aarch64-apple-darwin` 与 `x86_64-apple-darwin`
2. 执行 `tauri build --target universal-apple-darwin`
3. 校验 `.app` 是否同时包含 `arm64 + x86_64`
4. 校验 `LSMinimumSystemVersion` 是否仍是 `12.0`

## ✅ 本地校验

```bash
npm run verify:macos:universal -- /path/to/OneShell.app
```

## 🤖 GitHub Actions

仓库已经内置：

- `.github/workflows/macos-universal.yml`

它会在 macOS runner 上：

- 安装 Node
- 安装 Rust 双架构 target
- 构建通用 `.app + .dmg`
- 校验通用二进制
- 上传构建产物

## 📌 注意

- Universal Binary 体积会比单架构更大，这是正常现象。
- 如果后面接入签名 / notarization，要在这条通用构建链上继续补，不要拆回双包路线。
