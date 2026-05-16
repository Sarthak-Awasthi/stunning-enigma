# Mod Manager

A Linux-native Fallout 4 mod manager built as a Rust workspace with a Kirigami/QML frontend and a local IPC daemon backend.

## Current scope

- Game instance detection and registration (Steam/manual)
- Profile creation with isolated profile files
- Mod archive ingestion, indexing, wrapper-folder normalization
- FOMOD install path resolution via `fomod-oxide`
- Conflict resolution, deploy planning, symlink deploy/rollback
- Plugin sync, load order persistence, master dependency validation
- LOOT sorting through `libloot` with deterministic fallback
- Launch preflight and game launch through pinned runner (Proton/Wine)
- Local IPC API and a Kirigami UI shell for request/response workflows

## Workspace layout

```text
crates/
  domain-core/
  storage-sqlite/
  game-detect/
  runner-manager/
  mod-ingest/
  deploy-engine/
  plugins-engine/
  loot-engine/
  launch-engine/
  ipc-api/
  mm-daemon/
ui/
  kirigami-app/
```

## Build and run

```bash
cargo build
cargo test
cargo run -p mm-daemon
```

The daemon listens on a Unix socket and uses newline-delimited JSON-RPC.

## Notes

- SQLx offline cache is tracked in `.sqlx/` for offline builds.
- Runtime/local artifacts are ignored via `.gitignore`.

## Authorship

This project was co-authored with AI assistance (GitHub Copilot).
