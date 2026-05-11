# OneShell

OneShell is a clean-slate rebuild of a modern terminal and remote workbench for macOS.

This repository has been reset to remove the previous mixed SwiftUI / legacy runtime history and now focuses on a single direction:

- `Tauri 2`
- `Rust`
- `React 18`
- `Tailwind CSS`
- `xterm.js`
- `SQLCipher`

## Product Direction

OneShell is being rebuilt to exceed FinalShell in the areas that matter most on macOS:

- cleaner terminal rendering
- correct emoji width and glyph fallback
- strong local encrypted storage
- SSH + SOCKS5 proxy support
- SFTP with UTF-8 / emoji-safe filenames
- local script workstation
- local-first license / premium feature gating

## Current Rebuild Status

This repository is intentionally clean and early.

Already in place:

- Tauri desktop shell configuration
- React + Tailwind frontend foundation
- macOS-oriented terminal surface prototype
- xterm.js + WebGL fallback + Unicode11 emoji width handling
- SQLCipher-backed local database bootstrap
- AES-GCM field encryption for saved server passwords

Planned next:

1. Rust SSH session manager with SOCKS5 proxy support
2. Tauri event bridge for terminal stdin / stdout
3. script workstation scanning and execution
4. async SFTP runtime and file tree API
5. local license validation and feature gates

## Repository Layout

```text
oneshell/
├─ src/                  # React frontend
├─ src-tauri/            # Tauri + Rust backend
├─ docs/                 # rebuilt documentation
├─ package.json
└─ src-tauri/tauri.conf.json
```

## Local Development

```bash
npm install
npm run tauri:dev
```

## Documentation

- [Architecture](./docs/ARCHITECTURE.md)
- [Roadmap](./docs/ROADMAP.md)

## Notes

- The previous repository history was intentionally discarded to avoid architecture pollution.
- A local read-only bundle backup was created before the reset.
- This repo is now the canonical clean rebuild branch and history.
