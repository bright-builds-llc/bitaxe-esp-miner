//! Web UI variant identity recorded in the package manifest (ADR-0034).
//!
//! The packager passes the selected variant and the exact `www` tree packed
//! into `www.bin`; the manifest records the variant and a SHA-256 digest for
//! every file so hardware evidence can bind the precise web UI.

use std::fs;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// SPIFFS object names hold 64 bytes including the terminating NUL.
const SPIFFS_NAME_LIMIT_BYTES: usize = 63;
const REQUIRED_WEB_UI_FILES: [&str; 2] = ["/index.html", "/version.txt"];

/// Selectable web UI implementation packed into `www.bin`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WebUiVariant {
    /// Handwritten HTML, CSS and JavaScript in `firmware/bitaxe/static/www`.
    Current,
    /// SolidJS port in `firmware/bitaxe/web/solid`.
    Solid,
}

/// One file of the packed `www` tree.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WebUiAsset {
    /// Absolute SPIFFS object path, for example `/assets/app.css`.
    pub(crate) path: String,
    pub(crate) bytes: u64,
    pub(crate) sha256: String,
}

/// Digests every regular file below `www_dir`, sorted by SPIFFS path.
pub(crate) fn collect_web_ui_assets(www_dir: &Utf8Path) -> Result<Vec<WebUiAsset>> {
    let mut files = Vec::new();
    collect_files(www_dir, www_dir, &mut files)?;
    let mut assets = files
        .into_iter()
        .map(|(relative, absolute)| {
            let contents = fs::read(absolute.as_std_path())
                .with_context(|| format!("failed to read web UI asset {absolute}"))?;
            Ok(WebUiAsset {
                path: format!("/{relative}"),
                bytes: u64::try_from(contents.len())?,
                sha256: hex_digest(&contents),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    assets.sort_by(|left, right| left.path.cmp(&right.path));
    validate_web_ui_assets(&assets)?;
    Ok(assets)
}

fn collect_files(
    root: &Utf8Path,
    directory: &Utf8Path,
    files: &mut Vec<(String, Utf8PathBuf)>,
) -> Result<()> {
    let entries = fs::read_dir(directory.as_std_path())
        .with_context(|| format!("failed to list web UI directory {directory}"))?;
    for entry in entries {
        let entry = entry?;
        let Ok(path) = Utf8PathBuf::from_path_buf(entry.path()) else {
            bail!("web UI asset path is not UTF-8");
        };
        if entry.file_type()?.is_dir() {
            collect_files(root, &path, files)?;
            continue;
        }
        let relative = path.strip_prefix(root)?.as_str().replace('\\', "/");
        files.push((relative, path));
    }
    Ok(())
}

fn hex_digest(contents: &[u8]) -> String {
    Sha256::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Checks the manifest's web UI fields: absent together (historical packages) or a complete, sorted inventory.
pub(crate) fn validate_web_ui_manifest(
    maybe_variant: Option<WebUiVariant>,
    assets: &[WebUiAsset],
) -> Result<()> {
    match maybe_variant {
        None if assets.is_empty() => Ok(()),
        None => bail!("package manifest lists web_ui_assets without web_ui_variant"),
        Some(_) => validate_web_ui_assets(assets),
    }
}

fn validate_web_ui_assets(assets: &[WebUiAsset]) -> Result<()> {
    for required in REQUIRED_WEB_UI_FILES {
        if !assets.iter().any(|asset| asset.path == required) {
            bail!("web UI inventory is missing {required}");
        }
    }
    for pair in assets.windows(2) {
        if pair[0].path >= pair[1].path {
            bail!("web UI inventory must be sorted and unique");
        }
    }
    for asset in assets {
        let valid_path = asset.path.starts_with('/')
            && asset.path.len() <= SPIFFS_NAME_LIMIT_BYTES
            && !asset.path.contains("..")
            && !asset.path.contains("//");
        if !valid_path {
            bail!("web UI asset path is invalid: {}", asset.path);
        }
        let valid_digest = asset.sha256.len() == 64
            && asset
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !valid_digest {
            bail!("web UI asset digest is invalid: {}", asset.path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Utf8Path, relative: &str, contents: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture paths have parents"))
            .expect("create fixture dir");
        fs::write(path, contents).expect("write fixture");
    }

    fn fixture_root(dir: &tempfile::TempDir) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("UTF-8 temp dir")
    }

    #[test]
    fn collects_sorted_digests_for_the_packed_tree() {
        // Arrange
        let dir = tempfile::tempdir().expect("temp dir");
        let root = fixture_root(&dir);
        write(&root, "index.html", "<!doctype html>");
        write(&root, "version.txt", "0123456789ab-dev\n");
        write(&root, "assets/app.css", "body{}");

        // Act
        let assets = collect_web_ui_assets(&root).expect("collect assets");

        // Assert
        let paths: Vec<&str> = assets.iter().map(|asset| asset.path.as_str()).collect();
        assert_eq!(paths, ["/assets/app.css", "/index.html", "/version.txt"]);
        assert_eq!(assets[0].bytes, 6);
        assert_eq!(assets[0].sha256, hex_digest(b"body{}"));
    }

    #[test]
    fn rejects_a_tree_without_version_txt() {
        // Arrange
        let dir = tempfile::tempdir().expect("temp dir");
        let root = fixture_root(&dir);
        write(&root, "index.html", "<!doctype html>");

        // Act
        let result = collect_web_ui_assets(&root);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn manifest_web_ui_fields_are_absent_together_or_complete() {
        // Arrange
        let asset = WebUiAsset {
            path: "/index.html".to_owned(),
            bytes: 1,
            sha256: "0".repeat(64),
        };

        // Act
        let historical = validate_web_ui_manifest(None, &[]);
        let orphan_assets = validate_web_ui_manifest(None, std::slice::from_ref(&asset));
        let incomplete = validate_web_ui_manifest(Some(WebUiVariant::Solid), &[asset]);

        // Assert
        assert!(historical.is_ok());
        assert!(orphan_assets.is_err());
        assert!(incomplete.is_err());
    }

    #[test]
    fn variant_names_serialize_as_the_build_flag_values() {
        // Arrange
        let variants = [WebUiVariant::Current, WebUiVariant::Solid];

        // Act
        let names: Vec<String> = variants
            .iter()
            .map(|variant| serde_json::to_string(variant).expect("serialize variant"))
            .collect();

        // Assert
        assert_eq!(names, ["\"current\"", "\"solid\""]);
    }
}
