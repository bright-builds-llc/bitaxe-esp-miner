use anyhow::{bail, Result};

/// Exact application identity received from the admitted Worker CDC transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbRuntimeIdentity {
    /// Full firmware source commit.
    pub firmware_commit: String,
    /// Application ELF digest reported by the running image.
    pub app_elf_sha256: String,
}

impl UsbRuntimeIdentity {
    /// Validates expected identity before any device observation.
    pub fn new(firmware_commit: &str, app_elf_sha256: &str) -> Result<Self> {
        if !lower_hex(firmware_commit, 40) || !lower_hex(app_elf_sha256, 64) {
            bail!("usb_diagnostics=invalid_expected_identity");
        }
        Ok(Self {
            firmware_commit: firmware_commit.to_owned(),
            app_elf_sha256: app_elf_sha256.to_owned(),
        })
    }
}

/// Closed startup heap checkpoint from Worker evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbMemoryCheckpoint {
    /// Allowlisted startup stage.
    pub stage: String,
    /// Bytes available with internal DMA and 8-bit capabilities.
    pub free_bytes: u32,
    /// Largest contiguous eligible allocation.
    pub largest_block_bytes: u32,
    /// Configured internal-memory reserve.
    pub reserve_bytes: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct WorkerDiagnostics {
    pub maybe_identity: Option<UsbRuntimeIdentity>,
    pub memory: Vec<UsbMemoryCheckpoint>,
    pub worker_start_failed: bool,
}

impl WorkerDiagnostics {
    pub fn parse(text: &str) -> Result<Self> {
        let mut diagnostics = Self::default();
        for line in text.lines() {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            match fields.first().copied() {
                Some("usb_runtime_identity") => diagnostics.identity(&fields)?,
                Some("usb_memory_checkpoint") => diagnostics.memory(&fields)?,
                Some("bwg_worker_start_failure") => diagnostics.worker_start_failed = true,
                _ => {}
            }
        }
        Ok(diagnostics)
    }

    fn identity(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() != 5 || fields[1] != "schema=v1" || fields[4] != "redacted=true" {
            bail!("usb_diagnostics=malformed_runtime_identity");
        }
        let identity = UsbRuntimeIdentity::new(
            field(fields[2], "firmware_commit=")?,
            field(fields[3], "app_elf_sha256=")?,
        )?;
        if self
            .maybe_identity
            .as_ref()
            .is_some_and(|prior| prior != &identity)
        {
            bail!("usb_diagnostics=runtime_identity_changed");
        }
        self.maybe_identity = Some(identity);
        Ok(())
    }

    fn memory(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() != 6 || fields[5] != "redacted=true" {
            bail!("usb_diagnostics=malformed_memory_checkpoint");
        }
        let stage = field(fields[1], "stage=")?;
        if !matches!(
            stage,
            "worker_owner_prepare"
                | "usb_install"
                | "usb_installed"
                | "statistics_start"
                | "statistics_started"
        ) {
            bail!("usb_diagnostics=unknown_memory_stage");
        }
        let checkpoint = UsbMemoryCheckpoint {
            stage: stage.to_owned(),
            free_bytes: number(fields[2], "free_bytes=")?,
            largest_block_bytes: number(fields[3], "largest_block_bytes=")?,
            reserve_bytes: number(fields[4], "reserve_bytes=")?,
        };
        if checkpoint.largest_block_bytes > checkpoint.free_bytes {
            bail!("usb_diagnostics=invalid_memory_range");
        }
        if let Some(prior) = self.memory.iter().find(|prior| prior.stage == stage) {
            if prior != &checkpoint {
                bail!("usb_diagnostics=memory_checkpoint_changed");
            }
            return Ok(());
        }
        self.memory.push(checkpoint);
        Ok(())
    }
}

fn field<'a>(value: &'a str, prefix: &str) -> Result<&'a str> {
    value
        .strip_prefix(prefix)
        .ok_or_else(|| anyhow::anyhow!("usb_diagnostics=malformed_field"))
}

fn number(value: &str, prefix: &str) -> Result<u32> {
    field(value, prefix)?
        .parse()
        .map_err(|_| anyhow::anyhow!("usb_diagnostics=invalid_number"))
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(commit: char) -> String {
        format!(
            "usb_runtime_identity schema=v1 firmware_commit={} app_elf_sha256={} redacted=true",
            commit.to_string().repeat(40),
            "b".repeat(64)
        )
    }

    #[test]
    fn changed_runtime_identity_fails_closed() {
        // Arrange
        let transcript = format!("{}\n{}\n", identity('a'), identity('c'));
        // Act / Assert
        assert!(WorkerDiagnostics::parse(&transcript).is_err());
    }

    #[test]
    fn repeated_identical_checkpoint_is_retained_once() {
        // Arrange
        let line = "usb_memory_checkpoint stage=usb_install free_bytes=50000 largest_block_bytes=12000 reserve_bytes=98304 redacted=true\n";
        // Act
        let diagnostics = WorkerDiagnostics::parse(&line.repeat(2)).expect("valid checkpoint");
        // Assert
        assert_eq!(diagnostics.memory.len(), 1);
        assert_eq!(diagnostics.memory[0].largest_block_bytes, 12000);
    }

    #[test]
    fn unknown_stage_cannot_reach_public_output() {
        // Arrange
        let text = "usb_memory_checkpoint stage=private-value free_bytes=50 largest_block_bytes=12 reserve_bytes=98304 redacted=true\n";
        // Act / Assert
        assert!(WorkerDiagnostics::parse(text).is_err());
    }

    #[test]
    fn malformed_identity_is_not_silently_ignored() {
        // Arrange
        let text = identity('a').replace("schema=v1", "schema=v2");
        // Act / Assert
        assert!(WorkerDiagnostics::parse(&text).is_err());
    }
}
