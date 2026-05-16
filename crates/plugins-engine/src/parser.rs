use anyhow::Context;
use domain_core::entities::PluginKind;
use std::path::Path;

#[derive(Debug)]
pub struct PluginHeader {
    pub kind: PluginKind,
    pub masters: Vec<String>,
}

pub async fn parse_plugin_header(path: &Path) -> anyhow::Result<PluginHeader> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read plugin {}", path.display()))?;

    if bytes.len() < 24 {
        anyhow::bail!("file too small to be a valid plugin");
    }

    if &bytes[0..4] != b"TES4" {
        anyhow::bail!("not a valid Fallout 4 plugin (missing TES4 signature)");
    }

    let flags = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let esl_flag = (flags & 0x200) != 0;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let kind = match ext.as_str() {
        "esm" => PluginKind::Esm,
        "esl" => PluginKind::Esl,
        "esp" if esl_flag => PluginKind::Esl,
        _ => PluginKind::Esp,
    };

    let masters = parse_masters(&bytes);

    Ok(PluginHeader { kind, masters })
}

fn parse_masters(bytes: &[u8]) -> Vec<String> {
    let mut masters = Vec::new();
    let record_data_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let record_end = (24 + record_data_size).min(bytes.len());
    let mut pos = 24usize;

    while pos + 6 <= record_end {
        let subtype = &bytes[pos..pos + 4];
        let subsize = u16::from_le_bytes([bytes[pos + 4], bytes[pos + 5]]) as usize;
        pos += 6;

        if pos + subsize > record_end {
            break;
        }

        if subtype == b"MAST" && subsize > 0 {
            let raw = &bytes[pos..pos + subsize];
            let end = raw.iter().position(|&b| b == 0).unwrap_or(subsize);
            if let Ok(name) = std::str::from_utf8(&raw[..end]) {
                masters.push(name.to_string());
            }
        }

        pos += subsize;
    }

    masters
}
