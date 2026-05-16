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
