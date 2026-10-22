//! Hands bytes to and from a file-only tool through a named pipe, so they stay in memory.
//!
//! esptool's `read_flash` writes its output to a path and `nvs_tool` reads its input from one.
//! For data the evidence policy keeps out of storage (`NeverPersistRaw`, such as an NVS image
//! with Wi-Fi and pool secrets), each path is a FIFO in a private directory: the bytes pass
//! through kernel pipe buffers only. A tool that fails before opening its pipe never strands
//! the other end: the waits are bounded.

use crate::*;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

/// How long the other end of a pipe may take once the tool has finished.
const PIPE_SETTLE: Duration = Duration::from_secs(5);

/// A private directory (mode 0700) holding one FIFO (mode 0600); removed when dropped.
pub(crate) struct PrivateFifo {
    _dir: tempfile::TempDir,
    path: Utf8PathBuf,
}

impl PrivateFifo {
    pub(crate) fn create(parent: &Utf8Path, name: &str) -> Result<Self> {
        fs::create_dir_all(parent.as_std_path())?;
        let dir = tempfile::Builder::new()
            .prefix("memory-fifo-")
            .tempdir_in(parent.as_std_path())?;
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700))?;
        let path = Utf8PathBuf::from_path_buf(dir.path().join(name))
            .map_err(|_| anyhow::anyhow!("memory_fifo=blocked reason=path"))?;
        let status = Command::new("mkfifo")
            .args(["-m", "600", path.as_str()])
            .status()?;
        if !status.success() {
            bail!("memory_fifo=blocked reason=mkfifo");
        }
        Ok(Self { _dir: dir, path })
    }

    pub(crate) fn path(&self) -> &Utf8Path {
        &self.path
    }
}

/// Runs `tool`, which writes up to `limit` bytes to `fifo`, and returns what it wrote.
pub(crate) fn read_while(
    fifo: &PrivateFifo,
    limit: usize,
    tool: impl FnOnce() -> Result<()>,
) -> Result<Vec<u8>> {
    let path = fifo.path().to_owned();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let read = (|| -> io::Result<Vec<u8>> {
            let mut bytes = Vec::new();
            io::Read::take(fs::File::open(path.as_std_path())?, limit as u64 + 1)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        })();
        let _ = sender.send(read);
    });
    let outcome = tool();
    let received = match receiver.recv_timeout(Duration::from_millis(1)) {
        Ok(result) => Ok(result),
        // The tool is done; if it never opened the pipe, open the write end to release the reader.
        Err(_) => {
            let release = fifo.path().to_owned();
            thread::spawn(move || drop(OpenOptions::new().write(true).open(release.as_std_path())));
            receiver.recv_timeout(PIPE_SETTLE)
        }
    };
    outcome?;
    let bytes = received
        .map_err(|_| anyhow::anyhow!("memory_fifo=blocked reason=reader_timeout"))?
        .context("memory_fifo=blocked reason=read")?;
    if bytes.len() > limit {
        bail!("memory_fifo=blocked reason=oversize");
    }
    Ok(bytes)
}

/// Runs `tool`, which reads `bytes` from `fifo`, and returns the tool's result.
pub(crate) fn feed_while<T>(
    fifo: &PrivateFifo,
    bytes: Vec<u8>,
    tool: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let path = fifo.path().to_owned();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let wrote = OpenOptions::new()
            .write(true)
            .open(path.as_std_path())
            .and_then(|mut file| file.write_all(&bytes));
        let _ = sender.send(wrote);
    });
    let outcome = tool();
    let received = receiver.recv_timeout(Duration::from_millis(1));
    if received.is_err() {
        // The tool is done without reading; open the read end to release the writer.
        let release = fifo.path().to_owned();
        thread::spawn(move || drop(fs::File::open(release.as_std_path())));
    }
    // A failing tool's own error comes first; the writer's broken pipe would only hide it.
    let value = outcome?;
    match received {
        Ok(wrote) => wrote.context("memory_fifo=blocked reason=write")?,
        Err(_) => {
            if let Ok(wrote) = receiver.recv_timeout(PIPE_SETTLE) {
                wrote.context("memory_fifo=blocked reason=write")?;
            }
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests;
