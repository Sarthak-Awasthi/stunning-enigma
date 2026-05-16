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
