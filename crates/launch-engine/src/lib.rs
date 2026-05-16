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

pub async fn launch_game(
    profile_id: i64,
    use_f4se: bool,
    env_vars: Vec<(String, String)>,
    db: &Db,
) -> anyhow::Result<LaunchResult> {
    launch_internal(profile_id, use_f4se, false, env_vars, db).await
}

pub async fn launch_launcher(
    profile_id: i64,
    env_vars: Vec<(String, String)>,
    db: &Db,
) -> anyhow::Result<LaunchResult> {
    launch_internal(profile_id, false, true, env_vars, db).await
}

async fn launch_internal(
    profile_id: i64,
    use_f4se: bool,
    use_launcher: bool,
    env_vars: Vec<(String, String)>,
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

    // Apply custom environment variables
    for (key, value) in env_vars {
        command.env(&key, &value);
    }

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
