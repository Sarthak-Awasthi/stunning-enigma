use std::path::{Path, PathBuf};
use anyhow::Context;
use tracing::{debug, info};

#[derive(Debug, PartialEq)]
pub enum ArchiveKind {
    Zip,
    SevenZip,
    Rar,   // detected but unsupported — we'll error clearly
}

/// Detect archive type by magic bytes, not file extension.
pub fn detect_kind(path: &Path) -> anyhow::Result<ArchiveKind> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    let mut magic = [0u8; 7];
    f.read_exact(&mut magic)
        .context("file too small to detect archive type")?;

    // ZIP: PK\x03\x04
    if magic[..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Ok(ArchiveKind::Zip);
    }
    // 7z: 7z\xBC\xAF\x27\x1C
    if magic[..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return Ok(ArchiveKind::SevenZip);
    }
    // RAR: Rar!
    if magic[..4] == [0x52, 0x61, 0x72, 0x21] {
        return Ok(ArchiveKind::Rar);
    }

    anyhow::bail!("unrecognised archive format: {}", path.display())
}

/// Extract an archive to `dest_dir`. Returns list of extracted relative paths.
pub fn extract(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dest_dir)
        .context("failed to create extraction directory")?;

    match detect_kind(archive_path)? {
        ArchiveKind::Zip      => extract_zip(archive_path, dest_dir),
        ArchiveKind::SevenZip => extract_7z(archive_path, dest_dir),
        ArchiveKind::Rar      => anyhow::bail!(
            "RAR archives are not supported. Please repack as ZIP or 7z."
        ),
    }
}

fn extract_zip(archive_path: &Path, dest_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let file = std::fs::File::open(archive_path)?;
    let mut zip = zip::ZipArchive::new(file)
        .context("failed to open ZIP archive")?;

    let mut extracted = Vec::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let rel = PathBuf::from(entry.name());

        // Security: skip absolute paths and path traversal
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
    sevenz_rust2::decompress_file(archive_path, dest_dir)
        .context("7z extraction failed")?;

    // Walk dest_dir to collect what was extracted
    let extracted = walkdir::WalkDir::new(dest_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().strip_prefix(dest_dir).unwrap().to_path_buf())
        .collect::<Vec<_>>();

    info!(count = extracted.len(), "7z extraction complete");
    Ok(extracted)
}