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
