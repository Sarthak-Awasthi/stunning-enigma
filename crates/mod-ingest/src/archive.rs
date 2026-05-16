use anyhow::Context;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[derive(Debug, PartialEq)]
pub enum ArchiveKind {
    Zip,
    SevenZip,
    Rar,
}

pub fn detect_kind(path: &Path) -> anyhow::Result<ArchiveKind> {
    use std::io::Read;
    let mut f =
        std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let mut magic = [0u8; 7];
    f.read_exact(&mut magic)
        .context("file too small to detect archive type")?;

    if magic[..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Ok(ArchiveKind::Zip);
    }
    if magic[..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return Ok(ArchiveKind::SevenZip);
    }
    if magic[..4] == [0x52, 0x61, 0x72, 0x21] {
        return Ok(ArchiveKind::Rar);
    }

    anyhow::bail!("unrecognised archive format: {}", path.display())
}

pub fn extract(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dest_dir).context("failed to create extraction directory")?;

    let extracted = match detect_kind(archive_path)? {
        ArchiveKind::Zip => extract_zip(archive_path, dest_dir),
        ArchiveKind::SevenZip => extract_7z(archive_path, dest_dir),
        ArchiveKind::Rar => {
            anyhow::bail!("RAR archives are not supported. Please repack as ZIP or 7z.")
        }
    }?;

    strip_single_top_level_wrapper(dest_dir, extracted)
}

fn extract_zip(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let file = std::fs::File::open(archive_path)?;
    let mut zip = zip::ZipArchive::new(file).context("failed to open ZIP archive")?;

    let mut extracted = Vec::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let rel = PathBuf::from(entry.name());

        if rel.is_absolute() || rel.components().any(|c| c.as_os_str() == "..") {
            continue;
        }

        let out = dest_dir.join(&rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&out)?;
            std::io::copy(&mut entry, &mut outfile)?;
            debug!(path = %rel.display(), "extracted");
            extracted.push(rel);
        }
    }

    info!(count = extracted.len(), "ZIP extraction complete");
    Ok(extracted)
}

fn extract_7z(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    sevenz_rust2::decompress_file(archive_path, dest_dir).context("7z extraction failed")?;

    let extracted = collect_extracted_files(dest_dir)?;

    info!(count = extracted.len(), "7z extraction complete");
    Ok(extracted)
}

fn strip_single_top_level_wrapper(
    dest_dir: &Path,
    extracted: Vec<PathBuf>,
) -> anyhow::Result<Vec<PathBuf>> {
    if extracted.is_empty() {
        return Ok(extracted);
    }

    let mut top_levels = extracted
        .iter()
        .filter_map(|rel| rel.components().next().map(|c| c.as_os_str().to_owned()))
        .collect::<std::collections::HashSet<_>>();

    if top_levels.len() != 1 {
        return Ok(extracted);
    }

    let wrapper_name = match top_levels.drain().next() {
        Some(name) => PathBuf::from(name),
        None => return Ok(extracted),
    };
    let wrapper_dir = dest_dir.join(&wrapper_name);
    if !wrapper_dir.is_dir() {
        return Ok(extracted);
    }

    for entry in std::fs::read_dir(&wrapper_dir)
        .with_context(|| format!("failed to read wrapper dir {}", wrapper_dir.display()))?
    {
        let entry = entry?;
        let from = entry.path();
        let to = dest_dir.join(entry.file_name());

        if to.exists() {
            anyhow::bail!(
                "wrapper normalisation conflict while moving {} to {}",
                from.display(),
                to.display()
            );
        }

        std::fs::rename(&from, &to).with_context(|| {
            format!(
                "failed to move wrapped path {} to {}",
                from.display(),
                to.display()
            )
        })?;
    }

    std::fs::remove_dir(&wrapper_dir)
        .with_context(|| format!("failed to remove wrapper dir {}", wrapper_dir.display()))?;

    let normalized = collect_extracted_files(dest_dir)?;
    info!(
        wrapper = %wrapper_name.display(),
        count = normalized.len(),
        "stripped single top-level wrapper directory"
    );
    Ok(normalized)
}

fn collect_extracted_files(dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut extracted = Vec::new();

    for entry in walkdir::WalkDir::new(dest_dir) {
        let entry = entry.with_context(|| format!("failed to walk {}", dest_dir.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let rel = entry.path().strip_prefix(dest_dir).with_context(|| {
            format!(
                "failed to compute relative extracted path for {}",
                entry.path().display()
            )
        })?;
        extracted.push(rel.to_path_buf());
    }

    Ok(extracted)
}
