use crate::DetectError;
use std::path::Path;

/// Confirm a directory is a real Fallout 4 install by checking for
/// key files that must be present.
pub fn validate_fo4_path(path: &Path) -> Result<(), DetectError> {
    let required = ["Fallout4.exe", "Data"];

    for entry in &required {
        if !path.join(entry).exists() {
            return Err(DetectError::InvalidPath {
                reason: format!("missing '{entry}' in {}", path.display()),
            });
        }
    }

    Ok(())
}
