use std::collections::HashMap;

/// A single file that will be deployed, along with which mod won.
#[derive(Debug, Clone)]
pub struct Winner {
    pub rel_path:     String,   // e.g. "Textures/foo.dds"
    pub source_path:  String,   // absolute path to the winning file on disk
    pub winning_mod:  i64,      // mod_id
    pub priority:     i32,
}

/// Given a list of (mod_id, priority, install_path) tuples for the active
/// profile, compute the winning file for every relative path.
///
/// Higher priority number wins. Ties broken by mod_id (higher wins) for
/// determinism.
pub fn resolve(
    mods: &[(i64, i32, String)],   // (mod_id, priority, install_path)
    file_index: &[(i64, String)],  // (mod_id, rel_path)
) -> Vec<Winner> {
    // Map rel_path -> best (priority, mod_id, install_path)
    let mut best: HashMap<String, (i32, i64, String)> = HashMap::new();

    for (mod_id, priority, install_path) in mods {
        for (fmod_id, rel_path) in file_index {
            if fmod_id != mod_id {
                continue;
            }
            let entry = best.entry(rel_path.clone()).or_insert((*priority, *mod_id, install_path.clone()));
            // Higher priority wins; tie-break by mod_id
            if *priority > entry.0 || (*priority == entry.0 && mod_id > &entry.1) {
                *entry = (*priority, *mod_id, install_path.clone());
            }
        }
    }

    best.into_iter()
        .map(|(rel_path, (priority, mod_id, install_path))| {
            let source_path = format!("{}/{}", install_path, rel_path);
            Winner { rel_path, source_path, winning_mod: mod_id, priority }
        })
        .collect()
}