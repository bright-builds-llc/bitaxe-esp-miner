use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::{Context, Result};

use super::receive_only::{configure_serial, libc_flags, read_available};

pub(crate) struct WorkerEvidenceReader {
    file: File,
    port: String,
    dtr: libc::c_int,
}

impl WorkerEvidenceReader {
    pub(crate) fn open(port: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc_flags())
            .open(port)
            .context("Worker evidence serial open failed")?;
        configure_serial(&file)?;
        let mut dtr = libc::TIOCM_DTR;
        if unsafe { libc::ioctl(file.as_raw_fd(), libc::TIOCMBIS, &mut dtr) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("Worker evidence DTR assertion failed");
        }
        Ok(Self {
            file,
            port: port.to_owned(),
            dtr,
        })
    }

    pub(crate) fn read_available(&mut self) -> Result<Vec<u8>> {
        read_available(&mut self.file)
    }

    pub(crate) fn port(&self) -> &str {
        &self.port
    }
}

impl Drop for WorkerEvidenceReader {
    fn drop(&mut self) {
        let _result = unsafe { libc::ioctl(self.file.as_raw_fd(), libc::TIOCMBIC, &mut self.dtr) };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn worker_evidence_source_has_no_payload_or_maintenance_baud() {
        // Arrange
        let source = include_str!("worker_evidence.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");

        // Act / Assert
        assert!(source.contains("B115200") || source.contains("configure_serial"));
        assert!(source.contains("TIOCMBIS"));
        assert!(source.contains("TIOCMBIC"));
        for forbidden in ["B1200", "write_all", "std::io::Write", "SetBitRate(1_200)"] {
            assert!(
                !source.contains(forbidden),
                "Worker evidence source contains forbidden operation {forbidden}"
            );
        }
    }
}
