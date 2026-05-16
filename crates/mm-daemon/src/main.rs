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
    Db, instance_repo::InstanceRepo, profile_env_repo::ProfileEnvRepo, profile_mod_repo::ProfileModRepo, profile_repo::ProfileRepo, profile_plugin_repo::ProfilePluginRepo,
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
            // Fetch profile environment variables
            let env_repo = ProfileEnvRepo::new(&state.db.pool);
            let env_vars = match env_repo.list(profile_id).await {
                Ok(vars) => vars,
                Err(e) => {
                    warn!(profile_id, "failed to load env vars, continuing without them: {}", e);
                    vec![]
                }
            };
            match launch_game(profile_id, use_f4se, env_vars, &state.db).await {
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
            // Fetch profile environment variables
            let env_repo = ProfileEnvRepo::new(&state.db.pool);
            let env_vars = match env_repo.list(profile_id).await {
                Ok(vars) => vars,
                Err(e) => {
                    warn!(profile_id, "failed to load env vars, continuing without them: {}", e);
                    vec![]
                }
            };
            match launch_launcher(profile_id, env_vars, &state.db).await {
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

        Request::ListProfileEnvVars { profile_id } => {
            info!(profile_id, "ListProfileEnvVars");
            let repo = ProfileEnvRepo::new(&state.db.pool);
            match repo.list(profile_id).await {
                Ok(vars) => {
                    let env_vars = vars.into_iter().map(|(key, value)| ipc_api::ProfileEnvVarInfo { key, value }).collect();
                    Response::ProfileEnvVars { profile_id, env_vars }
                }
                Err(e) => err(e),
            }
        }

        Request::SetProfileEnvVar { profile_id, key, value } => {
            info!(profile_id, key, "SetProfileEnvVar");
            let repo = ProfileEnvRepo::new(&state.db.pool);
            match repo.set(profile_id, &key, &value).await {
                Ok(()) => Response::Ok,
                Err(e) => err(e),
            }
        }

        Request::DeleteProfileEnvVar { profile_id, key } => {
            info!(profile_id, key, "DeleteProfileEnvVar");
            let repo = ProfileEnvRepo::new(&state.db.pool);
            match repo.delete(profile_id, &key).await {
                Ok(()) => Response::Ok,
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
