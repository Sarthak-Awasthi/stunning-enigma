//! Conflict resolution for mod deployments.
//!
//! This module implements priority-based conflict resolution:
//! - Higher priority mods win file conflicts
//! - When priorities are equal, higher mod_id wins (deterministic tiebreaker)

use std::collections::HashMap;

/// A winning file in conflict resolution.
#[derive(Debug, Clone)]
pub struct Winner {
    pub rel_path: String,
    pub source_path: String,
    pub winning_mod: i64,
    pub priority: i32,
}

/// Resolve file conflicts among enabled mods.
///
/// Given a list of (mod_id, priority, install_path) tuples and a file index
/// of (mod_id, relative_path) pairs, determine which mod wins each file
/// based on priority (higher wins) with mod_id as a deterministic tiebreaker.
///
/// # Arguments
///
/// * `mods` - Slice of (mod_id, priority, install_path) tuples
/// * `file_index` - Slice of (mod_id, relative_path) pairs
///
/// # Returns
///
/// A vector of `Winner` entries, one for each unique file path.
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
