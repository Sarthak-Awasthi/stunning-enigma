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
