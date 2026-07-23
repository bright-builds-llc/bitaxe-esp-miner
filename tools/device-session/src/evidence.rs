use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

use crate::{SessionEvent, SessionState};

const MAX_EVENT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_HTTP_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SERIAL_BYTES: u64 = 16 * 1024 * 1024;

pub struct SessionArtifacts {
    root: Utf8PathBuf,
    projection_output: Utf8PathBuf,
    events: BufWriter<File>,
    http: BufWriter<File>,
    serial: BufWriter<File>,
    event_bytes: u64,
    http_bytes: u64,
    serial_bytes: u64,
}

impl SessionArtifacts {
    pub fn create(root: &Utf8Path, projection_output: &Utf8Path) -> Result<Self> {
        validate_empty_private_root(root)?;
        if fs::symlink_metadata(projection_output.as_std_path()).is_ok() {
            bail!("device-session projection output must not already exist");
        }
        let events = BufWriter::new(open_private_new(&root.join("events.private.jsonl"))?);
        let http = BufWriter::new(open_private_new(&root.join("http.private.jsonl"))?);
        let serial = BufWriter::new(open_private_new(&root.join("serial.private.bin"))?);
        Ok(Self {
            root: root.to_owned(),
            projection_output: projection_output.to_owned(),
            events,
            http,
            serial,
            event_bytes: 0,
            http_bytes: 0,
            serial_bytes: 0,
        })
    }

    pub fn record_serial(&mut self, bytes: &[u8]) -> Result<bool> {
        let count = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if self.serial_bytes.saturating_add(count) > MAX_SERIAL_BYTES {
            return Ok(false);
        }
        self.serial.write_all(bytes)?;
        self.serial.flush()?;
        self.serial_bytes = self.serial_bytes.saturating_add(count);
        Ok(true)
    }

    pub fn record_event(&mut self, event: &SessionEvent) -> Result<bool> {
        let mut encoded =
            serde_json::to_vec(event).context("failed to serialize device-session event")?;
        encoded.push(b'\n');
        let count = u64::try_from(encoded.len()).unwrap_or(u64::MAX);
        if self.event_bytes.saturating_add(count) > MAX_EVENT_BYTES {
            return Ok(false);
        }
        self.events.write_all(&encoded)?;
        self.event_bytes = self.event_bytes.saturating_add(count);
        if matches!(event, SessionEvent::BootBObserved { .. }) {
            if self.http_bytes.saturating_add(count) > MAX_HTTP_BYTES {
                return Ok(false);
            }
            self.http.write_all(&encoded)?;
            self.http_bytes = self.http_bytes.saturating_add(count);
        }
        Ok(true)
    }

    pub fn record_http_value(&mut self, value: &serde_json::Value) -> Result<bool> {
        let mut encoded = serde_json::to_vec(value)?;
        encoded.push(b'\n');
        let count = u64::try_from(encoded.len()).unwrap_or(u64::MAX);
        if self.http_bytes.saturating_add(count) > MAX_HTTP_BYTES {
            return Ok(false);
        }
        self.http.write_all(&encoded)?;
        self.http_bytes = self.http_bytes.saturating_add(count);
        Ok(true)
    }

    pub fn finish(mut self, state: &SessionState) -> Result<()> {
        self.events.flush()?;
        self.http.flush()?;
        self.serial.flush()?;
        write_json_new(
            &self.root.join("result.private.json"),
            &state.private_result(),
        )?;
        write_json_new(&self.projection_output, &state.projection())?;
        Ok(())
    }
}

pub fn validate_private_input(input: &Utf8Path) -> Result<()> {
    let metadata = fs::symlink_metadata(input.as_std_path())
        .with_context(|| format!("failed to inspect private input {input}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("device-session private input must be a regular non-symlink file");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        bail!("device-session private input must be mode 0600");
    }
    Ok(())
}

fn validate_empty_private_root(directory: &Utf8Path) -> Result<()> {
    let metadata = fs::symlink_metadata(directory.as_std_path())
        .with_context(|| format!("failed to inspect private root {directory}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("device-session private root must be a non-symlink directory");
    }
    #[cfg(unix)]
    {
        let mode = metadata.permissions().mode() & 0o777;
        if mode != 0o700 {
            bail!("device-session private root is not mode 0700");
        }
    }
    if fs::read_dir(directory.as_std_path())?.next().is_some() {
        bail!("device-session private root must be empty");
    }
    Ok(())
}

fn open_private_new(path: &Utf8Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options
        .open(path.as_std_path())
        .with_context(|| format!("failed to create private artifact {path}"))?;
    #[cfg(unix)]
    fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

fn write_json_new(path: &Utf8Path, value: &impl serde::Serialize) -> Result<()> {
    let mut file = open_private_new(path)?;
    serde_json::to_writer(&mut file, value)
        .with_context(|| format!("failed to serialize device-session artifact {path}"))?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}
