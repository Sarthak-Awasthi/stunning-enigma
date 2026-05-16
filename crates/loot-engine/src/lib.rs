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
