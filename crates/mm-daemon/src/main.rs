mod profile_fs;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
};
use tracing::{error, info, warn};

use game_detect::{validate_fo4_path, SteamDetector};
use ipc_api::{ErrorCode, ProfileInfo, Request, Response};
use storage_sqlite::{profile_repo::ProfileRepo, Db};

const SOCKET_PATH: &str = "/tmp/mm-daemon.sock";

/// Shared daemon state passed into every connection handler.
#[derive(Clone)]
struct AppState {
    db:       Db,
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

    // Clean up stale socket
    let socket_path = Path::new(SOCKET_PATH);
    if socket_path.exists() {
        std::fs::remove_file(socket_path)
            .context("failed to remove stale socket")?;
    }

    let listener = UnixListener::bind(socket_path)
        .context("failed to bind Unix socket")?;
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

async fn handle_connection(
    stream: tokio::net::UnixStream,
    state: AppState,
) -> anyhow::Result<()> {
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

/// Route a raw JSON line to the right handler and return a Response.
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
            Response::Pong { version: env!("CARGO_PKG_VERSION").to_string() }
        }

        Request::DetectGame => {
            info!("DetectGame");
            match SteamDetector::detect() {
                Ok(path) => match validate_fo4_path(&path) {
                    Ok(()) => Response::GameDetected {
                        install_path: path.display().to_string(),
                        source: "steam".to_string(),
                    },
                    Err(e) => err(e),
                },
                Err(e) => err(e),
            }
        }

        Request::RegisterGame { path } => {
            info!(path = %path, "RegisterGame");
            let p = PathBuf::from(&path);
            match validate_fo4_path(&p) {
                Ok(()) => Response::GameDetected {
                    install_path: path,
                    source: "manual".to_string(),
                },
                Err(e) => err(e),
            }
        }

        Request::CreateProfile { instance_id, name } => {
            info!(instance_id, name = %name, "CreateProfile");
            let repo = ProfileRepo::new(&state.db.pool);
            match repo.create(instance_id, &name).await {
                Ok(id) => {
                    // Materialise the directory on disk
                    match profile_fs::ensure_profile_dir(&state.data_dir, id).await {
                        Ok(_) => Response::ProfileCreated { profile_id: id },
                        Err(e) => err(e),
                    }
                }
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
                            id:          p.id,
                            name:        p.name,
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
    }
}

fn err(e: impl std::fmt::Display) -> Response {
    Response::Error {
        code: ErrorCode::Internal,
        message: e.to_string(),
    }
}

/// Returns the directory containing the running binary.
/// All app data lives here alongside the executable.
fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()
        .context("failed to determine executable path")?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .context("executable has no parent directory")
}