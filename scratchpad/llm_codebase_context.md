# Codebase Context

> This document contains the compiled source code and documentation for a project utilizing a Rust backend, C++, Kirigami-UI frontend, and an SQLite database.

## File: `Cargo.toml`
````toml
[workspace]
resolver = "2"
members = [
    "crates/domain-core",
    "crates/storage-sqlite",
    "crates/game-detect",
    "crates/runner-manager",
    "crates/mod-ingest",
    "crates/deploy-engine",
    "crates/plugins-engine",
    "crates/loot-engine",
    "crates/launch-engine",
    "crates/ipc-api",
    "crates/mm-daemon",
]

[workspace.dependencies]
tokio              = { version = "1", features = ["full"] }
serde              = { version = "1", features = ["derive"] }
serde_json         = "1"
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow             = "1"
thiserror          = "1"
sqlx               = { version = "0.8", features = ["sqlite", "runtime-tokio-native-tls", "migrate"] }
keyvalues-parser   = "0.2"
sha2               = "0.11"
hex                = "0.4"
sevenz-rust2       = "0.21"
zip                = "8"
rar                = "0.4"
walkdir            = "2"
fomod-oxide        = "0.1.0"
libloot            = "0.29.5"

domain-core        = { path = "crates/domain-core" }
storage-sqlite     = { path = "crates/storage-sqlite" }
game-detect        = { path = "crates/game-detect" }
runner-manager     = { path = "crates/runner-manager" }
mod-ingest         = { path = "crates/mod-ingest" }
deploy-engine      = { path = "crates/deploy-engine" }
plugins-engine     = { path = "crates/plugins-engine" }
loot-engine        = { path = "crates/loot-engine" }
launch-engine      = { path = "crates/launch-engine" }
ipc-api            = { path = "crates/ipc-api" }
````

## File: `README.md`
````markdown
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
````

## File: `Roadmap.md`
````markdown
# Project Roadmap: Linux Native Mod Manager

This roadmap outlines the path from the current working MVP to a fully featured, Mod Organizer 2 (MO2) equivalent mod manager built natively for Linux using Rust and Kirigami/QML.

## 🏁 Milestone 1: UI Foundation & Basic Workflows
*Focus: Upgrading the QML UI to be highly interactive and matching the MO2 top-bar layout.*

- [ ] **Top Toolbar Overhaul:** 
  - Add a global "Install Mod" button.
  - Add a visual Profile selector.
  - Add a "Shortcuts" dropdown to configure and launch auxiliary tools (xEdit, WryeBash, Bodyslide, etc.).
- [ ] **Runner & Launch Options:**
  - Enhance the Runner dropdown to include a "Settings" button.
  - Allow defining custom environment variables per profile/runner (e.g., `WINEDLLOVERRIDES`, `PROTON_LOG=1`).
- [ ] **Drag-and-Drop Installation:** 
  - Implement a QML `DropArea` so users can drag `.zip` and `.7z` files directly from their file manager into the app to trigger extraction and ingestion.

## 📦 Milestone 2: Archive & Plugin Expansions
*Focus: Expanding format support and properly handling base game files.*

- [ ] **RAR Archive Support:** 
  - Integrate a Rust crate (e.g., `unrar` or `compress-tools`) into `mod-ingest/archive.rs` to support extracting `.rar` mod files.
- [ ] **Base Game Plugins Visibility:** 
  - Auto-detect `Fallout4.esm` and official DLCs.
  - Pin them to the top of the Plugin Load Order (indexes 0, 1, 2...).
  - Prevent the user from unchecking, deleting, or sorting base game files.

## 🗂️ Milestone 3: Virtual File System (VFS) Visibility
*Focus: Letting the user see exactly what the symlink deployer is doing.*

- [ ] **VFS Backend API:** 
  - Create an IPC endpoint that reads the current `deploy_manifest` and constructs a hierarchical JSON tree of deployed files.
- [ ] **Data Tab (Tree View):** 
  - Implement a `TreeView` in the right pane's "Data" tab.
  - Display the VFS tree, showing which mod provides which specific file (e.g., `Textures/weapons/gun.dds -> [Weapon Mod A]`).

## ⚡ Milestone 4: Advanced List Management (MO2 Parity)
*Focus: Visual reordering, conflict resolution, and organization.*

- [ ] **Drag-and-Drop Priority (Left Pane):** 
  - Allow users to drag rows to change mod priority dynamically.
- [ ] **Drag-and-Drop Load Order (Right Pane):** 
  - Allow users to manually drag plugins to override LOOT sorting.
- [ ] **Conflict Visualization:** 
  - Add MO2's lightning bolt icons (`+` overwrites, `-` overwritten, `+/-` mixed) to the mod list.
  - Populate the bottom-right conflict detail pane when a mod is clicked.
- [ ] **Categories & Filters:** 
  - Add a database column for categories.
  - Add a bottom search bar to filter the mod list by name or category.
- [ ] **Separators:** 
  - Allow users to right-click and create "Dummy Mods" to act as visual separators in the priority list.

## 🛠️ Milestone 5: Tools & Integration
*Focus: Built-in utilities for serious modders.*

- [ ] **INI Editor:** 
  - A dedicated tab/window to safely edit the profile-specific `fallout4.ini`, `fallout4prefs.ini`, and `fallout4custom.ini`.
- [ ] **Save Game Manager:** 
  - A tab next to Plugins/Data to view local save files.
  - Cross-reference save files with the active load order and flag saves that are missing required plugins.
- [ ] **Downloads Tab:** 
  - Manage a designated downloads directory.
  - Double-click to install downloaded archives.
- [ ] **Nexus Integration (Future):** 
  - Store Nexus Mod IDs and versions.
  - Handle `nxm://` links from the browser.
  - Display "Update Available" warnings.

## 🐧 Milestone 6: Linux Specifics & Polish
*Focus: Stability, feedback, and debugging on Proton/Wine.*

- [ ] **Proton/Wine Console Log Window:** 
  - Add a dockable window or tab that captures and streams the `stdout/stderr` of the launched game process in real-time for debugging F4SE/ENB crashes.
- [ ] **Global Progress / Status Bar:** 
  - Add a bottom status bar with a loading spinner/progress bar so the UI doesn't look frozen during heavy I/O tasks (hashing, extracting).
- [ ] **Onboarding & Instance Management:** 
  - Replace the hardcoded `instance_id` logic. 
  - Show a welcome screen for first-time users to detect or manually locate their game installation.
````

## File: `crates/mm-daemon/Cargo.toml`
````toml
[package]
name    = "mm-daemon"
version = "0.1.0"
edition = "2024"

[dependencies]
domain-core        = { workspace = true }
storage-sqlite     = { workspace = true }
game-detect        = { workspace = true }
runner-manager     = { workspace = true }
mod-ingest         = { workspace = true }
deploy-engine      = { workspace = true }
ipc-api            = { workspace = true }
plugins-engine     = { workspace = true }
loot-engine        = { workspace = true }
launch-engine      = { workspace = true }
tokio              = { workspace = true }
tracing            = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow             = { workspace = true }
serde_json         = { workspace = true }
serde              = { workspace = true }
````

## File: `crates/mm-daemon/src/main.rs`
````rust
mod profile_fs;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use deploy_engine::{apply_plan, build_plan, rollback};
use game_detect::{SteamDetector, validate_fo4_path};
use ipc_api::{ErrorCode, ProfileInfo, ProfileModInfo, Request, Response, RunnerInfo};
use launch_engine::{launch_game, launch_launcher, preflight_launch};
use loot_engine::sort_profile_plugins;
use mod_ingest::ingest_mod;
use plugins_engine::{sync_plugins, validate_masters, write_load_order};
use profile_fs::profile_dir;
use runner_manager::{detect_runners, list_runners, pin_profile_runner};
use storage_sqlite::{
    Db, instance_repo::InstanceRepo, profile_mod_repo::ProfileModRepo, profile_repo::ProfileRepo, profile_plugin_repo::ProfilePluginRepo,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
};
use tracing::{error, info, warn};

const SOCKET_PATH: &str = "/tmp/mm-daemon.sock";

#[derive(Clone)]
struct AppState {
    db: Db,
    data_dir: Arc<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .init();

    info!(version = env!("CARGO_PKG_VERSION"), "mm-daemon starting");

    let data_dir = exe_dir()?;
    std::fs::create_dir_all(&data_dir)?;

    let db = Db::open(&data_dir.join("state.db")).await?;
    info!("database ready");

    let state = AppState {
        db,
        data_dir: Arc::new(data_dir),
    };

    let socket_path = Path::new(SOCKET_PATH);
    if socket_path.exists() {
        std::fs::remove_file(socket_path).context("failed to remove stale socket")?;
    }

    let listener = UnixListener::bind(socket_path).context("failed to bind Unix socket")?;
    info!(path = SOCKET_PATH, "IPC socket listening");

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                error!(err = %e, "connection handler failed");
            }
        });
    }
}

async fn handle_connection(stream: tokio::net::UnixStream, state: AppState) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let response = dispatch(&line, &state).await;

        let mut encoded = serde_json::to_string(&response)?;
        encoded.push('\n');
        writer.write_all(encoded.as_bytes()).await?;
    }

    info!("client disconnected");
    Ok(())
}

async fn dispatch(line: &str, state: &AppState) -> Response {
    let request = match serde_json::from_str::<Request>(line) {
        Ok(r) => r,
        Err(e) => {
            warn!(err = %e, raw = %line, "invalid request");
            return Response::Error {
                code: ErrorCode::InvalidRequest,
                message: e.to_string(),
            };
        }
    };

    match request {
        Request::Ping => {
            info!("Ping");
            Response::Pong {
                version: env!("CARGO_PKG_VERSION").to_string(),
            }
        }

        Request::DetectGame => {
            info!("DetectGame");
            match SteamDetector::detect() {
                Ok(path) => match validate_fo4_path(&path) {
                    Ok(()) => {
                        let install_path = path.display().to_string();
                        match upsert_instance(state, &install_path, "steam", "Fallout 4 (Steam)")
                            .await
                        {
                            Ok(instance_id) => Response::GameDetected {
                                instance_id,
                                install_path,
                                source: "steam".to_string(),
                            },
                            Err(e) => err(e),
                        }
                    }
                    Err(e) => err(e),
                },
                Err(e) => err(e),
            }
        }

        Request::RegisterGame { path } => {
            info!(path = %path, "RegisterGame");
            let p = PathBuf::from(&path);
            match validate_fo4_path(&p) {
                Ok(()) => match upsert_instance(state, &path, "manual", "Fallout 4 (Manual)").await
                {
                    Ok(instance_id) => Response::GameDetected {
                        instance_id,
                        install_path: path,
                        source: "manual".to_string(),
                    },
                    Err(e) => err(e),
                },
                Err(e) => err(e),
            }
        }

        Request::ListRunners => {
            info!("ListRunners");
            match detect_runners(&state.db).await {
                Ok(()) => match list_runners(&state.db).await {
                    Ok(runners) => Response::RunnerList {
                        runners: runners
                            .into_iter()
                            .map(|r| RunnerInfo {
                                id: r.id,
                                kind: runner_kind_to_str(&r.kind).to_string(),
                                version: r.version,
                                install_path: r.install_path,
                                verified: r.verified,
                            })
                            .collect(),
                    },
                    Err(e) => err(e),
                },
                Err(e) => err(e),
            }
        }

        Request::PinRunner {
            profile_id,
            runner_id,
        } => {
            info!(profile_id, runner_id, "PinRunner");
            match pin_profile_runner(profile_id, runner_id, &state.db).await {
                Ok(()) => Response::RunnerPinned {
                    profile_id,
                    runner_id,
                },
                Err(e) => err(e),
            }
        }

        Request::ListProfileMods { profile_id } => {
            info!(profile_id, "ListProfileMods");
            let repo = ProfileModRepo::new(&state.db.pool);
            match repo.list(profile_id).await {
                Ok(rows) => Response::ProfileMods {
                    profile_id,
                    mods: rows
                        .into_iter()
                        .map(|m| ProfileModInfo {
                            mod_id: m.mod_id,
                            mod_name: m.mod_name,
                            enabled: m.enabled,
                            priority: m.priority,
                        })
                        .collect(),
                },
                Err(e) => err(e),
            }
        }

        Request::UpsertProfileMod {
            profile_id,
            mod_id,
            enabled,
            priority,
        } => {
            info!(profile_id, mod_id, enabled, priority, "UpsertProfileMod");
            let repo = ProfileModRepo::new(&state.db.pool);
            match repo.upsert(profile_id, mod_id, enabled, priority).await {
                Ok(()) => Response::ProfileModUpdated { profile_id, mod_id },
                Err(e) => err(e),
            }
        }

        Request::SetProfileModEnabled {
            profile_id,
            mod_id,
            enabled,
        } => {
            info!(profile_id, mod_id, enabled, "SetProfileModEnabled");
            let repo = ProfileModRepo::new(&state.db.pool);
            match repo.set_enabled(profile_id, mod_id, enabled).await {
                Ok(()) => Response::ProfileModUpdated { profile_id, mod_id },
                Err(e) => err(e),
            }
        }

        Request::SetProfileModPriority {
            profile_id,
            mod_id,
            priority,
        } => {
            info!(profile_id, mod_id, priority, "SetProfileModPriority");
            let repo = ProfileModRepo::new(&state.db.pool);
            match repo.set_priority(profile_id, mod_id, priority).await {
                Ok(()) => Response::ProfileModUpdated { profile_id, mod_id },
                Err(e) => err(e),
            }
        }

        Request::CreateProfile { instance_id, name } => {
            info!(instance_id, name = %name, "CreateProfile");
            let repo = ProfileRepo::new(&state.db.pool);
            match repo.create(instance_id, &name).await {
                Ok(id) => match profile_fs::ensure_profile_dir(&state.data_dir, id).await {
                    Ok(_) => Response::ProfileCreated { profile_id: id },
                    Err(e) => err(e),
                },
                Err(e) => err(e),
            }
        }

        Request::ListProfiles { instance_id } => {
            info!(instance_id, "ListProfiles");
            let repo = ProfileRepo::new(&state.db.pool);
            match repo.list(instance_id).await {
                Ok(profiles) => Response::ProfileList {
                    profiles: profiles
                        .into_iter()
                        .map(|p| ProfileInfo {
                            id: p.id,
                            name: p.name,
                            auto_deploy: p.auto_deploy,
                        })
                        .collect(),
                },
                Err(e) => err(e),
            }
        }

        Request::DeleteProfile { profile_id } => {
            info!(profile_id, "DeleteProfile");
            let repo = ProfileRepo::new(&state.db.pool);
            match repo.delete(profile_id).await {
                Ok(()) => Response::ProfileDeleted { profile_id },
                Err(e) => err(e),
            }
        }

        Request::IngestMod { archive_path } => {
            info!(path = %archive_path, "IngestMod");
            let mods_dir = state.data_dir.join("mods");
            match ingest_mod(Path::new(&archive_path), &mods_dir, &state.db).await {
                Ok(r) => Response::ModIngested {
                    mod_id: r.mod_id,
                    name: r.name,
                    file_count: r.file_count,
                },
                Err(e) => err(e),
            }
        }

        Request::DeployPreview {
            profile_id,
            game_data_dir,
        } => {
            info!(profile_id, "DeployPreview");
            match build_plan(profile_id, Path::new(&game_data_dir), &state.db).await {
                Ok(plan) => {
                    let entries = plan
                        .entries
                        .iter()
                        .map(|e| format!("{} -> {}", e.source, e.target))
                        .collect();
                    Response::DeployPreview {
                        profile_id,
                        entry_count: plan.entries.len(),
                        entries,
                    }
                }
                Err(e) => err(e),
            }
        }

        Request::DeployApply {
            profile_id,
            game_data_dir,
        } => {
            info!(profile_id, "DeployApply");
            match build_plan(profile_id, Path::new(&game_data_dir), &state.db).await {
                Ok(plan) => match apply_plan(plan, &state.db).await {
                    Ok(manifest_id) => Response::DeployApplied { manifest_id },
                    Err(e) => err(e),
                },
                Err(e) => err(e),
            }
        }

        Request::DeployRollback { manifest_id } => {
            info!(manifest_id, "DeployRollback");
            match rollback(manifest_id, &state.db).await {
                Ok(()) => Response::RolledBack { manifest_id },
                Err(e) => err(e),
            }
        }

        Request::SyncPlugins {
            profile_id,
            data_dir,
        } => {
            info!(profile_id, "SyncPlugins");
            match sync_plugins(profile_id, Path::new(&data_dir), &state.db).await {
                Ok(added) => Response::PluginsSynced { added },
                Err(e) => err(e),
            }
        }

        Request::ValidatePlugins { profile_id } => {
            info!(profile_id, "ValidatePlugins");
            match validate_masters(profile_id, &state.db).await {
                Ok(report) => Response::PluginsValid {
                    missing_masters: report
                        .missing_masters
                        .iter()
                        .map(|m| format!("{} requires {}", m.plugin, m.missing_master))
                        .collect(),
                },
                Err(e) => err(e),
            }
        }

        Request::SortWithLoot { profile_id } => {
            info!(profile_id, "SortWithLoot");
            match sort_profile_plugins(profile_id, &state.db).await {
                Ok(result) => {
                    let dir = profile_dir(&state.data_dir, profile_id);
                    match write_load_order(profile_id, &dir, &state.db).await {
                        Ok(()) => Response::PluginsSorted {
                            profile_id,
                            engine: result.engine,
                            order: result.order,
                        },
                        Err(e) => err(e),
                    }
                }
                Err(e) => err(e),
            }
        }

        Request::LaunchPreflight {
            profile_id,
            use_f4se,
        } => {
            info!(profile_id, use_f4se, "LaunchPreflight");
            match preflight_launch(profile_id, use_f4se, &state.db).await {
                Ok(report) => Response::LaunchPreflight {
                    profile_id: report.profile_id,
                    runner_kind: report.runner_kind,
                    game_install_path: report.game_install_path,
                    f4se_available: report.f4se_available,
                },
                Err(e) => err(e),
            }
        }

        Request::LaunchGame {
            profile_id,
            use_f4se,
        } => {
            info!(profile_id, use_f4se, "LaunchGame");
            match launch_game(profile_id, use_f4se, &state.db).await {
                Ok(result) => Response::GameLaunched {
                    profile_id: result.profile_id,
                    runner_kind: result.runner_kind,
                    executable: result.executable,
                    pid: result.pid,
                },
                Err(e) => err(e),
            }
        }

        Request::LaunchLauncher { profile_id } => {
            info!(profile_id, "LaunchLauncher");
            match launch_launcher(profile_id, &state.db).await {
                Ok(result) => Response::GameLaunched {
                    profile_id: result.profile_id,
                    runner_kind: result.runner_kind,
                    executable: result.executable,
                    pid: result.pid,
                },
                Err(e) => err(e),
            }
        }

        Request::WriteLoadOrder { profile_id } => {
            info!(profile_id, "WriteLoadOrder");
            let dir = profile_dir(&state.data_dir, profile_id);
            match write_load_order(profile_id, &dir, &state.db).await {
                Ok(()) => Response::LoadOrderWritten,
                Err(e) => err(e),
            }
        }

        Request::ListProfilePlugins { profile_id } => {
                    info!(profile_id, "ListProfilePlugins");
                    let repo = ProfilePluginRepo::new(&state.db.pool);
                    match repo.list(profile_id).await {
                        Ok(plugins) => Response::ProfilePlugins { profile_id, plugins },
                        Err(e) => err(e),
                    }
                }
        
                Request::SetProfilePluginEnabled { profile_id, plugin_id, enabled } => {
                    info!(profile_id, plugin_id, enabled, "SetProfilePluginEnabled");
                    let repo = ProfilePluginRepo::new(&state.db.pool);
                    match repo.set_enabled(profile_id, plugin_id, enabled).await {
                        Ok(()) => Response::ProfilePluginUpdated { profile_id, plugin_id },
                        Err(e) => err(e),
                    }
                }
    }
}

fn err(e: impl std::fmt::Display) -> Response {
    Response::Error {
        code: ErrorCode::Internal,
        message: e.to_string(),
    }
}

async fn upsert_instance(
    state: &AppState,
    install_path: &str,
    source_type: &str,
    label: &str,
) -> anyhow::Result<i64> {
    let repo = InstanceRepo::new(&state.db.pool);
    repo.upsert_fallout4_instance(install_path, source_type, label)
        .await
}

fn runner_kind_to_str(kind: &domain_core::entities::RunnerKind) -> &'static str {
    match kind {
        domain_core::entities::RunnerKind::Proton => "proton",
        domain_core::entities::RunnerKind::Wine => "wine",
    }
}

fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe().context("failed to determine executable path")?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .context("executable has no parent directory")
}
````

## File: `crates/mm-daemon/src/profile_fs.rs`
````rust
use std::path::{Path, PathBuf};

use anyhow::Context;
use tracing::info;

/// Default INI content — minimal valid files so FO4 doesn't complain.
const DEFAULT_FALLOUT4_INI: &str = "[General]\n";
const DEFAULT_PREFS_INI: &str = "[Display]\n";
const DEFAULT_CUSTOM_INI: &str = "[General]\n";

/// Materialise the on-disk directory structure for a profile if it
/// doesn't already exist.
pub async fn ensure_profile_dir(data_dir: &Path, profile_id: i64) -> anyhow::Result<PathBuf> {
    let profile_dir = data_dir.join("profiles").join(profile_id.to_string());

    tokio::fs::create_dir_all(&profile_dir)
        .await
        .context("failed to create profile directory")?;

    // Write default files only if they don't exist yet
    write_if_absent(&profile_dir.join("plugins.txt"), "").await?;
    write_if_absent(&profile_dir.join("loadorder.txt"), "").await?;
    write_if_absent(&profile_dir.join("fallout4.ini"), DEFAULT_FALLOUT4_INI).await?;
    write_if_absent(&profile_dir.join("fallout4prefs.ini"), DEFAULT_PREFS_INI).await?;
    write_if_absent(&profile_dir.join("fallout4custom.ini"), DEFAULT_CUSTOM_INI).await?;

    info!(profile_id, path = %profile_dir.display(), "profile directory ready");
    Ok(profile_dir)
}

/// Return the path to a profile's directory without creating it.
pub fn profile_dir(data_dir: &Path, profile_id: i64) -> PathBuf {
    data_dir.join("profiles").join(profile_id.to_string())
}

async fn write_if_absent(path: &Path, content: &str) -> anyhow::Result<()> {
    if !path.exists() {
        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}
````

## File: `crates/domain-core/Cargo.toml`
````toml
[package]
name = "domain-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde     = { workspace = true }
thiserror = { workspace = true }
````

## File: `crates/domain-core/src/lib.rs`
````rust
pub mod entities;
pub mod error;
````

## File: `crates/domain-core/src/entities.rs`
````rust
use serde::{Deserialize, Serialize};

// ── Game ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: i64,
    pub name: String,
    pub canonical_id: String, // "fallout4"
}

// ── Instance ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: i64,
    pub game_id: i64,
    pub label: String,
    pub source_type: SourceType,
    pub install_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Steam,
    Gog,
    Manual,
}

// ── Profile ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub instance_id: i64,
    pub name: String,
    pub pinned_runner_id: Option<i64>,
    pub auto_deploy: bool,
}

// ── Mod ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: i64,
    pub name: String,
    pub version: Option<String>,
    pub source_hash: String, // SHA-256 of the original archive
    pub install_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMod {
    pub profile_id: i64,
    pub mod_id: i64,
    pub enabled: bool,
    pub priority: i32, // lower = lower priority; higher wins conflicts
}

// ── Plugin ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: i64,
    pub mod_id: Option<i64>, // None = loose / unmanaged plugin
    pub filename: String,
    pub kind: PluginKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Esm, // master file
    Esp, // regular plugin
    Esl, // light plugin
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePlugin {
    pub profile_id: i64,
    pub plugin_id: i64,
    pub enabled: bool,
    pub load_index: i32,
}

// ── Runner ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runner {
    pub id: i64,
    pub kind: RunnerKind,
    pub version: String,
    pub source_url: Option<String>,
    pub install_path: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerKind {
    Proton,
    Wine,
}

// ── Deploy manifest ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployManifest {
    pub id: i64,
    pub profile_id: i64,
    pub symlink_plan: Vec<SymlinkEntry>, // decoded from the JSON blob
    pub status: DeployStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkEntry {
    pub source: String, // absolute path inside mod's install dir
    pub target: String, // absolute path inside game's Data dir
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeployStatus {
    Active,
    RolledBack,
}
````

## File: `crates/domain-core/src/error.rs`
````rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("profile '{name}' already exists for this instance")]
    ProfileAlreadyExists { name: String },

    #[error("priority {priority} is already taken in profile {profile_id}")]
    PriorityConflict { profile_id: i64, priority: i32 },

    #[error("runner is not verified and cannot be pinned")]
    RunnerNotVerified,

    #[error("mod '{name}' has no priority assigned in this profile")]
    ModNotInProfile { name: String },

    #[error("load index {index} is already taken in profile {profile_id}")]
    LoadIndexConflict { profile_id: i64, index: i32 },

    #[error("install path does not exist or is not a directory: {path}")]
    InvalidInstallPath { path: String },
}
````

## File: `crates/storage-sqlite/Cargo.toml`
````toml
[package]
name = "storage-sqlite"
version = "0.1.0"
edition = "2024"

[dependencies]
sqlx        = { workspace = true }
anyhow      = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
domain-core = { workspace = true }
ipc-api     = { workspace = true }
````

## File: `crates/storage-sqlite/src/lib.rs`
````rust
pub mod instance_repo;
pub mod migrations;
pub mod profile_mod_repo;
pub mod profile_repo;
pub mod profile_plugin_repo;

use anyhow::Context;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::{path::Path, str::FromStr};
use tracing::info;

/// The main handle to the SQLite database.
/// Clone it freely — the pool manages connections internally.
#[derive(Clone, Debug)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    /// Open (or create) the database at `path` and run all pending migrations.
    pub async fn open(path: &Path) -> anyhow::Result<Self> {
        let url = format!("sqlite://{}?mode=rwc", path.display());

        let options = SqliteConnectOptions::from_str(&url)
            .context("invalid database URL")?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) // better concurrency
            .foreign_keys(true); // enforce FK constraints

        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .context("failed to open SQLite database")?;

        info!(path = %path.display(), "database opened");

        migrations::run(&pool).await?;

        Ok(Self { pool })
    }
}
````

## File: `crates/storage-sqlite/src/migrations.rs`
````rust
use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

/// Run all embedded migrations in order.
pub async fn run(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations") // path is relative to this crate's Cargo.toml
        .run(pool)
        .await
        .context("database migration failed")?;

    info!("migrations applied");
    Ok(())
}
````

## File: `crates/storage-sqlite/src/profile_repo.rs`
````rust
use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

use domain_core::entities::Profile;

pub struct ProfileRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> ProfileRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new profile for an instance. Returns the new profile's id.
    pub async fn create(&self, instance_id: i64, name: &str) -> anyhow::Result<i64> {
        let id = sqlx::query!(
            r#"
            INSERT INTO profiles (instance_id, name)
            VALUES (?1, ?2)
            "#,
            instance_id,
            name,
        )
        .execute(self.pool)
        .await
        .context("failed to create profile")?
        .last_insert_rowid();

        info!(profile_id = id, name, "profile created");
        Ok(id)
    }

    /// List all profiles for an instance.
    pub async fn list(&self, instance_id: i64) -> anyhow::Result<Vec<Profile>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, instance_id, name, pinned_runner_id, auto_deploy
            FROM profiles
            WHERE instance_id = ?1
            ORDER BY created_at
            "#,
            instance_id,
        )
        .fetch_all(self.pool)
        .await
        .context("failed to list profiles")?;

        Ok(rows
            .into_iter()
            .map(|r| Profile {
                id: r.id.expect("id is always set for persisted rows"),
                instance_id: r.instance_id,
                name: r.name,
                pinned_runner_id: r.pinned_runner_id,
                auto_deploy: r.auto_deploy != 0,
            })
            .collect())
    }

    /// Delete a profile and cascade all its mod/plugin state.
    pub async fn delete(&self, profile_id: i64) -> anyhow::Result<()> {
        sqlx::query!("DELETE FROM profiles WHERE id = ?1", profile_id)
            .execute(self.pool)
            .await
            .context("failed to delete profile")?;

        info!(profile_id, "profile deleted");
        Ok(())
    }
}
````

## File: `crates/storage-sqlite/src/instance_repo.rs`
````rust
use anyhow::Context;
use sqlx::SqlitePool;
use tracing::info;

pub struct InstanceRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> InstanceRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert or update a Fallout 4 instance by install path and return its id.
    pub async fn upsert_fallout4_instance(
        &self,
        install_path: &str,
        source_type: &str,
        label: &str,
    ) -> anyhow::Result<i64> {
        let game_id: i64 = sqlx::query_scalar("SELECT id FROM games WHERE canonical_id = ?1")
            .bind("fallout4")
            .fetch_one(self.pool)
            .await
            .context("failed to load fallout4 game id")?;

        let instance_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO instances (game_id, label, source_type, install_path)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(install_path) DO UPDATE SET
                label = excluded.label,
                source_type = excluded.source_type
            RETURNING id
            "#,
        )
        .bind(game_id)
        .bind(label)
        .bind(source_type)
        .bind(install_path)
        .fetch_one(self.pool)
        .await
        .context("failed to upsert game instance")?;

        info!(
            instance_id,
            install_path, source_type, "game instance upserted"
        );
        Ok(instance_id)
    }
}
````

## File: `crates/storage-sqlite/src/profile_mod_repo.rs`
````rust
use anyhow::Context;
use sqlx::{Row, SqlitePool};
use tracing::info;

#[derive(Debug, Clone)]
pub struct ProfileModRow {
    pub mod_id: i64,
    pub mod_name: String,
    pub enabled: bool,
    pub priority: i32,
}

pub struct ProfileModRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> ProfileModRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, profile_id: i64) -> anyhow::Result<Vec<ProfileModRow>> {
        let rows = sqlx::query(
            r#"
            SELECT pm.mod_id, m.name AS mod_name, pm.enabled, pm.priority
            FROM profile_mods pm
            JOIN mods m ON m.id = pm.mod_id
            WHERE pm.profile_id = ?1
            ORDER BY pm.priority ASC
            "#,
        )
        .bind(profile_id)
        .fetch_all(self.pool)
        .await
        .context("failed to list profile mods")?;

        rows.into_iter()
            .map(|row| {
                Ok(ProfileModRow {
                    mod_id: row.try_get("mod_id")?,
                    mod_name: row.try_get("mod_name")?,
                    enabled: row.try_get::<i64, _>("enabled")? != 0,
                    priority: row.try_get("priority")?,
                })
            })
            .collect()
    }

    pub async fn upsert(
        &self,
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
        priority: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO profile_mods (profile_id, mod_id, enabled, priority)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(profile_id, mod_id) DO UPDATE SET
                enabled = excluded.enabled,
                priority = excluded.priority
            "#,
        )
        .bind(profile_id)
        .bind(mod_id)
        .bind(if enabled { 1_i64 } else { 0_i64 })
        .bind(priority)
        .execute(self.pool)
        .await
        .context("failed to upsert profile mod state")?;

        info!(
            profile_id,
            mod_id, enabled, priority, "profile mod state upserted"
        );
        Ok(())
    }

    pub async fn set_enabled(
        &self,
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE profile_mods SET enabled = ?1 WHERE profile_id = ?2 AND mod_id = ?3",
        )
        .bind(if enabled { 1_i64 } else { 0_i64 })
        .bind(profile_id)
        .bind(mod_id)
        .execute(self.pool)
        .await
        .context("failed to update profile mod enabled state")?;

        if result.rows_affected() == 0 {
            anyhow::bail!("mod {mod_id} is not attached to profile {profile_id}");
        }

        info!(profile_id, mod_id, enabled, "profile mod enabled updated");
        Ok(())
    }

    pub async fn set_priority(
        &self,
        profile_id: i64,
        mod_id: i64,
        priority: i32,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE profile_mods SET priority = ?1 WHERE profile_id = ?2 AND mod_id = ?3",
        )
        .bind(priority)
        .bind(profile_id)
        .bind(mod_id)
        .execute(self.pool)
        .await
        .context("failed to update profile mod priority")?;

        if result.rows_affected() == 0 {
            anyhow::bail!("mod {mod_id} is not attached to profile {profile_id}");
        }

        info!(profile_id, mod_id, priority, "profile mod priority updated");
        Ok(())
    }
}
````

## File: `crates/storage-sqlite/src/profile_plugin_repo.rs`
````rust
use anyhow::Context;
use sqlx::{Row, SqlitePool};
use tracing::info;

pub struct ProfilePluginRepo<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> ProfilePluginRepo<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, profile_id: i64) -> anyhow::Result<Vec<ipc_api::ProfilePluginInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT pp.plugin_id, p.filename, p.kind, pp.enabled, pp.load_index
            FROM profile_plugins pp
            JOIN plugins p ON p.id = pp.plugin_id
            WHERE pp.profile_id = ?1
            ORDER BY pp.load_index ASC
            "#,
        )
        .bind(profile_id)
        .fetch_all(self.pool)
        .await
        .context("failed to list profile plugins")?;

        rows.into_iter()
            .map(|row| {
                Ok(ipc_api::ProfilePluginInfo {
                    plugin_id: row.try_get("plugin_id")?,
                    filename: row.try_get("filename")?,
                    kind: row.try_get("kind")?,
                    enabled: row.try_get::<i64, _>("enabled")? != 0,
                    load_index: row.try_get("load_index")?,
                })
            })
            .collect()
    }

    pub async fn set_enabled(
        &self,
        profile_id: i64,
        plugin_id: i64,
        enabled: bool,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE profile_plugins SET enabled = ?1 WHERE profile_id = ?2 AND plugin_id = ?3")
            .bind(if enabled { 1_i64 } else { 0_i64 })
            .bind(profile_id)
            .bind(plugin_id)
            .execute(self.pool)
            .await
            .context("failed to update plugin enabled state")?;

        info!(profile_id, plugin_id, enabled, "plugin enabled state updated");
        Ok(())
    }
}
````

## File: `crates/storage-sqlite/migrations/0001_initial.sql`
````sql
-- Games supported by the manager (v1: Fallout 4 only)
CREATE TABLE IF NOT EXISTS games (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    canonical_id TEXT NOT NULL UNIQUE   -- e.g. "fallout4"
);

-- A specific installation of a game on disk
CREATE TABLE IF NOT EXISTS instances (
    id          INTEGER PRIMARY KEY,
    game_id     INTEGER NOT NULL REFERENCES games(id),
    label       TEXT NOT NULL,
    source_type TEXT NOT NULL,          -- "steam" | "gog" | "manual"
    install_path TEXT NOT NULL UNIQUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Profiles: isolated mod/plugin/config environments
CREATE TABLE IF NOT EXISTS profiles (
    id                INTEGER PRIMARY KEY,
    instance_id       INTEGER NOT NULL REFERENCES instances(id),
    name              TEXT NOT NULL,
    pinned_runner_id  INTEGER,           -- NULL = no runner pinned yet
    auto_deploy       INTEGER NOT NULL DEFAULT 0,  -- 0=false, 1=true
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(instance_id, name)
);

-- Global mod store: one row per installed mod archive
CREATE TABLE IF NOT EXISTS mods (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    version      TEXT,
    source_hash  TEXT NOT NULL UNIQUE,  -- SHA-256 of original archive
    install_path TEXT NOT NULL,         -- path inside managed mods dir
    installed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Per-profile mod state
CREATE TABLE IF NOT EXISTS profile_mods (
    profile_id  INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    mod_id      INTEGER NOT NULL REFERENCES mods(id)     ON DELETE CASCADE,
    enabled     INTEGER NOT NULL DEFAULT 1,
    priority    INTEGER NOT NULL,       -- lower number = lower priority
    PRIMARY KEY (profile_id, mod_id),
    UNIQUE(profile_id, priority)
);

-- Plugin metadata cache
CREATE TABLE IF NOT EXISTS plugins (
    id       INTEGER PRIMARY KEY,
    mod_id   INTEGER REFERENCES mods(id) ON DELETE SET NULL,
    filename TEXT NOT NULL UNIQUE,
    kind     TEXT NOT NULL              -- "esm" | "esp" | "esl"
);

-- Per-profile plugin state
CREATE TABLE IF NOT EXISTS profile_plugins (
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    plugin_id  INTEGER NOT NULL REFERENCES plugins(id)  ON DELETE CASCADE,
    enabled    INTEGER NOT NULL DEFAULT 1,
    load_index INTEGER NOT NULL,        -- position in load order
    PRIMARY KEY (profile_id, plugin_id),
    UNIQUE(profile_id, load_index)
);

-- Deploy manifests: immutable record of each deployment
CREATE TABLE IF NOT EXISTS deploy_manifests (
    id          INTEGER PRIMARY KEY,
    profile_id  INTEGER NOT NULL REFERENCES profiles(id),
    deployed_at TEXT NOT NULL DEFAULT (datetime('now')),
    symlink_plan TEXT NOT NULL,         -- JSON blob of {source -> target} map
    status      TEXT NOT NULL DEFAULT 'active'  -- "active" | "rolled_back"
);

-- File index: every file owned by every mod
CREATE TABLE IF NOT EXISTS file_index (
    id         INTEGER PRIMARY KEY,
    mod_id     INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    rel_path   TEXT NOT NULL,           -- e.g. "Textures/foo.dds"
    is_ba2     INTEGER NOT NULL DEFAULT 0,
    UNIQUE(mod_id, rel_path)
);

-- Runner catalog: Proton and Wine runners
CREATE TABLE IF NOT EXISTS runner_catalog (
    id           INTEGER PRIMARY KEY,
    kind         TEXT NOT NULL,         -- "proton" | "wine"
    version      TEXT NOT NULL,
    source_url   TEXT,
    install_path TEXT NOT NULL,
    verified     INTEGER NOT NULL DEFAULT 0,
    installed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- F4SE state per instance
CREATE TABLE IF NOT EXISTS f4se_state (
    id           INTEGER PRIMARY KEY,
    instance_id  INTEGER NOT NULL REFERENCES instances(id) ON DELETE CASCADE UNIQUE,
    detected_version TEXT,
    compatible   INTEGER,               -- NULL=unchecked, 1=ok, 0=mismatch
    last_checked TEXT
);

-- Global settings key-value store
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Seed the one supported game
INSERT OR IGNORE INTO games (name, canonical_id)
VALUES ('Fallout 4', 'fallout4');
````

## File: `crates/storage-sqlite/migrations/0002_plugin_masters.sql`
````sql
ALTER TABLE plugins
ADD COLUMN masters_json TEXT NOT NULL DEFAULT '[]';
````

## File: `crates/game-detect/Cargo.toml`
````toml
[package]
name = "game-detect"
version = "0.1.0"
edition = "2024"

[dependencies]
domain-core      = { workspace = true }
anyhow           = { workspace = true }
thiserror        = { workspace = true }
tracing          = { workspace = true }
tokio            = { workspace = true }
keyvalues-parser = { workspace = true }
````

## File: `crates/game-detect/src/lib.rs`
````rust
mod steam;
mod validate;

pub use steam::SteamDetector;
pub use validate::validate_fo4_path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectError {
    #[error("Steam not found at expected paths")]
    SteamNotFound,

    #[error("Fallout 4 not found in any Steam library")]
    Fo4NotFound,

    #[error("path is not a valid Fallout 4 install: {reason}")]
    InvalidPath { reason: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
````

## File: `crates/game-detect/src/steam.rs`
````rust
use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};

use crate::DetectError;

const FO4_APP_ID: &str = "377160";

pub struct SteamDetector;

impl SteamDetector {
    pub fn detect() -> Result<PathBuf, DetectError> {
        let steam_root = Self::find_steam_root()?;
        info!(path = %steam_root.display(), "found Steam root");

        let libraries = Self::find_library_folders(&steam_root)?;
        debug!(count = libraries.len(), "Steam library folders found");

        for lib in &libraries {
            debug!(path = %lib.display(), "scanning library");
            match Self::find_fo4_in_library(lib) {
                Some(fo4_path) => {
                    info!(path = %fo4_path.display(), "Fallout 4 found");
                    return Ok(fo4_path);
                }
                None => continue,
            }
        }

        Err(DetectError::Fo4NotFound)
    }

    fn find_steam_root() -> Result<PathBuf, DetectError> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());

        let candidates = [
            PathBuf::from(&home).join(".steam/steam"),
            PathBuf::from(&home).join(".local/share/Steam"),
            PathBuf::from("/usr/share/steam"),
        ];

        for path in &candidates {
            if path.join("steamapps").exists() {
                return Ok(path.clone());
            }
        }

        Err(DetectError::SteamNotFound)
    }

    fn find_library_folders(steam_root: &Path) -> Result<Vec<PathBuf>, DetectError> {
        let vdf_path = steam_root.join("steamapps/libraryfolders.vdf");

        let mut libraries = vec![steam_root.join("steamapps")];

        if !vdf_path.exists() {
            warn!("libraryfolders.vdf not found, using Steam root only");
            return Ok(libraries);
        }

        let content = std::fs::read_to_string(&vdf_path)?;

        use keyvalues_parser::Vdf;
        if let Ok(vdf) = Vdf::parse(&content) {
            if let Some(obj) = vdf.value.get_obj() {
                for (_key, values) in obj.iter() {
                    for value in values {
                        if let Some(inner) = value.get_obj() {
                            if let Some(path_vals) = inner.get("path") {
                                if let Some(path_str) = path_vals.first().and_then(|v| v.get_str())
                                {
                                    let lib = PathBuf::from(path_str).join("steamapps");
                                    if lib.exists() {
                                        libraries.push(lib);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(libraries)
    }

    fn find_fo4_in_library(steamapps: &Path) -> Option<PathBuf> {
        let manifest = steamapps.join(format!("appmanifest_{FO4_APP_ID}.acf"));
        if !manifest.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&manifest).ok()?;
        let install_dir = Self::extract_install_dir(&content)?;

        let game_path = steamapps.join("common").join(install_dir);
        if game_path.exists() {
            Some(game_path)
        } else {
            None
        }
    }

    fn extract_install_dir(content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("\"installdir\"") {
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if let Some(val) = parts.last() {
                    return Some(val.trim().trim_matches('"').to_string());
                }
            }
        }
        None
    }
}
````

## File: `crates/game-detect/src/validate.rs`
````rust
use crate::DetectError;
use std::path::Path;

/// Confirm a directory is a real Fallout 4 install by checking for
/// key files that must be present.
pub fn validate_fo4_path(path: &Path) -> Result<(), DetectError> {
    let required = ["Fallout4.exe", "Data"];

    for entry in &required {
        if !path.join(entry).exists() {
            return Err(DetectError::InvalidPath {
                reason: format!("missing '{entry}' in {}", path.display()),
            });
        }
    }

    Ok(())
}
````

## File: `crates/runner-manager/Cargo.toml`
````toml
[package]
name = "runner-manager"
version = "0.1.0"
edition = "2024"

[dependencies]
domain-core    = { workspace = true }
storage-sqlite = { workspace = true }
anyhow         = { workspace = true }
tracing        = { workspace = true }
sqlx           = { workspace = true }
````

## File: `crates/runner-manager/src/lib.rs`
````rust
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Context;
use domain_core::{
    entities::{Runner, RunnerKind},
    error::DomainError,
};
use sqlx::Row;
use storage_sqlite::Db;
use tracing::{info, warn};

#[derive(Debug)]
struct RunnerCandidate {
    kind: String,
    version: String,
    install_path: String,
}

pub async fn detect_runners(db: &Db) -> anyhow::Result<()> {
    let mut candidates = discover_proton_candidates()?;
    candidates.extend(discover_wine_candidates());
    let detected_count = candidates.len();

    for candidate in candidates {
        let existing_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM runner_catalog WHERE kind = ?1 AND install_path = ?2 LIMIT 1",
        )
        .bind(&candidate.kind)
        .bind(&candidate.install_path)
        .fetch_optional(&db.pool)
        .await
        .context("failed to query existing runner")?;

        if let Some(id) = existing_id {
            sqlx::query("UPDATE runner_catalog SET version = ?1, verified = 1 WHERE id = ?2")
                .bind(&candidate.version)
                .bind(id)
                .execute(&db.pool)
                .await
                .context("failed to update runner")?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO runner_catalog (kind, version, source_url, install_path, verified)
                VALUES (?1, ?2, NULL, ?3, 1)
                "#,
            )
            .bind(&candidate.kind)
            .bind(&candidate.version)
            .bind(&candidate.install_path)
            .execute(&db.pool)
            .await
            .context("failed to insert runner")?;
        }
    }

    info!(count = detected_count, "runner detection complete");
    Ok(())
}

pub async fn list_runners(db: &Db) -> anyhow::Result<Vec<Runner>> {
    let rows = sqlx::query(
        r#"
        SELECT id, kind, version, source_url, install_path, verified
        FROM runner_catalog
        ORDER BY kind, version
        "#,
    )
    .fetch_all(&db.pool)
    .await
    .context("failed to list runners")?;

    rows.into_iter().map(row_to_runner).collect()
}

pub async fn pin_profile_runner(profile_id: i64, runner_id: i64, db: &Db) -> anyhow::Result<()> {
    let verified: Option<i64> =
        sqlx::query_scalar("SELECT verified FROM runner_catalog WHERE id = ?1")
            .bind(runner_id)
            .fetch_optional(&db.pool)
            .await
            .context("failed to load runner")?;

    let verified = match verified {
        Some(v) => v != 0,
        None => anyhow::bail!("runner {runner_id} was not found"),
    };

    if !verified {
        return Err(DomainError::RunnerNotVerified.into());
    }

    let result = sqlx::query("UPDATE profiles SET pinned_runner_id = ?1 WHERE id = ?2")
        .bind(runner_id)
        .bind(profile_id)
        .execute(&db.pool)
        .await
        .context("failed to pin runner to profile")?;

    if result.rows_affected() == 0 {
        anyhow::bail!("profile {profile_id} was not found");
    }

    info!(profile_id, runner_id, "runner pinned to profile");
    Ok(())
}

fn row_to_runner(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<Runner> {
    let kind_raw: String = row.try_get("kind")?;
    let kind = parse_kind(&kind_raw)?;
    let verified: i64 = row.try_get("verified")?;

    Ok(Runner {
        id: row.try_get("id")?,
        kind,
        version: row.try_get("version")?,
        source_url: row.try_get("source_url")?,
        install_path: row.try_get("install_path")?,
        verified: verified != 0,
    })
}

fn parse_kind(kind: &str) -> anyhow::Result<RunnerKind> {
    match kind {
        "proton" => Ok(RunnerKind::Proton),
        "wine" => Ok(RunnerKind::Wine),
        other => anyhow::bail!("unknown runner kind in catalog: {other}"),
    }
}

fn discover_proton_candidates() -> anyhow::Result<Vec<RunnerCandidate>> {
    let home = std::env::var("HOME").context("HOME is not set")?;
    let roots = [
        (
            PathBuf::from(&home).join(".steam/steam/steamapps/common"),
            true,
        ),
        (
            PathBuf::from(&home).join(".local/share/Steam/steamapps/common"),
            true,
        ),
        (
            PathBuf::from(&home).join(".steam/steam/compatibilitytools.d"),
            false,
        ),
        (
            PathBuf::from(&home).join(".local/share/Steam/compatibilitytools.d"),
            false,
        ),
    ];

    let mut found = Vec::new();
    let mut seen_paths = HashSet::new();

    for (root, require_proton_prefix) in roots {
        scan_proton_root(&root, require_proton_prefix, &mut seen_paths, &mut found)?;
    }

    Ok(found)
}

fn scan_proton_root(
    root: &Path,
    require_proton_prefix: bool,
    seen_paths: &mut HashSet<String>,
    found: &mut Vec<RunnerCandidate>,
) -> anyhow::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    let entries =
        std::fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if require_proton_prefix && !name.starts_with("Proton") {
            continue;
        }

        if !path.join("proton").is_file() {
            continue;
        }

        let canonical = std::fs::canonicalize(&path).unwrap_or(path);
        let install_path = canonical.display().to_string();
        if !seen_paths.insert(install_path.clone()) {
            continue;
        }

        found.push(RunnerCandidate {
            kind: "proton".to_string(),
            version: name,
            install_path,
        });
    }

    Ok(())
}

fn discover_wine_candidates() -> Vec<RunnerCandidate> {
    let paths = ["/usr/bin/wine", "/bin/wine"];
    let mut found = Vec::new();

    for path in paths {
        if !Path::new(path).exists() {
            continue;
        }

        found.push(RunnerCandidate {
            kind: "wine".to_string(),
            version: read_wine_version(path),
            install_path: path.to_string(),
        });
    }

    found
}

fn read_wine_version(binary: &str) -> String {
    let output = std::process::Command::new(binary).arg("--version").output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if text.is_empty() {
                "system".to_string()
            } else {
                text
            }
        }
        Ok(_) | Err(_) => {
            warn!(
                binary,
                "failed to detect wine version, defaulting to system"
            );
            "system".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn steam_common_scan_requires_proton_prefix() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Proton 9.0")).expect("failed to create Proton dir");
        fs::create_dir_all(root.join("GE-Proton9-24")).expect("failed to create GE dir");
        fs::write(root.join("Proton 9.0/proton"), b"").expect("failed to create proton binary");
        fs::write(root.join("GE-Proton9-24/proton"), b"").expect("failed to create proton binary");

        let mut found = Vec::new();
        let mut seen = HashSet::new();
        scan_proton_root(&root, true, &mut seen, &mut found).expect("scan should succeed");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].version, "Proton 9.0");
        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    #[test]
    fn compatibilitytools_scan_accepts_non_proton_prefix() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("GE-Proton9-24")).expect("failed to create GE dir");
        fs::create_dir_all(root.join("NotARunner")).expect("failed to create non-runner dir");
        fs::write(root.join("GE-Proton9-24/proton"), b"").expect("failed to create proton binary");

        let mut found = Vec::new();
        let mut seen = HashSet::new();
        scan_proton_root(&root, false, &mut seen, &mut found).expect("scan should succeed");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].version, "GE-Proton9-24");
        fs::remove_dir_all(root).expect("cleanup should succeed");
    }

    fn unique_temp_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after epoch")
            .as_nanos();
        let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("mm-runner-manager-tests-{nonce}-{seq}"));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }
}
````

## File: `crates/mod-ingest/Cargo.toml`
````toml
[package]
name = "mod-ingest"
version = "0.1.0"
edition = "2024"

[dependencies]
domain-core    = { workspace = true }
storage-sqlite = { workspace = true }
anyhow         = { workspace = true }
thiserror      = { workspace = true }
tracing        = { workspace = true }
tokio          = { workspace = true }
serde          = { workspace = true }
serde_json     = { workspace = true }
sha2           = { workspace = true }
hex            = { workspace = true }
sevenz-rust2   = { workspace = true }
zip            = { workspace = true }
walkdir        = { workspace = true }
sqlx           = { workspace = true }
fomod-oxide    = { workspace = true }
````

## File: `crates/mod-ingest/src/lib.rs`
````rust
pub mod archive;
pub mod fomod;
pub mod hasher;
pub mod ingest;

pub use ingest::{IngestResult, ingest_mod};
````

## File: `crates/mod-ingest/src/hasher.rs`
````rust
use anyhow::Context;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Compute SHA-256 of a file and return it as a lowercase hex string.
pub async fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read {} for hashing", path.display()))?;

    let hash = Sha256::digest(&bytes);
    Ok(hex::encode(hash))
}
````

## File: `crates/mod-ingest/src/archive.rs`
````rust
use anyhow::Context;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[derive(Debug, PartialEq)]
pub enum ArchiveKind {
    Zip,
    SevenZip,
    Rar,
}

pub fn detect_kind(path: &Path) -> anyhow::Result<ArchiveKind> {
    use std::io::Read;
    let mut f =
        std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let mut magic = [0u8; 7];
    f.read_exact(&mut magic)
        .context("file too small to detect archive type")?;

    if magic[..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Ok(ArchiveKind::Zip);
    }
    if magic[..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return Ok(ArchiveKind::SevenZip);
    }
    if magic[..4] == [0x52, 0x61, 0x72, 0x21] {
        return Ok(ArchiveKind::Rar);
    }

    anyhow::bail!("unrecognised archive format: {}", path.display())
}

pub fn extract(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dest_dir).context("failed to create extraction directory")?;

    let extracted = match detect_kind(archive_path)? {
        ArchiveKind::Zip => extract_zip(archive_path, dest_dir),
        ArchiveKind::SevenZip => extract_7z(archive_path, dest_dir),
        ArchiveKind::Rar => {
            anyhow::bail!("RAR archives are not supported. Please repack as ZIP or 7z.")
        }
    }?;

    strip_single_top_level_wrapper(dest_dir, extracted)
}

fn extract_zip(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let file = std::fs::File::open(archive_path)?;
    let mut zip = zip::ZipArchive::new(file).context("failed to open ZIP archive")?;

    let mut extracted = Vec::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let rel = PathBuf::from(entry.name());

        if rel.is_absolute() || rel.components().any(|c| c.as_os_str() == "..") {
            continue;
        }

        let out = dest_dir.join(&rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&out)?;
            std::io::copy(&mut entry, &mut outfile)?;
            debug!(path = %rel.display(), "extracted");
            extracted.push(rel);
        }
    }

    info!(count = extracted.len(), "ZIP extraction complete");
    Ok(extracted)
}

fn extract_7z(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    sevenz_rust2::decompress_file(archive_path, dest_dir).context("7z extraction failed")?;

    let extracted = collect_extracted_files(dest_dir)?;

    info!(count = extracted.len(), "7z extraction complete");
    Ok(extracted)
}

fn strip_single_top_level_wrapper(
    dest_dir: &Path,
    extracted: Vec<PathBuf>,
) -> anyhow::Result<Vec<PathBuf>> {
    if extracted.is_empty() {
        return Ok(extracted);
    }

    let mut top_levels = extracted
        .iter()
        .filter_map(|rel| rel.components().next().map(|c| c.as_os_str().to_owned()))
        .collect::<std::collections::HashSet<_>>();

    if top_levels.len() != 1 {
        return Ok(extracted);
    }

    let wrapper_name = match top_levels.drain().next() {
        Some(name) => PathBuf::from(name),
        None => return Ok(extracted),
    };
    let wrapper_dir = dest_dir.join(&wrapper_name);
    if !wrapper_dir.is_dir() {
        return Ok(extracted);
    }

    for entry in std::fs::read_dir(&wrapper_dir)
        .with_context(|| format!("failed to read wrapper dir {}", wrapper_dir.display()))?
    {
        let entry = entry?;
        let from = entry.path();
        let to = dest_dir.join(entry.file_name());

        if to.exists() {
            anyhow::bail!(
                "wrapper normalisation conflict while moving {} to {}",
                from.display(),
                to.display()
            );
        }

        std::fs::rename(&from, &to).with_context(|| {
            format!(
                "failed to move wrapped path {} to {}",
                from.display(),
                to.display()
            )
        })?;
    }

    std::fs::remove_dir(&wrapper_dir)
        .with_context(|| format!("failed to remove wrapper dir {}", wrapper_dir.display()))?;

    let normalized = collect_extracted_files(dest_dir)?;
    info!(
        wrapper = %wrapper_name.display(),
        count = normalized.len(),
        "stripped single top-level wrapper directory"
    );
    Ok(normalized)
}

fn collect_extracted_files(dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut extracted = Vec::new();

    for entry in walkdir::WalkDir::new(dest_dir) {
        let entry = entry.with_context(|| format!("failed to walk {}", dest_dir.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let rel = entry.path().strip_prefix(dest_dir).with_context(|| {
            format!(
                "failed to compute relative extracted path for {}",
                entry.path().display()
            )
        })?;
        extracted.push(rel.to_path_buf());
    }

    Ok(extracted)
}
````

## File: `crates/mod-ingest/src/ingest.rs`
````rust
use std::path::Path;

use anyhow::Context;
use tracing::info;

use storage_sqlite::Db;

use crate::{archive, fomod, hasher};

#[derive(Debug)]
pub struct IngestResult {
    pub mod_id: i64,
    pub name: String,
    pub source_hash: String,
    pub file_count: usize,
}

/// Full pipeline: hash → dedup check → extract → index → record in DB.
pub async fn ingest_mod(
    archive_path: &Path,
    mods_dir: &Path,
    db: &Db,
) -> anyhow::Result<IngestResult> {
    // 1. Hash the archive for deduplication
    let hash = hasher::sha256_file(archive_path).await?;
    info!(hash = %hash, "archive hashed");

    // 2. Check if already installed
    let existing = sqlx::query!("SELECT id, name FROM mods WHERE source_hash = ?1", hash)
        .fetch_optional(&db.pool)
        .await
        .context("dedup check failed")?;

    if let Some(row) = existing {
        info!(
            mod_id = row.id,
            "mod already installed, skipping extraction"
        );
        let file_count =
            sqlx::query_scalar!("SELECT COUNT(*) FROM file_index WHERE mod_id = ?1", row.id)
                .fetch_one(&db.pool)
                .await? as usize;

        return Ok(IngestResult {
            mod_id: row.id.expect("id is always set for persisted rows"),
            name: row.name,
            source_hash: hash,
            file_count,
        });
    }

    // 3. Derive mod name from archive filename
    let name = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown_mod")
        .to_string();

    // 4. Extract into mods/<hash>/
    let install_path = mods_dir.join(&hash);
    let install_path_str = install_path.display().to_string();

    info!(dest = %install_path.display(), "extracting archive");
    let files = tokio::task::spawn_blocking({
        let ap = archive_path.to_path_buf();
        let ip = install_path.clone();
        move || archive::extract(&ap, &ip)
    })
    .await
    .context("extraction task panicked")?
    .context("extraction failed")?;

    let files = tokio::task::spawn_blocking({
        let ip = install_path.clone();
        move || fomod::apply_if_present(&ip, files)
    })
    .await
    .context("FOMOD processing task panicked")?
    .context("FOMOD processing failed")?;

    // 5. Record mod in DB
    let mod_id = sqlx::query!(
        r#"
        INSERT INTO mods (name, source_hash, install_path)
        VALUES (?1, ?2, ?3)
        "#,
        name,
        hash,
        install_path_str,
    )
    .execute(&db.pool)
    .await
    .context("failed to record mod")?
    .last_insert_rowid();

    // 6. Index all extracted files
    for rel in &files {
        let rel_str = rel.display().to_string();
        // Normalise path separator to forward slash for consistency
        let rel_str = rel_str.replace('\\', "/");
        let is_ba2 = rel_str.to_lowercase().ends_with(".ba2") as i64;

        sqlx::query!(
            "INSERT OR IGNORE INTO file_index (mod_id, rel_path, is_ba2) VALUES (?1, ?2, ?3)",
            mod_id,
            rel_str,
            is_ba2,
        )
        .execute(&db.pool)
        .await
        .context("failed to index file")?;
    }

    info!(mod_id, name = %name, files = files.len(), "mod ingested");

    Ok(IngestResult {
        mod_id,
        name,
        source_hash: hash,
        file_count: files.len(),
    })
}
````

## File: `crates/mod-ingest/src/fomod.rs`
````rust
use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

use anyhow::Context;
use fomod_oxide::{
    Installer, ModuleConfig,
    config::GroupType,
    installer::{FileOperation, InstallPlan},
};
use tracing::info;

pub fn apply_if_present(
    install_root: &Path,
    extracted_files: Vec<PathBuf>,
) -> anyhow::Result<Vec<PathBuf>> {
    let Some(config_path) = find_module_config(install_root)? else {
        return Ok(extracted_files);
    };

    let config_xml = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let config = ModuleConfig::parse(&config_xml).context("failed to parse ModuleConfig.xml")?;

    let mut installer = Installer::new(config);
    if !installer.check_dependencies() {
        anyhow::bail!("FOMOD module dependencies are not satisfied");
    }

    let selections = collect_default_selections(&installer)?;
    for (step_idx, group_idx, selected) in selections {
        installer.select(step_idx, group_idx, selected);
    }

    if !installer.is_ready_to_install() {
        anyhow::bail!("FOMOD requires interactive selections that are not safely auto-resolvable");
    }

    let plan = installer.resolve();
    if plan.operations.is_empty() {
        return Ok(extracted_files);
    }

    let selected_files = apply_install_plan(install_root, &plan)?;
    info!(
        count = selected_files.len(),
        "FOMOD install plan applied via fomod-oxide"
    );
    Ok(selected_files)
}

fn collect_default_selections(
    installer: &Installer,
) -> anyhow::Result<Vec<(usize, usize, Vec<usize>)>> {
    let mut selections = Vec::new();

    for (step_idx, step) in installer.visible_steps() {
        let Some(groups) = &step.optional_file_groups else {
            continue;
        };

        for (group_idx, group) in groups.groups.iter().enumerate() {
            let mut selected = Installer::default_selections_in_context(group, installer.context());
            if selected.is_empty() {
                selected =
                    fallback_selection_for_group(group.group_type, group.plugins.plugins.len());
            }

            Installer::validate_selection(group, &selected).with_context(|| {
                format!(
                    "failed to auto-select FOMOD group '{}' in step '{}'",
                    group.name, step.name
                )
            })?;

            selections.push((step_idx, group_idx, selected));
        }
    }

    Ok(selections)
}

fn fallback_selection_for_group(group_type: GroupType, plugin_count: usize) -> Vec<usize> {
    if plugin_count == 0 {
        return Vec::new();
    }

    match group_type {
        GroupType::SelectExactlyOne | GroupType::SelectAtLeastOne => vec![0],
        GroupType::SelectAll => (0..plugin_count).collect(),
        GroupType::SelectAtMostOne | GroupType::SelectAny => Vec::new(),
    }
}

fn apply_install_plan(install_root: &Path, plan: &InstallPlan) -> anyhow::Result<Vec<PathBuf>> {
    let mut selected = HashSet::new();

    for op in &plan.operations {
        if op.is_folder {
            apply_folder_operation(install_root, op, &mut selected)?;
        } else {
            apply_file_operation(install_root, op, &mut selected)?;
        }
    }

    let mut out = selected.into_iter().collect::<Vec<_>>();
    out.sort();
    Ok(out)
}

fn apply_file_operation(
    install_root: &Path,
    op: &FileOperation,
    selected: &mut HashSet<PathBuf>,
) -> anyhow::Result<()> {
    let source_rel = sanitize_relative(&op.source)
        .with_context(|| format!("invalid FOMOD file source: {}", op.source))?;
    let source_abs = install_root.join(&source_rel);
    if !source_abs.is_file() {
        anyhow::bail!("FOMOD file source does not exist: {}", source_abs.display());
    }

    let destination_rel = normalize_destination_file(&op.destination, &source_rel)
        .with_context(|| format!("invalid FOMOD file destination: {}", op.destination))?;

    copy_into_install_root(install_root, &source_abs, &destination_rel)?;
    selected.insert(destination_rel);
    Ok(())
}

fn apply_folder_operation(
    install_root: &Path,
    op: &FileOperation,
    selected: &mut HashSet<PathBuf>,
) -> anyhow::Result<()> {
    let source_rel = sanitize_relative(&op.source)
        .with_context(|| format!("invalid FOMOD folder source: {}", op.source))?;
    let source_abs = install_root.join(&source_rel);
    if !source_abs.is_dir() {
        anyhow::bail!(
            "FOMOD folder source does not exist: {}",
            source_abs.display()
        );
    }

    let destination_base = normalize_destination_folder(&op.destination)
        .with_context(|| format!("invalid FOMOD folder destination: {}", op.destination))?;

    for entry in walkdir::WalkDir::new(&source_abs) {
        let entry = entry
            .with_context(|| format!("failed to walk FOMOD source {}", source_abs.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let sub_rel = entry.path().strip_prefix(&source_abs).with_context(|| {
            format!(
                "failed to compute relative FOMOD path for {}",
                entry.path().display()
            )
        })?;
        let destination_rel = destination_base.join(sub_rel);

        copy_into_install_root(install_root, entry.path(), &destination_rel)?;
        selected.insert(destination_rel);
    }

    Ok(())
}

fn copy_into_install_root(
    install_root: &Path,
    source_abs: &Path,
    destination_rel: &Path,
) -> anyhow::Result<()> {
    let destination_abs = install_root.join(destination_rel);
    if source_abs == destination_abs {
        return Ok(());
    }

    if let Some(parent) = destination_abs.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    std::fs::copy(source_abs, &destination_abs).with_context(|| {
        format!(
            "failed to copy FOMOD file {} -> {}",
            source_abs.display(),
            destination_abs.display()
        )
    })?;
    Ok(())
}

fn find_module_config(install_root: &Path) -> anyhow::Result<Option<PathBuf>> {
    for entry in walkdir::WalkDir::new(install_root) {
        let entry = entry.with_context(|| format!("failed to walk {}", install_root.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let rel = entry.path().strip_prefix(install_root).with_context(|| {
            format!(
                "failed to compute relative path for {}",
                entry.path().display()
            )
        })?;
        let rel_lower = rel.to_string_lossy().replace('\\', "/").to_lowercase();
        if rel_lower.ends_with("fomod/moduleconfig.xml") {
            return Ok(Some(entry.path().to_path_buf()));
        }
    }
    Ok(None)
}

fn normalize_destination_file(raw: &str, source_rel: &Path) -> anyhow::Result<PathBuf> {
    let destination = normalize_destination(raw)?;
    let source_name = source_rel
        .file_name()
        .context("FOMOD file source missing filename")?;

    let looks_like_directory = destination.as_os_str().is_empty()
        || raw.ends_with('/')
        || raw.ends_with('\\')
        || destination.extension().is_none()
        || destination
            .file_name()
            .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("data"));

    if looks_like_directory {
        Ok(destination.join(source_name))
    } else {
        Ok(destination)
    }
}

fn normalize_destination_folder(raw: &str) -> anyhow::Result<PathBuf> {
    normalize_destination(raw)
}

fn normalize_destination(raw: &str) -> anyhow::Result<PathBuf> {
    let mut rel = sanitize_relative(raw)?;
    if matches!(
        rel.components().next(),
        Some(Component::Normal(first)) if first.to_string_lossy().eq_ignore_ascii_case("data")
    ) {
        rel = rel
            .components()
            .skip(1)
            .fold(PathBuf::new(), |mut acc, component| {
                if let Component::Normal(part) = component {
                    acc.push(part);
                }
                acc
            });
    }
    Ok(rel)
}

fn sanitize_relative(raw: &str) -> anyhow::Result<PathBuf> {
    let normalized = raw.trim().replace('\\', "/");
    let raw_path = Path::new(normalized.trim_start_matches('/'));
    if raw_path.as_os_str().is_empty() || raw_path == Path::new(".") {
        return Ok(PathBuf::new());
    }

    let mut out = PathBuf::new();
    for component in raw_path.components() {
        match component {
            Component::Normal(seg) => out.push(seg),
            Component::CurDir => {}
            _ => anyhow::bail!("path traversal is not allowed: {raw}"),
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fomod_oxide::config::GroupType;

    #[test]
    fn sanitize_relative_rejects_parent_traversal() {
        let err = sanitize_relative("../outside").expect_err("expected traversal rejection");
        assert!(err.to_string().contains("path traversal"));
    }

    #[test]
    fn normalize_destination_file_strips_data_root() {
        let source = Path::new("Meshes/Foo.nif");
        let out = normalize_destination_file("Data/Meshes", source).expect("valid destination");
        assert_eq!(out, PathBuf::from("Meshes/Foo.nif"));
    }

    #[test]
    fn fallback_selection_matches_group_rules() {
        assert_eq!(
            fallback_selection_for_group(GroupType::SelectExactlyOne, 3),
            vec![0]
        );
        assert_eq!(
            fallback_selection_for_group(GroupType::SelectAtLeastOne, 2),
            vec![0]
        );
        assert_eq!(
            fallback_selection_for_group(GroupType::SelectAll, 3),
            vec![0, 1, 2]
        );
        assert_eq!(
            fallback_selection_for_group(GroupType::SelectAtMostOne, 3),
            Vec::<usize>::new()
        );
        assert_eq!(
            fallback_selection_for_group(GroupType::SelectAny, 3),
            Vec::<usize>::new()
        );
    }
}
````

## File: `crates/deploy-engine/Cargo.toml`
````toml
[package]
name = "deploy-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
domain-core    = { workspace = true }
storage-sqlite = { workspace = true }
anyhow         = { workspace = true }
thiserror      = { workspace = true }
tracing        = { workspace = true }
tokio          = { workspace = true }
serde          = { workspace = true }
serde_json     = { workspace = true }
walkdir        = { workspace = true }
sqlx           = { workspace = true }
````

## File: `crates/deploy-engine/src/lib.rs`
````rust
pub mod apply;
pub mod conflict;
pub mod plan;

pub use apply::{apply_plan, rollback};
pub use plan::{DeployPlan, build_plan};
````

## File: `crates/deploy-engine/src/conflict.rs`
````rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Winner {
    pub rel_path: String,
    pub source_path: String,
    pub winning_mod: i64,
    pub priority: i32,
}

pub fn resolve(mods: &[(i64, i32, String)], file_index: &[(i64, String)]) -> Vec<Winner> {
    let mut best: HashMap<String, (i32, i64, String)> = HashMap::new();

    for (mod_id, priority, install_path) in mods {
        for (fmod_id, rel_path) in file_index {
            if fmod_id != mod_id {
                continue;
            }
            let entry =
                best.entry(rel_path.clone())
                    .or_insert((*priority, *mod_id, install_path.clone()));
            if *priority > entry.0 || (*priority == entry.0 && mod_id > &entry.1) {
                *entry = (*priority, *mod_id, install_path.clone());
            }
        }
    }

    best.into_iter()
        .map(|(rel_path, (priority, mod_id, install_path))| {
            let source_path = format!("{}/{}", install_path, rel_path);
            Winner {
                rel_path,
                source_path,
                winning_mod: mod_id,
                priority,
            }
        })
        .collect()
}
````

## File: `crates/deploy-engine/src/plan.rs`
````rust
use anyhow::Context;
use std::path::{Path, PathBuf};
use tracing::info;

use domain_core::entities::SymlinkEntry;
use storage_sqlite::Db;

use crate::conflict::resolve;

#[derive(Debug)]
pub struct DeployPlan {
    pub profile_id: i64,
    pub entries: Vec<SymlinkEntry>,
}

/// Build a deploy plan for a profile: query enabled mods + file index,
/// resolve conflicts, and produce a list of symlink operations.
pub async fn build_plan(
    profile_id: i64,
    game_data_dir: &Path, // e.g. /path/to/Fallout 4/Data
    db: &Db,
) -> anyhow::Result<DeployPlan> {
    // 1. Load enabled mods for this profile, ordered by priority
    let mod_rows = sqlx::query!(
        r#"
        SELECT m.id, pm.priority, m.install_path
        FROM profile_mods pm
        JOIN mods m ON m.id = pm.mod_id
        WHERE pm.profile_id = ?1 AND pm.enabled = 1
        ORDER BY pm.priority ASC
        "#,
        profile_id,
    )
    .fetch_all(&db.pool)
    .await
    .context("failed to load profile mods")?;

    let mods: Vec<(i64, i32, String)> = mod_rows
        .into_iter()
        .map(|r| (r.id.expect("id set"), r.priority as i32, r.install_path))
        .collect();

    // 2. Load full file index for those mods
    let mod_ids: Vec<i64> = mods.iter().map(|(id, _, _)| *id).collect();

    let mut file_index: Vec<(i64, String)> = Vec::new();
    for mod_id in &mod_ids {
        let files = sqlx::query!(
            "SELECT mod_id, rel_path FROM file_index WHERE mod_id = ?1",
            mod_id
        )
        .fetch_all(&db.pool)
        .await?;
        for f in files {
            file_index.push((f.mod_id, f.rel_path));
        }
    }

    // 3. Resolve conflicts
    let winners = resolve(&mods, &file_index);
    info!(
        profile_id,
        winners = winners.len(),
        "conflict resolution complete"
    );

    // 4. Build symlink entries
    let entries = winners
        .into_iter()
        .map(|w| SymlinkEntry {
            source: w.source_path,
            target: target_path_for_rel_path(game_data_dir, &w.rel_path)
                .display()
                .to_string(),
        })
        .collect();

    Ok(DeployPlan {
        profile_id,
        entries,
    })
}

fn target_path_for_rel_path(game_data_dir: &Path, rel_path: &str) -> PathBuf {
    let rel = Path::new(rel_path);
    if should_deploy_to_game_root(rel) {
        if let Some(game_root) = game_data_dir.parent() {
            return game_root.join(rel);
        }
    }
    game_data_dir.join(rel)
}

fn should_deploy_to_game_root(rel_path: &Path) -> bool {
    // F4SE loader/runtime files must live next to Fallout4.exe, not in Data/.
    if rel_path.components().count() != 1 {
        return false;
    }

    let file_name = rel_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    file_name == "f4se_loader.exe"
        || (file_name.starts_with("f4se_") && file_name.ends_with(".dll"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f4se_loader_targets_game_root() {
        let data_dir = Path::new("/games/fallout4/Data");
        let target = target_path_for_rel_path(data_dir, "f4se_loader.exe");
        assert_eq!(target, PathBuf::from("/games/fallout4/f4se_loader.exe"));
    }

    #[test]
    fn regular_mod_file_targets_data_dir() {
        let data_dir = Path::new("/games/fallout4/Data");
        let target = target_path_for_rel_path(data_dir, "textures/foo.dds");
        assert_eq!(
            target,
            PathBuf::from("/games/fallout4/Data/textures/foo.dds")
        );
    }
}
````

## File: `crates/deploy-engine/src/apply.rs`
````rust
use anyhow::Context;
use std::path::Path;
use tracing::{info, warn};

use domain_core::entities::SymlinkEntry;
use storage_sqlite::Db;

use crate::plan::DeployPlan;

/// Apply a deploy plan: create symlinks and write a manifest for rollback.
pub async fn apply_plan(plan: DeployPlan, db: &Db) -> anyhow::Result<i64> {
    let symlink_json =
        serde_json::to_string(&plan.entries).context("failed to serialise symlink plan")?;

    // Write manifest first — if symlinking fails we still have a record
    let manifest_id = sqlx::query!(
        r#"
        INSERT INTO deploy_manifests (profile_id, symlink_plan, status)
        VALUES (?1, ?2, 'active')
        "#,
        plan.profile_id,
        symlink_json,
    )
    .execute(&db.pool)
    .await
    .context("failed to write deploy manifest")?
    .last_insert_rowid();

    // Create symlinks
    let mut created = 0usize;
    let mut skipped = 0usize;

    for entry in &plan.entries {
        let source = Path::new(&entry.source);
        let target = Path::new(&entry.target);

        if !source.exists() {
            warn!(source = %entry.source, "source file missing, skipping");
            skipped += 1;
            continue;
        }

        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create dir {}", parent.display()))?;
        }

        // Remove stale symlink if present
        if target.exists() || target.symlink_metadata().is_ok() {
            tokio::fs::remove_file(target).await.ok();
        }

        tokio::fs::symlink(&entry.source, target)
            .await
            .with_context(|| format!("failed to symlink {} -> {}", entry.source, entry.target))?;

        created += 1;
    }

    info!(
        profile_id = plan.profile_id,
        manifest_id, created, skipped, "deploy complete"
    );

    Ok(manifest_id)
}

/// Undo a deployment by removing all symlinks in a manifest.
pub async fn rollback(manifest_id: i64, db: &Db) -> anyhow::Result<()> {
    let row = sqlx::query!(
        "SELECT symlink_plan FROM deploy_manifests WHERE id = ?1",
        manifest_id
    )
    .fetch_one(&db.pool)
    .await
    .context("manifest not found")?;

    let entries: Vec<SymlinkEntry> =
        serde_json::from_str(&row.symlink_plan).context("failed to parse manifest")?;

    for entry in &entries {
        let target = Path::new(&entry.target);
        if target.symlink_metadata().is_ok() {
            tokio::fs::remove_file(target).await.ok();
        }
    }

    sqlx::query!(
        "UPDATE deploy_manifests SET status = 'rolled_back' WHERE id = ?1",
        manifest_id
    )
    .execute(&db.pool)
    .await
    .context("failed to update manifest status")?;

    info!(manifest_id, "rollback complete");
    Ok(())
}
````

## File: `crates/plugins-engine/Cargo.toml`
````toml
[package]
name = "plugins-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
domain-core    = { workspace = true }
storage-sqlite = { workspace = true }
anyhow         = { workspace = true }
thiserror      = { workspace = true }
tracing        = { workspace = true }
tokio          = { workspace = true }
serde          = { workspace = true }
sqlx           = { workspace = true }
serde_json     = { workspace = true }
````

## File: `crates/plugins-engine/src/lib.rs`
````rust
pub mod load_order;
pub mod parser;
pub mod validate;

pub use load_order::{sync_plugins, write_load_order};
pub use parser::{PluginHeader, parse_plugin_header};
pub use validate::validate_masters;
````

## File: `crates/plugins-engine/src/parser.rs`
````rust
use anyhow::Context;
use domain_core::entities::PluginKind;
use std::path::Path;

#[derive(Debug)]
pub struct PluginHeader {
    pub kind: PluginKind,
    pub masters: Vec<String>,
}

pub async fn parse_plugin_header(path: &Path) -> anyhow::Result<PluginHeader> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read plugin {}", path.display()))?;

    if bytes.len() < 24 {
        anyhow::bail!("file too small to be a valid plugin");
    }

    if &bytes[0..4] != b"TES4" {
        anyhow::bail!("not a valid Fallout 4 plugin (missing TES4 signature)");
    }

    let flags = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let esl_flag = (flags & 0x200) != 0;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let kind = match ext.as_str() {
        "esm" => PluginKind::Esm,
        "esl" => PluginKind::Esl,
        "esp" if esl_flag => PluginKind::Esl,
        _ => PluginKind::Esp,
    };

    let masters = parse_masters(&bytes);

    Ok(PluginHeader { kind, masters })
}

fn parse_masters(bytes: &[u8]) -> Vec<String> {
    let mut masters = Vec::new();
    let record_data_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let record_end = (24 + record_data_size).min(bytes.len());
    let mut pos = 24usize;

    while pos + 6 <= record_end {
        let subtype = &bytes[pos..pos + 4];
        let subsize = u16::from_le_bytes([bytes[pos + 4], bytes[pos + 5]]) as usize;
        pos += 6;

        if pos + subsize > record_end {
            break;
        }

        if subtype == b"MAST" && subsize > 0 {
            let raw = &bytes[pos..pos + subsize];
            let end = raw.iter().position(|&b| b == 0).unwrap_or(subsize);
            if let Ok(name) = std::str::from_utf8(&raw[..end]) {
                masters.push(name.to_string());
            }
        }

        pos += subsize;
    }

    masters
}
````

## File: `crates/plugins-engine/src/load_order.rs`
````rust
use anyhow::Context;
use std::path::Path;
use tracing::info;

use storage_sqlite::Db;

use crate::parser::parse_plugin_header;

/// Scan a deployed Data directory for plugins and sync them into the DB
/// for the given profile. New plugins get appended at the end of the load order.
pub async fn sync_plugins(profile_id: i64, data_dir: &Path, db: &Db) -> anyhow::Result<usize> {
    let mut added = 0usize;

    let mut read_dir = tokio::fs::read_dir(data_dir)
        .await
        .context("failed to read data directory")?;

    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !matches!(ext.as_str(), "esp" | "esm" | "esl") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Parse header for kind and masters
        let header = match parse_plugin_header(&path).await {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!(file = %filename, err = %e, "skipping invalid plugin");
                continue;
            }
        };

        let kind_str = format!("{:?}", header.kind).to_lowercase();
        let masters_json = serde_json::to_string(&header.masters).unwrap_or_else(|_| "[]".into());

        // Upsert plugin record
        let plugin_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO plugins (filename, kind, masters_json)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(filename) DO UPDATE SET
                kind = excluded.kind,
                masters_json = excluded.masters_json
            RETURNING id
            "#,
        )
        .bind(&filename)
        .bind(&kind_str)
        .bind(&masters_json)
        .fetch_one(&db.pool)
        .await
        .context("failed to upsert plugin")?;

        // Add to profile if not already present
        let already = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM profile_plugins WHERE profile_id=?1 AND plugin_id=?2",
            profile_id,
            plugin_id,
        )
        .fetch_one(&db.pool)
        .await?
            > 0;

        if !already {
            // Append at end of load order
            let next_index = sqlx::query_scalar!(
                "SELECT COALESCE(MAX(load_index), -1) + 1 FROM profile_plugins WHERE profile_id=?1",
                profile_id,
            )
            .fetch_one(&db.pool)
            .await?;

            sqlx::query!(
                "INSERT INTO profile_plugins (profile_id, plugin_id, enabled, load_index)
                 VALUES (?1, ?2, 1, ?3)",
                profile_id,
                plugin_id,
                next_index,
            )
            .execute(&db.pool)
            .await?;

            added += 1;
        }
    }

    info!(profile_id, added, "plugin sync complete");
    Ok(added)
}

/// Write the profile's load order to plugins.txt and loadorder.txt.
pub async fn write_load_order(profile_id: i64, profile_dir: &Path, db: &Db) -> anyhow::Result<()> {
    let rows = sqlx::query!(
        r#"
        SELECT p.filename, pp.enabled
        FROM profile_plugins pp
        JOIN plugins p ON p.id = pp.plugin_id
        WHERE pp.profile_id = ?1
        ORDER BY pp.load_index
        "#,
        profile_id,
    )
    .fetch_all(&db.pool)
    .await
    .context("failed to load plugin order")?;

    // plugins.txt: enabled plugins prefixed with *
    let plugins_txt = rows
        .iter()
        .map(|r| {
            if r.enabled != 0 {
                format!("*{}", r.filename)
            } else {
                r.filename.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // loadorder.txt: all plugins in order, no prefix
    let loadorder_txt = rows
        .iter()
        .map(|r| r.filename.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    tokio::fs::write(profile_dir.join("plugins.txt"), plugins_txt)
        .await
        .context("failed to write plugins.txt")?;

    tokio::fs::write(profile_dir.join("loadorder.txt"), loadorder_txt)
        .await
        .context("failed to write loadorder.txt")?;

    info!(profile_id, "load order written");
    Ok(())
}
````

## File: `crates/plugins-engine/src/validate.rs`
````rust
use anyhow::Context;
use sqlx::Row;
use std::collections::HashSet;
use storage_sqlite::Db;

#[derive(Debug)]
pub struct ValidationReport {
    pub missing_masters: Vec<MissingMaster>,
}

#[derive(Debug)]
pub struct MissingMaster {
    pub plugin: String,
    pub missing_master: String,
}

/// Check that every master a plugin depends on is present and enabled
/// in the profile's load order.
pub async fn validate_masters(profile_id: i64, db: &Db) -> anyhow::Result<ValidationReport> {
    let rows = sqlx::query(
        r#"
        SELECT p.filename, p.masters_json
        FROM profile_plugins pp
        JOIN plugins p ON p.id = pp.plugin_id
        WHERE pp.profile_id = ?1 AND pp.enabled = 1
        ORDER BY pp.load_index
        "#,
    )
    .bind(profile_id)
    .fetch_all(&db.pool)
    .await?;

    let enabled: HashSet<String> = rows
        .iter()
        .map(|r| r.try_get::<String, _>("filename"))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|name| name.to_lowercase())
        .collect();

    let mut missing_masters = Vec::new();
    let mut seen_pairs = HashSet::new();

    for row in rows {
        let plugin: String = row.try_get("filename")?;
        let masters_json: String = row.try_get("masters_json")?;
        let masters: Vec<String> = serde_json::from_str(&masters_json)
            .with_context(|| format!("invalid masters_json for plugin {plugin}"))?;

        for master in masters {
            if enabled.contains(&master.to_lowercase()) {
                continue;
            }

            let key = (plugin.to_lowercase(), master.to_lowercase());
            if seen_pairs.insert(key) {
                missing_masters.push(MissingMaster {
                    plugin: plugin.clone(),
                    missing_master: master,
                });
            }
        }
    }

    Ok(ValidationReport { missing_masters })
}
````

## File: `crates/loot-engine/Cargo.toml`
````toml
[package]
name = "loot-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
storage-sqlite = { workspace = true }
anyhow         = { workspace = true }
tracing        = { workspace = true }
sqlx           = { workspace = true }
libloot        = { workspace = true }
````

## File: `crates/loot-engine/src/lib.rs`
````rust
use std::path::{Path, PathBuf};

use anyhow::Context;
use libloot::{Game, GameType};
use sqlx::Row;
use storage_sqlite::Db;
use tracing::{info, warn};

#[derive(Debug, Clone)]
struct PluginRow {
    plugin_id: i64,
    filename: String,
    kind: String,
    enabled: bool,
    load_index: i64,
}

#[derive(Debug)]
pub struct SortResult {
    pub engine: String,
    pub order: Vec<String>,
}

pub async fn sort_profile_plugins(profile_id: i64, db: &Db) -> anyhow::Result<SortResult> {
    let rows = sqlx::query(
        r#"
        SELECT
            i.install_path AS game_install_path,
            pp.plugin_id AS plugin_id,
            p.filename AS filename,
            p.kind AS kind,
            pp.enabled AS enabled,
            pp.load_index AS load_index
        FROM profiles pr
        JOIN instances i ON i.id = pr.instance_id
        JOIN profile_plugins pp ON pp.profile_id = pr.id
        JOIN plugins p ON p.id = pp.plugin_id
        WHERE pr.id = ?1
        ORDER BY pp.load_index
        "#,
    )
    .bind(profile_id)
    .fetch_all(&db.pool)
    .await
    .context("failed to load profile plugins for sorting")?;

    if rows.is_empty() {
        return Ok(SortResult {
            engine: "libloot".to_string(),
            order: Vec::new(),
        });
    }

    let game_install_path: String = rows[0].try_get("game_install_path")?;
    let mut plugins = rows
        .into_iter()
        .map(|row| -> anyhow::Result<PluginRow> {
            Ok(PluginRow {
                plugin_id: row.try_get("plugin_id")?,
                filename: row.try_get("filename")?,
                kind: row.try_get("kind")?,
                enabled: row.try_get::<i64, _>("enabled")? != 0,
                load_index: row.try_get("load_index")?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let (engine, sorted_names) = match sort_with_libloot(&game_install_path, &plugins) {
        Ok(order) => ("libloot".to_string(), order),
        Err(e) => {
            warn!(profile_id, err = %e, "libloot sort failed; falling back to deterministic sort");
            plugins.sort_by(|a, b| {
                b.enabled
                    .cmp(&a.enabled)
                    .then(kind_rank(&a.kind).cmp(&kind_rank(&b.kind)))
                    .then(a.filename.to_lowercase().cmp(&b.filename.to_lowercase()))
            });
            (
                "libloot_fallback".to_string(),
                plugins.iter().map(|p| p.filename.clone()).collect(),
            )
        }
    };

    let order_map = sorted_names
        .iter()
        .enumerate()
        .map(|(idx, name)| (name.to_lowercase(), idx))
        .collect::<std::collections::HashMap<_, _>>();
    plugins.sort_by(|a, b| {
        let ai = order_map
            .get(&a.filename.to_lowercase())
            .copied()
            .unwrap_or(usize::MAX);
        let bi = order_map
            .get(&b.filename.to_lowercase())
            .copied()
            .unwrap_or(usize::MAX);
        ai.cmp(&bi).then(a.load_index.cmp(&b.load_index))
    });

    let mut tx = db
        .pool
        .begin()
        .await
        .context("failed to begin plugin sort transaction")?;

    sqlx::query(
        "UPDATE profile_plugins SET load_index = load_index + 1000000 WHERE profile_id = ?1",
    )
    .bind(profile_id)
    .execute(&mut *tx)
    .await
    .context("failed to reserve load indexes for sorting")?;

    for (idx, plugin) in plugins.iter().enumerate() {
        sqlx::query(
            "UPDATE profile_plugins SET load_index = ?1 WHERE profile_id = ?2 AND plugin_id = ?3",
        )
        .bind(idx as i64)
        .bind(profile_id)
        .bind(plugin.plugin_id)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("failed to set load index for {}", plugin.filename))?;
    }

    tx.commit().await.context("failed to commit plugin sort")?;

    let order = plugins.into_iter().map(|p| p.filename).collect::<Vec<_>>();
    info!(
        profile_id,
        count = order.len(),
        engine,
        "plugin sorting complete"
    );
    Ok(SortResult { engine, order })
}

fn sort_with_libloot(
    game_install_path: &str,
    plugins: &[PluginRow],
) -> anyhow::Result<Vec<String>> {
    let install_path = Path::new(game_install_path);
    let data_dir = install_path.join("Data");

    let mut game = Game::with_local_path(GameType::Fallout4, install_path, install_path)
        .map_err(|e| anyhow::anyhow!("failed to initialise libloot game handle: {e}"))?;

    let mut loaded_paths = Vec::<PathBuf>::new();
    let mut loaded_names = Vec::<String>::new();
    let mut missing_names = Vec::<String>::new();

    for plugin in plugins {
        let path = data_dir.join(&plugin.filename);
        if path.is_file() {
            loaded_paths.push(path);
            loaded_names.push(plugin.filename.clone());
        } else {
            missing_names.push(plugin.filename.clone());
        }
    }

    if loaded_paths.is_empty() {
        anyhow::bail!("no plugin files exist in {}", data_dir.display());
    }

    let path_refs = loaded_paths
        .iter()
        .map(PathBuf::as_path)
        .collect::<Vec<_>>();
    game.load_plugin_headers(&path_refs)
        .map_err(|e| anyhow::anyhow!("libloot failed to load plugin headers: {e}"))?;

    let name_refs = loaded_names.iter().map(String::as_str).collect::<Vec<_>>();
    let mut sorted = game
        .sort_plugins(&name_refs)
        .map_err(|e| anyhow::anyhow!("libloot failed to sort plugins: {e}"))?;

    if !missing_names.is_empty() {
        sorted.extend(missing_names);
    }

    Ok(sorted)
}

fn kind_rank(kind: &str) -> i32 {
    match kind.to_ascii_lowercase().as_str() {
        "esm" => 0,
        "esp" => 1,
        "esl" => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::kind_rank;

    #[test]
    fn kind_rank_orders_known_types_first() {
        assert!(kind_rank("esm") < kind_rank("esp"));
        assert!(kind_rank("esp") < kind_rank("esl"));
        assert!(kind_rank("esl") < kind_rank("unknown"));
    }
}
````

## File: `crates/launch-engine/Cargo.toml`
````toml
[package]
name = "launch-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
storage-sqlite = { workspace = true }
anyhow         = { workspace = true }
tracing        = { workspace = true }
sqlx           = { workspace = true }

[dev-dependencies]
tokio          = { workspace = true }
````

## File: `crates/launch-engine/src/lib.rs`
````rust
use std::path::{Path, PathBuf};

use anyhow::Context;
use sqlx::Row;
use storage_sqlite::Db;
use tracing::info;

#[derive(Debug)]
pub struct LaunchPreflight {
    pub profile_id: i64,
    pub runner_kind: String,
    pub game_install_path: String,
    pub f4se_available: bool,
}

#[derive(Debug)]
pub struct LaunchResult {
    pub profile_id: i64,
    pub runner_kind: String,
    pub executable: String,
    pub pid: u32,
}

pub async fn preflight_launch(
    profile_id: i64,
    use_f4se: bool,
    db: &Db,
) -> anyhow::Result<LaunchPreflight> {
    let row = sqlx::query(
        r#"
        SELECT
            p.id AS profile_id,
            p.instance_id AS instance_id,
            p.pinned_runner_id AS pinned_runner_id,
            i.install_path AS game_install_path,
            rc.kind AS runner_kind,
            rc.install_path AS runner_install_path,
            rc.verified AS runner_verified
        FROM profiles p
        JOIN instances i ON i.id = p.instance_id
        LEFT JOIN runner_catalog rc ON rc.id = p.pinned_runner_id
        WHERE p.id = ?1
        "#,
    )
    .bind(profile_id)
    .fetch_optional(&db.pool)
    .await
    .context("failed to load launch context")?;

    let row = row.with_context(|| format!("profile {profile_id} was not found"))?;

    let instance_id: i64 = row.try_get("instance_id")?;
    let pinned_runner_id: Option<i64> = row.try_get("pinned_runner_id")?;
    let runner_kind: Option<String> = row.try_get("runner_kind")?;
    let runner_install_path: Option<String> = row.try_get("runner_install_path")?;
    let runner_verified: Option<i64> = row.try_get("runner_verified")?;
    let game_install_path: String = row.try_get("game_install_path")?;

    if pinned_runner_id.is_none() {
        anyhow::bail!("profile {profile_id} has no pinned runner");
    }

    let runner_kind = runner_kind
        .with_context(|| format!("pinned runner for profile {profile_id} was not found"))?;
    let runner_install_path = runner_install_path.with_context(|| {
        format!("pinned runner install path for profile {profile_id} was not found")
    })?;
    if runner_verified.unwrap_or(0) == 0 {
        anyhow::bail!("pinned runner for profile {profile_id} is not verified");
    }

    let runner_path = Path::new(&runner_install_path);
    match runner_kind.as_str() {
        "proton" => {
            let proton_bin = if runner_path.is_dir() {
                runner_path.join("proton")
            } else {
                runner_path.to_path_buf()
            };
            if !proton_bin.is_file() {
                anyhow::bail!("proton binary not found at {}", proton_bin.display());
            }
        }
        "wine" => {
            if !runner_path.is_file() {
                anyhow::bail!("wine binary not found at {}", runner_path.display());
            }
        }
        other => anyhow::bail!("unsupported runner kind: {other}"),
    }

    let game_exe = Path::new(&game_install_path).join("Fallout4.exe");
    if !game_exe.is_file() {
        anyhow::bail!("game executable not found at {}", game_exe.display());
    }

    let f4se_loader = Path::new(&game_install_path).join("f4se_loader.exe");
    let f4se_available = f4se_loader.is_file();
    if use_f4se && !f4se_available {
        anyhow::bail!("F4SE requested but f4se_loader.exe was not found");
    }

    sqlx::query(
        r#"
        INSERT INTO f4se_state (instance_id, detected_version, compatible, last_checked)
        VALUES (?1, NULL, ?2, datetime('now'))
        ON CONFLICT(instance_id) DO UPDATE SET
            detected_version = excluded.detected_version,
            compatible = excluded.compatible,
            last_checked = excluded.last_checked
        "#,
    )
    .bind(instance_id)
    .bind(if f4se_available { 1_i64 } else { 0_i64 })
    .execute(&db.pool)
    .await
    .context("failed to update f4se_state")?;

    Ok(LaunchPreflight {
        profile_id,
        runner_kind,
        game_install_path,
        f4se_available,
    })
}

pub async fn launch_game(profile_id: i64, use_f4se: bool, db: &Db) -> anyhow::Result<LaunchResult> {
    launch_internal(profile_id, use_f4se, false, db).await
}

pub async fn launch_launcher(profile_id: i64, db: &Db) -> anyhow::Result<LaunchResult> {
    launch_internal(profile_id, false, true, db).await
}

async fn launch_internal(
    profile_id: i64,
    use_f4se: bool,
    use_launcher: bool,
    db: &Db,
) -> anyhow::Result<LaunchResult> {
    let preflight = preflight_launch(profile_id, use_f4se, db).await?;

    let row = sqlx::query(
        r#"
        SELECT rc.kind AS runner_kind, rc.install_path AS runner_install_path
        FROM profiles p
        JOIN runner_catalog rc ON rc.id = p.pinned_runner_id
        WHERE p.id = ?1
        "#,
    )
    .bind(profile_id)
    .fetch_one(&db.pool)
    .await
    .context("failed to load runner for launch")?;

    let runner_kind: String = row.try_get("runner_kind")?;
    let runner_install_path: String = row.try_get("runner_install_path")?;

    let game_install_path = PathBuf::from(&preflight.game_install_path);
    let executable = if use_launcher {
        game_install_path.join("Fallout4Launcher.exe")
    } else if use_f4se && preflight.f4se_available {
        game_install_path.join("f4se_loader.exe")
    } else {
        game_install_path.join("Fallout4.exe")
    };
    if !executable.is_file() {
        anyhow::bail!("launch executable not found at {}", executable.display());
    }

    let mut command = match runner_kind.as_str() {
        "proton" => {
            let runner_path = Path::new(&runner_install_path);
            let proton_bin = if runner_path.is_dir() {
                runner_path.join("proton")
            } else {
                runner_path.to_path_buf()
            };
            if !proton_bin.is_file() {
                anyhow::bail!("proton binary not found at {}", proton_bin.display());
            }

            let mut cmd = std::process::Command::new(proton_bin);
            cmd.arg("run").arg(&executable);
            cmd
        }
        "wine" => {
            let runner_path = Path::new(&runner_install_path);
            if !runner_path.is_file() {
                anyhow::bail!("wine binary not found at {}", runner_path.display());
            }

            let mut cmd = std::process::Command::new(runner_path);
            cmd.arg(&executable);
            cmd
        }
        other => anyhow::bail!("unsupported runner kind: {other}"),
    };

    command.current_dir(&game_install_path);
    let child = command.spawn().with_context(|| {
        format!(
            "failed to launch {} via {}",
            executable.display(),
            runner_kind
        )
    })?;
    let pid = child.id();

    info!(
        profile_id,
        pid,
        runner_kind = %runner_kind,
        executable = %executable.display(),
        "game launched"
    );

    Ok(LaunchResult {
        profile_id,
        runner_kind,
        executable: executable.display().to_string(),
        pid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    async fn setup_profile(
        with_f4se: bool,
        create_runner_binary: bool,
    ) -> anyhow::Result<(Db, i64, PathBuf)> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after epoch")
            .as_nanos();
        let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("mm-launch-engine-tests-{nonce}-{seq}"));
        fs::create_dir_all(&root)?;

        let game_path = root.join("game");
        fs::create_dir_all(&game_path)?;
        touch(&game_path.join("Fallout4.exe"))?;
        if with_f4se {
            touch(&game_path.join("f4se_loader.exe"))?;
        }

        let runner_path = root.join("wine");
        if create_runner_binary {
            touch(&runner_path)?;
        }

        let db = Db::open(&root.join("state.db")).await?;
        let game_id: i64 = sqlx::query_scalar("SELECT id FROM games WHERE canonical_id = ?1")
            .bind("fallout4")
            .fetch_one(&db.pool)
            .await?;

        let instance_id: i64 = sqlx::query_scalar(
            "INSERT INTO instances (game_id, label, source_type, install_path) VALUES (?1, ?2, ?3, ?4) RETURNING id",
        )
        .bind(game_id)
        .bind("Test FO4")
        .bind("manual")
        .bind(game_path.display().to_string())
        .fetch_one(&db.pool)
        .await?;

        let runner_id: i64 = sqlx::query_scalar(
            "INSERT INTO runner_catalog (kind, version, install_path, verified) VALUES (?1, ?2, ?3, 1) RETURNING id",
        )
        .bind("wine")
        .bind("system")
        .bind(runner_path.display().to_string())
        .fetch_one(&db.pool)
        .await?;

        let profile_id: i64 = sqlx::query_scalar(
            "INSERT INTO profiles (instance_id, name, pinned_runner_id) VALUES (?1, ?2, ?3) RETURNING id",
        )
        .bind(instance_id)
        .bind("Default")
        .bind(runner_id)
        .fetch_one(&db.pool)
        .await?;

        Ok((db, profile_id, root))
    }

    fn touch(path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, b"")?;
        Ok(())
    }

    #[tokio::test]
    async fn preflight_succeeds_and_updates_f4se_state() {
        let (db, profile_id, root) = setup_profile(false, true).await.expect("setup should work");

        let preflight = preflight_launch(profile_id, false, &db)
            .await
            .expect("preflight should succeed");
        assert_eq!(preflight.runner_kind, "wine");
        assert!(!preflight.f4se_available);

        let compatible: Option<i64> = sqlx::query_scalar(
            "SELECT fs.compatible FROM f4se_state fs JOIN profiles p ON p.instance_id = fs.instance_id WHERE p.id = ?1",
        )
        .bind(profile_id)
        .fetch_optional(&db.pool)
        .await
        .expect("f4se_state query should succeed");
        assert_eq!(compatible, Some(0));

        fs::remove_dir_all(root).expect("cleanup should work");
    }

    #[tokio::test]
    async fn preflight_fails_if_f4se_requested_but_missing() {
        let (db, profile_id, root) = setup_profile(false, true).await.expect("setup should work");

        let err = preflight_launch(profile_id, true, &db)
            .await
            .expect_err("expected preflight to fail");
        assert!(err.to_string().contains("F4SE requested"));

        fs::remove_dir_all(root).expect("cleanup should work");
    }

    #[tokio::test]
    async fn preflight_fails_if_runner_binary_missing() {
        let (db, profile_id, root) = setup_profile(false, false)
            .await
            .expect("setup should work");

        let err = preflight_launch(profile_id, false, &db)
            .await
            .expect_err("expected preflight to fail");
        assert!(err.to_string().contains("wine binary not found"));

        fs::remove_dir_all(root).expect("cleanup should work");
    }
}
````

## File: `crates/ipc-api/Cargo.toml`
````toml
[package]
name = "ipc-api"
version = "0.1.0"
edition = "2024"

[dependencies]
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
game-detect = { workspace = true }
````

## File: `crates/ipc-api/src/lib.rs`
````rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum Request {
    Ping,
    DetectGame,
    RegisterGame {
        path: String,
    },
    ListRunners,
    PinRunner {
        profile_id: i64,
        runner_id: i64,
    },
    ListProfilePlugins {
            profile_id: i64,
        },
        SetProfilePluginEnabled {
            profile_id: i64,
            plugin_id: i64,
            enabled: bool,
        },
    ListProfileMods {
        profile_id: i64,
    },
    UpsertProfileMod {
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
        priority: i32,
    },
    SetProfileModEnabled {
        profile_id: i64,
        mod_id: i64,
        enabled: bool,
    },
    SetProfileModPriority {
        profile_id: i64,
        mod_id: i64,
        priority: i32,
    },
    CreateProfile {
        instance_id: i64,
        name: String,
    },
    ListProfiles {
        instance_id: i64,
    },
    DeleteProfile {
        profile_id: i64,
    },
    IngestMod {
        archive_path: String,
    },
    DeployPreview {
        profile_id: i64,
        game_data_dir: String,
    },
    DeployApply {
        profile_id: i64,
        game_data_dir: String,
    },
    DeployRollback {
        manifest_id: i64,
    },
    SyncPlugins {
        profile_id: i64,
        data_dir: String,
    },
    ValidatePlugins {
        profile_id: i64,
    },
    SortWithLoot {
        profile_id: i64,
    },
    LaunchPreflight {
        profile_id: i64,
        use_f4se: bool,
    },
    LaunchGame {
        profile_id: i64,
        use_f4se: bool,
    },
    LaunchLauncher {
        profile_id: i64,
    },
    WriteLoadOrder {
        profile_id: i64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum Response {
    Pong {
        version: String,
    },
    GameDetected {
        instance_id: i64,
        install_path: String,
        source: String,
    },
    RunnerList {
        runners: Vec<RunnerInfo>,
    },
    RunnerPinned {
        profile_id: i64,
        runner_id: i64,
    },
    ProfilePlugins {
            profile_id: i64,
            plugins: Vec<ProfilePluginInfo>,
        },
        ProfilePluginUpdated {
            profile_id: i64,
            plugin_id: i64,
        },
    ProfileMods {
        profile_id: i64,
        mods: Vec<ProfileModInfo>,
    },
    ProfileModUpdated {
        profile_id: i64,
        mod_id: i64,
    },
    ProfileCreated {
        profile_id: i64,
    },
    ProfileList {
        profiles: Vec<ProfileInfo>,
    },
    ProfileDeleted {
        profile_id: i64,
    },
    ModIngested {
        mod_id: i64,
        name: String,
        file_count: usize,
    },
    DeployPreview {
        profile_id: i64,
        entry_count: usize,
        entries: Vec<String>,
    },
    DeployApplied {
        manifest_id: i64,
    },
    RolledBack {
        manifest_id: i64,
    },
    PluginsSynced {
        added: usize,
    },
    PluginsValid {
        missing_masters: Vec<String>,
    },
    PluginsSorted {
        profile_id: i64,
        engine: String,
        order: Vec<String>,
    },
    LaunchPreflight {
        profile_id: i64,
        runner_kind: String,
        game_install_path: String,
        f4se_available: bool,
    },
    GameLaunched {
        profile_id: i64,
        runner_kind: String,
        executable: String,
        pid: u32,
    },
    LoadOrderWritten,
    Ok,
    Error {
        code: ErrorCode,
        message: String,
    },
}

/// Lightweight profile summary sent over IPC (no need for the full entity).
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileInfo {
    pub id: i64,
    pub name: String,
    pub auto_deploy: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunnerInfo {
    pub id: i64,
    pub kind: String,
    pub version: String,
    pub install_path: String,
    pub verified: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileModInfo {
    pub mod_id: i64,
    pub mod_name: String,
    pub enabled: bool,
    pub priority: i32,
}

#[derive(Debug, Deserialize, Serialize, thiserror::Error)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    #[error("unknown method")]
    UnknownMethod,
    #[error("invalid request payload")]
    InvalidRequest,
    #[error("internal daemon error")]
    Internal,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfilePluginInfo {
    pub plugin_id: i64,
    pub filename: String,
    pub kind: String,
    pub enabled: bool,
    pub load_index: i32,
}
````

## File: `ui/kirigami-app/CMakeLists.txt`
````cmake
cmake_minimum_required(VERSION 3.20)
project(ModManagerKirigamiApp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Network Qml Quick)
find_package(KF6Kirigami REQUIRED)

qt_standard_project_setup()

qt_add_executable(mod-manager-ui
    src/main.cpp
    src/ipc_client.cpp
    src/ipc_client.h
)

qt_add_qml_module(mod-manager-ui
    URI ModManager
    VERSION 1.0
    QML_FILES
        src/Main.qml
)

target_link_libraries(mod-manager-ui
    PRIVATE
        Qt6::Core
        Qt6::Gui
        Qt6::Network
        Qt6::Qml
        Qt6::Quick
        KF6::Kirigami
)
````

## File: `ui/kirigami-app/src/main.cpp`
````cpp
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <qqml.h>

#include "ipc_client.h"

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    QCoreApplication::setOrganizationName(QStringLiteral("ModManager"));
    QCoreApplication::setOrganizationDomain(QStringLiteral("modmanager.local"));
    QCoreApplication::setApplicationName(QStringLiteral("ModManager"));

    QQmlApplicationEngine engine;
    qmlRegisterType<IpcClient>("ModManager", 1, 0, "IpcClient");
    QObject::connect(
        &engine,
        &QQmlApplicationEngine::objectCreationFailed,
        &app,
        []() { QCoreApplication::exit(-1); },
        Qt::QueuedConnection
    );

    engine.loadFromModule(QStringLiteral("ModManager"), QStringLiteral("Main"));
    return app.exec();
}
````

## File: `ui/kirigami-app/src/ipc_client.h`
````cpp
#pragma once

#include <QObject>
#include <QVariant>

class IpcClient : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString socketPath READ socketPath WRITE setSocketPath NOTIFY socketPathChanged)

public:
    explicit IpcClient(QObject *parent = nullptr);

    QString socketPath() const;
    void setSocketPath(const QString &value);

    Q_INVOKABLE void call(const QString &method, const QVariantMap &params = QVariantMap());

signals:
    void socketPathChanged();
    void responseReceived(const QVariant &response);
    void requestFailed(const QString &message);

private:
    QString m_socketPath;
};
````

## File: `ui/kirigami-app/src/ipc_client.cpp`
````cpp
#include "ipc_client.h"

#include <QElapsedTimer>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLocalSocket>

IpcClient::IpcClient(QObject *parent)
    : QObject(parent),
      m_socketPath("/tmp/mm-daemon.sock") {}

QString IpcClient::socketPath() const {
    return m_socketPath;
}

void IpcClient::setSocketPath(const QString &value) {
    if (m_socketPath == value) {
        return;
    }

    m_socketPath = value;
    emit socketPathChanged();
}

void IpcClient::call(const QString &method, const QVariantMap &params) {
    if (method.trimmed().isEmpty()) {
        emit requestFailed("Method must not be empty");
        return;
    }

    QJsonObject request;
    request.insert("method", method);
    if (!params.isEmpty()) {
        request.insert("params", QJsonObject::fromVariantMap(params));
    }

    QLocalSocket socket;
    socket.connectToServer(m_socketPath);
    if (!socket.waitForConnected(3000)) {
        emit requestFailed(QString("Failed to connect to %1: %2").arg(m_socketPath, socket.errorString()));
        return;
    }

    QByteArray encoded = QJsonDocument(request).toJson(QJsonDocument::Compact);
    encoded.append('\n');

    if (socket.write(encoded) == -1 || !socket.waitForBytesWritten(3000)) {
        emit requestFailed(QString("Failed to send request: %1").arg(socket.errorString()));
        return;
    }

    QByteArray buffer;
    QElapsedTimer timer;
    timer.start();

    while (timer.elapsed() < 6000) {
        if (!socket.waitForReadyRead(250)) {
            continue;
        }

        buffer.append(socket.readAll());
        const int newline = buffer.indexOf('\n');
        if (newline == -1) {
            continue;
        }

        const QByteArray line = buffer.left(newline).trimmed();
        if (line.isEmpty()) {
            emit requestFailed("Received empty response line from daemon");
            return;
        }

        QJsonParseError parseError;
        const QJsonDocument doc = QJsonDocument::fromJson(line, &parseError);
        if (parseError.error != QJsonParseError::NoError) {
            emit requestFailed(QString("Failed to parse daemon response: %1").arg(parseError.errorString()));
            return;
        }

        emit responseReceived(doc.toVariant());
        return;
    }

    emit requestFailed("Timed out waiting for daemon response");
}
````

## File: `ui/kirigami-app/src/Main.qml`
````qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtCore
import org.kde.kirigami as Kirigami
import ModManager 1.0

Kirigami.ApplicationWindow {
    id: root
    width: 1400
    height: 900
    visible: true
    title: "Mod Manager - Fallout 4"

    property string activeGameDataDir: ""
    property bool pendingLaunch: false
    property bool useF4seCache: true // Toggles whether to use F4SE on launch

    // --- State Models ---
    ListModel { id: profilesModel }
    ListModel { id: executablesModel }
    ListModel { id: modPriorityModel } // Left pane
    ListModel { id: pluginsModel }     // Right pane (Plugins tab)

    // --- Main Layout ---
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // 1. Top Toolbar (Matches MO2 Top Bar)
        ToolBar {
            Layout.fillWidth: true
            RowLayout {
                anchors.fill: parent
                anchors.margins: Kirigami.Units.smallSpacing

                Label { text: "Profile:" }
                ComboBox {
                    id: profileComboBox
                    Layout.preferredWidth: 200
                    model: profilesModel
                    textRole: "name"
                    onActivated: root.loadProfileData(currentValue)
                }

                Button {
                    icon.name: "configure"
                    text: "Configure"
                }

                Item { Layout.fillWidth: true } // Spacer

                // Run Section
                ComboBox {
                    id: executableComboBox
                    Layout.preferredWidth: 200
                    model: ["Fallout 4", "F4SE", "Fallout 4 Launcher"]
                }
                Button {
                                    text: "Run"
                                    icon.name: "media-playback-start"
                                    highlighted: true
                                    Layout.preferredWidth: 100
                                    onClicked: {
                                        const profileId = root.currentProfileId();
                                        if (profileId < 0) return;
                                        
                                        // Parse dropdown for F4SE
                                        useF4seCache = (executableComboBox.currentText === "F4SE");
                                        
                                        if (activeGameDataDir === "") {
                                            appendLog("Error: Game not detected. Cannot deploy.");
                                            return;
                                        }
                
                                        appendLog("Preparing Virtual File System...");
                                        pendingLaunch = true;
                                        
                                        // This triggers the deploy. When "deploy_applied" is received, 
                                        // handleIpcResponse will automatically trigger "launch_game".
                                        root.send("deploy_apply", {
                                            profile_id: profileId,
                                            game_data_dir: activeGameDataDir
                                        });
                                    }
                                }
            }
        }

        // 2. Main Dual-Pane Area
        SplitView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            orientation: Qt.Horizontal

            // LEFT PANE: Mod Priority List
            Frame {
                SplitView.fillWidth: true
                SplitView.preferredWidth: root.width * 0.6
                padding: 0

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0
                    
                    // Header Row
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 30
                        color: Kirigami.Theme.alternateBackgroundColor
                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 4
                            Label { text: "Mod Name"; Layout.fillWidth: true }
                            Label { text: "Conflicts"; Layout.preferredWidth: 60 }
                            Label { text: "Category"; Layout.preferredWidth: 120 }
                            Label { text: "Priority"; Layout.preferredWidth: 60 }
                        }
                    }

                    // Mod List
                    ListView {
                        id: modListView
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        model: modPriorityModel
                        clip: true
                        boundsBehavior: Flickable.StopAtBounds

                        delegate: Rectangle {
                            width: modListView.width
                            height: 28
                            color: index % 2 === 0 ? "transparent" : Kirigami.Theme.alternateBackgroundColor

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 4
                                anchors.rightMargin: 4

                                CheckBox {
                                    checked: model.enabled
                                    onToggled: root.toggleMod(model.modId, checked)
                                }
                                Label { 
                                    text: model.modName
                                    Layout.fillWidth: true
                                    font.bold: model.isSeparator // Bold if it's a category separator
                                }
                                // Conflict Icons placeholder
                                Label { text: model.conflicts ? "⚡" : ""; Layout.preferredWidth: 60 }
                                Label { text: model.category; Layout.preferredWidth: 120 }
                                Label { text: model.priority; Layout.preferredWidth: 60 }
                            }
                        }
                    }
                }
            }

            // RIGHT PANE: Plugins / Data
            Frame {
                SplitView.preferredWidth: root.width * 0.4
                padding: 0

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    TabBar {
                        id: rightTabBar
                        Layout.fillWidth: true
                        TabButton { text: "Plugins" }
                        TabButton { text: "Data (VFS)" }
                        TabButton { text: "Downloads" }
                    }

                    StackLayout {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        currentIndex: rightTabBar.currentIndex

                        // Tab 1: Plugins
                        Item {
                            ColumnLayout {
                                anchors.fill: parent
                                Button {
                                    text: "Sort with LOOT"
                                    icon.name: "view-sort-ascending"
                                    Layout.fillWidth: true
                                    onClicked: root.send("sort_with_loot", { profile_id: currentProfileId() })
                                }
                                ListView {
                                                                    id: pluginListView
                                                                    Layout.fillWidth: true
                                                                    Layout.fillHeight: true
                                                                    model: pluginsModel
                                                                    clip: true
                                                                    delegate: CheckBox {
                                                                        text: `[${model.loadIndex}] ${model.filename} (${model.kind.toUpperCase()})`
                                                                        checked: model.enabled
                                                                        onToggled: {
                                                                            root.send("set_profile_plugin_enabled", {
                                                                                profile_id: root.currentProfileId(),
                                                                                plugin_id: model.pluginId,
                                                                                enabled: checked
                                                                            })
                                                                        }
                                                                    }
                                                                }
                                }
                            }

                        // Tab 2: Data (Virtual File System)
                        Item {
                            Label {
                                anchors.centerIn: parent
                                text: "File tree view goes here\n(Requires backend VFS API)"
                                horizontalAlignment: Text.AlignHCenter
                            }
                        }
                    }
                }
            }
        }

        // 3. Bottom Area: Logs & Conflict Details (Matches MO2 bottom)
        SplitView {
            Layout.fillWidth: true
            Layout.preferredHeight: 150
            orientation: Qt.Horizontal

            TextArea {
                id: logArea
                SplitView.fillWidth: true
                SplitView.preferredWidth: root.width * 0.6
                readOnly: true
                font.family: "monospace"
                text: "Mod Manager Initialized...\n"
            }

            TextArea {
                id: conflictDetailArea
                SplitView.fillWidth: true
                SplitView.preferredWidth: root.width * 0.4
                readOnly: true
                placeholderText: "Select a mod to view conflict details..."
            }
        }
    }

    // --- IPC Client Setup ---
    IpcClient {
        id: ipcClient
        onResponseReceived: (response) => root.handleIpcResponse(response)
    }

    function send(method, params) {
        ipcClient.call(method, params)
    }

    function appendLog(msg) {
        logArea.text += `[${new Date().toLocaleTimeString()}] ${msg}\n`
    }
    
    // Auto-fetch data on boot
    Component.onCompleted: {
        // Assume instance ID 1 for MVP if you only have one game installed
        send("list_profiles", { instance_id: 1 }) 
    }

    // Process responses to populate the UI automatically
    function handleIpcResponse(response) {
        if(response.type === "profile_list") {
            profilesModel.clear()
            response.payload.profiles.forEach(p => profilesModel.append(p))
            if(profilesModel.count > 0) {
                profileComboBox.currentIndex = 0
                loadProfileData(profilesModel.get(0).id)
            }
        }
        else if(response.type === "profile_mods") {
            modPriorityModel.clear()
            response.payload.mods.forEach(m => modPriorityModel.append({
                modId: m.mod_id,
                modName: m.mod_name,
                enabled: m.enabled,
                priority: m.priority,
                category: "Unassigned", // Placeholder until backend supports categories
                conflicts: false        // Placeholder until backend supports conflict checks
            }))
        }

        else if (response.type === "game_detected") {
                    // Save this so we know where to deploy
                    activeGameDataDir = response.payload.install_path + "/Data";
                    appendLog(`Linked game at: ${response.payload.install_path}`);
                }
                else if(response.type === "profile_plugins") {
                    pluginsModel.clear()
                    response.payload.plugins.forEach(p => pluginsModel.append({
                        pluginId: p.plugin_id,
                        filename: p.filename,
                        kind: p.kind,
                        enabled: p.enabled,
                        loadIndex: p.load_index
                    }))
                }
                else if (response.type === "profile_plugin_updated") {
                    // Refresh list to ensure load order indexes are correct
                    send("list_profile_plugins", { profile_id: root.currentProfileId() })
                }
                else if (response.type === "plugins_sorted") {
                    appendLog(`Plugins sorted successfully via ${response.payload.engine}.`)
                    send("write_load_order", { profile_id: root.currentProfileId() })
                    send("list_profile_plugins", { profile_id: root.currentProfileId() })
                }
                else if (response.type === "deploy_applied") {
                    appendLog(`Virtual File System Deployed (Manifest ${response.payload.manifest_id})`);
                    
                    // Auto-Launch chained event
                    if (pendingLaunch) {
                        pendingLaunch = false;
                        appendLog("Starting game executable...");
                        send("launch_game", { 
                            profile_id: root.currentProfileId(), 
                            use_f4se: useF4seCache 
                        });
                    }
                }
        // Handle other responses...
    }

    function currentProfileId() {
        if(profileComboBox.currentIndex >= 0)
            return profilesModel.get(profileComboBox.currentIndex).id
        return -1
    }

    function loadProfileData(profileId) {
            appendLog(`Loading data for profile ${profileId}`)
            send("list_profile_mods", { profile_id: profileId })
            send("list_profile_plugins", { profile_id: profileId })
        }
}
````

