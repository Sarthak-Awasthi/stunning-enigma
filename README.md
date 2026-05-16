# Mod Manager

A Linux-native Fallout 4 mod manager built as a Rust workspace with a Kirigami/QML frontend and a local IPC daemon backend.

## Current scope

- Game instance detection and registration (Steam/manual)
- Runner detection from Steam Proton, Steam `compatibilitytools.d`, and system Wine
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
SQLX_OFFLINE=true cargo test
cargo run -p mm-daemon
```

The daemon listens on a Unix socket and uses newline-delimited JSON-RPC.

### Run Kirigami UI

```bash
cmake -S ui/kirigami-app -B build/ui
cmake --build build/ui
./build/ui/mod-manager-ui
```

### Regenerate SQLx offline cache

From workspace root:

```bash
cargo sqlx database create
cargo sqlx migrate run --source crates/storage-sqlite/migrations
cargo sqlx prepare --workspace
```

## MVP self-test flow (real game install)

Use the UI sections in order; each step captures IDs needed by the next one.

1. **Link game install**: use **Detect via Steam** or enter path + **Register Path**.
2. **Create profile**: choose profile name, click **Create Profile**.
3. **Runner**: click **Refresh Runners**, pick one, click **Pin Runner**.
4. **Mod ingest**: enter archive path, click **Ingest Mod**, then **Save Profile Mod**.
5. **Deploy/plugins**: set `Data` directory, run **Deploy Apply**, then **Sync Plugins** and **Sort with LOOT**.
6. **First run setup**: use **Run Launcher (First Run)** without F4SE once so default config files are generated.
7. **Launch**: run **Preflight** and then **Launch Game** (enable F4SE only after first-run setup).

If any step fails, check the UI activity log and raw daemon response pane for the exact error.

## Notes

- SQLx offline cache is tracked in `.sqlx/` for offline builds.
- Runtime/local artifacts are ignored via `.gitignore`.
- Deploy planning now places known F4SE root files (like `f4se_loader.exe` and `f4se_*.dll`) in the game root while normal mod files still deploy under `Data/`.

## Authorship

This project was co-authored with AI assistance (GitHub Copilot).
