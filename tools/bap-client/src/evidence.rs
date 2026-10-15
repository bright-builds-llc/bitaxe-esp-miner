//! The probe's protected evidence root, per the hardware attempt policy: the caller
//! makes a mode-0700 parent and saves the detector report in it as a mode-0600
//! sibling; the probe alone creates the attempt child, writes one record into it and
//! seals it read-only. An existing child is never reused.

use serde_json::Value;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub const RECORD_FILE: &str = "probe.json";
pub const BINDING_FILE: &str = "binding.json";

fn refuse(category: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, category)
}

fn current_user() -> u32 {
    // SAFETY: getuid has no preconditions and cannot fail.
    unsafe { libc::getuid() }
}

/// True for a real (non-symlink) entry the current user owns with exactly `mode`.
fn private(metadata: &fs::Metadata, mode: u32) -> bool {
    metadata.uid() == current_user() && metadata.permissions().mode() & 0o777 == mode
}

/// The attempt child, created by the probe inside a protected parent.
#[derive(Debug)]
pub struct AttemptRoot {
    child: PathBuf,
}

impl AttemptRoot {
    /// Creates `child` (mode 0700) after proving its parent is a mode-0700 directory
    /// the user owns and that `child` does not yet exist.
    pub fn create(child: &Path) -> io::Result<Self> {
        let parent = child
            .parent()
            .ok_or_else(|| refuse("evidence_parent_missing"))?;
        let parent_metadata = fs::symlink_metadata(parent)?;
        if !parent_metadata.file_type().is_dir() || !private(&parent_metadata, 0o700) {
            return Err(refuse("evidence_parent_not_private"));
        }
        match fs::symlink_metadata(child) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Ok(_) => return Err(refuse("evidence_child_exists")),
            Err(error) => return Err(error),
        }
        DirBuilder::new().mode(0o700).create(child)?;
        Ok(Self {
            child: child.to_owned(),
        })
    }

    /// Writes the one record (mode 0600) as `file`, then seals file and child read-only.
    pub fn seal(self, file: &str, record: &Value) -> io::Result<()> {
        let path = self.child.join(file);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(&serde_json::to_vec_pretty(record)?)?;
        file.sync_all()?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400))?;
        fs::set_permissions(&self.child, fs::Permissions::from_mode(0o500))
    }
}

/// Reads a saved report that must be a mode-0600 file beside the attempt child.
pub fn read_private_sibling(report: &Path, child: &Path) -> io::Result<String> {
    if report.parent() != child.parent() {
        return Err(refuse("report_not_sibling"));
    }
    let metadata = fs::symlink_metadata(report)?;
    if !metadata.file_type().is_file() || !private(&metadata, 0o600) {
        return Err(refuse("report_not_private"));
    }
    fs::read_to_string(report)
}

/// Reads the record of a sealed probe attempt beside the attempt child: the probe
/// child must still be read-only (0500) and its record 0400.
pub fn read_sealed_probe(probe_child: &Path, child: &Path) -> io::Result<String> {
    if probe_child.parent() != child.parent() {
        return Err(refuse("probe_not_sibling"));
    }
    let directory = fs::symlink_metadata(probe_child)?;
    let record_path = probe_child.join(RECORD_FILE);
    let record = fs::symlink_metadata(&record_path)?;
    if !directory.file_type().is_dir()
        || !private(&directory, 0o500)
        || !record.file_type().is_file()
        || !private(&record, 0o400)
    {
        return Err(refuse("probe_not_sealed"));
    }
    fs::read_to_string(record_path)
}

/// Admits the run from the detector's saved report: a mode-0600 sibling of the
/// attempt child, naming exactly one port, with the board running its firmware over
/// native USB Serial/JTAG. Only the profile category is kept.
pub fn admit_detector(report: &Path, child: &Path) -> io::Result<&'static str> {
    let text = read_private_sibling(report, child)?;
    let value = |label: &str| {
        let prefix = format!("{label}: ");
        let matches: Vec<&str> = text
            .lines()
            .filter_map(|line| line.strip_prefix(prefix.as_str()))
            .collect();
        (matches.len() == 1).then(|| matches[0])
    };
    if value("port").is_none() {
        return Err(refuse("detector_port_not_unique"));
    }
    match value("usb_profile") {
        Some("serial_jtag_runtime") => Ok("serial_jtag_runtime"),
        _ => Err(refuse("detector_not_runtime")),
    }
}

#[cfg(test)]
mod tests;
