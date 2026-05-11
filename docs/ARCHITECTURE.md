# OneShell Architecture

## Goal

Build a clean, high-performance macOS remote workbench around one consistent stack:

- frontend: React + Tailwind + xterm.js
- shell: Tauri 2
- backend: Rust
- storage: SQLCipher + AES-GCM

## Layers

### Frontend

`src/`

- desktop shell UI
- terminal surface
- connection manager
- script workstation
- SFTP browser
- license / feature gate views

### Tauri Bridge

`src-tauri/src/commands/`

- thin command handlers
- frontend-facing invoke contracts
- event emission for terminal and transfer streams

### Backend Modules

`src-tauri/src/modules/`

- `db.rs`: SQLCipher database bootstrap and CRUD
- `crypto.rs`: master key generation, key derivation, AES-GCM helpers
- `models.rs`: shared DTOs
- future `ssh.rs`: async SSH runtime
- future `sftp.rs`: async file operations
- future `scripts.rs`: local script scanning and execution
- future `license.rs`: local machine code and license validation

## Storage Strategy

Two layers protect secrets:

1. SQLCipher encrypts the full database file.
2. saved passwords are encrypted again with AES-256-GCM before storage.

The local `master.key` is generated on first launch and used to derive:

- the SQLCipher key
- the password field encryption key

## Terminal Strategy

The terminal rendering path is built around:

- `xterm.js`
- `xterm-addon-webgl`
- `xterm-addon-unicode11`

Key macOS requirements:

- emoji width must remain correct
- no glyph overlap on double-width characters
- graceful fallback from WebGL to canvas
- proper font fallback to `Apple Color Emoji`

## Runtime Direction

The next runtime milestone is a single SSH session pipeline with:

- password auth
- SOCKS5 proxy support
- keep-alive
- stdin / stdout event bridge
- PTY resize

After that, OneShell will expand into:

- multi-session registry
- SFTP runtime
- script workstation
- local license / premium gates
