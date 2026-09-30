//! Compiler input closure shared by Cargo stamps and live freshness checks.
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const ROOTS: &[&str] = &[
    "crates/bitaxe-api",
    "crates/bitaxe-asic",
    "crates/bitaxe-config",
    "crates/bitaxe-core",
    "crates/bitaxe-safety",
    "crates/bitaxe-stratum",
    "crates/bitaxe-runtime",
    "crates/bitaxe-virtual-board",
    "crates/bitaxe-worker-control",
    "crates/bitaxe-simulation",
    "tools/stratum-v2-fixture",
    "tools/virtual-board",
];
pub const FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "BUILD.bazel",
    "MODULE.bazel",
    "MODULE.bazel.lock",
    ".bazelrc",
    ".bazelversion",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    "firmware/bitaxe/sdkconfig.defaults",
];
fn collect(root: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(root.join(relative))? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            return Err(io::Error::other("source symlink"));
        }
        let path = relative.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            collect(root, &path, files)?;
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("rs" | "toml" | "bzl" | "mjs")
        ) || path.file_name().is_some_and(|name| name == "BUILD.bazel")
        {
            files.push(path);
        }
    }
    Ok(())
}
pub fn snapshot(root: &Path) -> io::Result<String> {
    let mut files = FILES.iter().map(PathBuf::from).collect::<Vec<_>>();
    for folder in ROOTS {
        collect(root, Path::new(folder), &mut files)?;
    }
    files.sort();
    files.dedup();
    let mut rows = files
        .into_iter()
        .map(|path| {
            let content = fs::read(root.join(&path))?;
            Ok((
                path.to_string_lossy().replace('\\', "/"),
                Sha256::digest(content)
                    .iter()
                    .flat_map(|byte| {
                        let digits = b"0123456789abcdef";
                        [
                            digits[(byte >> 4) as usize] as char,
                            digits[(byte & 15) as usize] as char,
                        ]
                    })
                    .collect::<String>(),
            ))
        })
        .collect::<io::Result<Vec<_>>>()?;
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    serde_json::to_string(&rows).map_err(io::Error::other)
}
