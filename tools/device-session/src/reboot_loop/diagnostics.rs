use anyhow::{bail, Result};

/// Exact application identity received from the admitted Serial/JTAG transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbRuntimeIdentity {
    /// Full firmware source commit.
    pub firmware_commit: String,
    /// Application ELF digest reported by the running image.
    pub app_elf_sha256: String,
}

impl UsbRuntimeIdentity {
    /// Parses one complete closed application-identity line without admitting a transport.
    pub fn parse(line: &str) -> Result<Self> {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.first() != Some(&"usb_runtime_identity") {
            bail!("usb_diagnostics=malformed_runtime_identity");
        }
        let mut diagnostics = WorkerDiagnostics::default();
        diagnostics.identity(&fields)?;
        diagnostics
            .maybe_identity
            .ok_or_else(|| anyhow::anyhow!("usb_diagnostics=missing_runtime_identity"))
    }

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

/// Closed, nonsecret progress retained even when application startup cannot complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbStartupProgress {
    /// Current allowlisted startup phase.
    pub stage: &'static str,
    /// Entered, failed, or complete phase state.
    pub state: &'static str,
    /// Earliest failed phase in the current boot, if present.
    pub maybe_first_failure: Option<&'static str>,
    /// Device monotonic time when this diagnostic was emitted.
    pub uptime_ms: u64,
}
impl UsbStartupProgress {
    /// Parses one complete closed startup line; capture framing belongs to the caller.
    pub fn parse(line: &str) -> Result<Self> {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.first() != Some(&"usb_startup") {
            bail!("usb_diagnostics=malformed_startup_progress");
        }
        let mut diagnostics = WorkerDiagnostics::default();
        diagnostics.startup(&fields)?;
        diagnostics
            .maybe_startup
            .ok_or_else(|| anyhow::anyhow!("usb_diagnostics=missing_startup_progress"))
    }

    /// Renders only closed validated fields for public diagnostic output.
    pub fn marker(&self) -> String {
        format!(
            "usb_startup schema=v1 stage={} state={} first_failure={} uptime_ms={} redacted=true",
            self.stage,
            self.state,
            self.maybe_first_failure.unwrap_or("none"),
            self.uptime_ms
        )
    }
}

impl UsbMemoryCheckpoint {
    /// Parses one complete closed heap checkpoint without accepting arbitrary diagnostic text.
    pub fn parse(line: &str) -> Result<Self> {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.first() != Some(&"usb_memory_checkpoint") {
            bail!("usb_diagnostics=malformed_memory_checkpoint");
        }
        let mut diagnostics = WorkerDiagnostics::default();
        diagnostics.memory(&fields)?;
        diagnostics
            .memory
            .pop()
            .ok_or_else(|| anyhow::anyhow!("usb_diagnostics=missing_memory_checkpoint"))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct WorkerDiagnostics {
    pub maybe_identity: Option<UsbRuntimeIdentity>,
    pub memory: Vec<UsbMemoryCheckpoint>,
    pub worker_start_failed: bool,
    pub maybe_startup: Option<UsbStartupProgress>,
}

impl WorkerDiagnostics {
    pub fn parse(text: &str) -> Result<Self> {
        let mut diagnostics = Self::default();
        for line in text.split_inclusive('\n') {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            match fields.first().copied() {
                Some("usb_runtime_identity") => diagnostics.identity(&fields)?,
                Some("usb_memory_checkpoint") => diagnostics.memory(&fields)?,
                // A bounded read can stop within a wire record; only LF closes this record.
                Some("usb_startup") if line.ends_with('\n') => diagnostics.startup(&fields)?,
                Some("bwg_worker_start_failure") => diagnostics.worker_start_failed = true,
                _ => {}
            }
        }
        Ok(diagnostics)
    }

    fn startup(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() != 7 || fields[1] != "schema=v1" || fields[6] != "redacted=true" {
            bail!("usb_diagnostics=malformed_startup_progress");
        }
        let stage = startup_stage(field(fields[2], "stage=")?)?;
        let state = match field(fields[3], "state=")? {
            "entered" => "entered",
            "failed" => "failed",
            "complete" => "complete",
            _ => bail!("usb_diagnostics=unknown_startup_state"),
        };
        let maybe_first_failure = match field(fields[4], "first_failure=")? {
            "none" => None,
            other => Some(startup_stage(other)?),
        };
        if state == "failed" && maybe_first_failure.is_none() {
            bail!("usb_diagnostics=missing_startup_failure");
        }
        let uptime = field(fields[5], "uptime_ms=")?;
        if uptime.is_empty() || !uptime.bytes().all(|byte| byte.is_ascii_digit()) {
            bail!("usb_diagnostics=invalid_startup_uptime");
        }
        let uptime_ms = uptime
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("usb_diagnostics=invalid_startup_uptime"))?;
        if self.maybe_startup.as_ref().is_some_and(|prior| {
            prior.uptime_ms > uptime_ms
                || (prior.maybe_first_failure.is_some()
                    && prior.maybe_first_failure != maybe_first_failure)
        }) {
            bail!("usb_diagnostics=inconsistent_startup_progress");
        }
        self.maybe_startup = Some(UsbStartupProgress {
            stage,
            state,
            maybe_first_failure,
            uptime_ms,
        });
        Ok(())
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
                | "wifi_driver_prepare"
                | "wifi_driver_prepared"
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

fn startup_stage(value: &str) -> Result<&'static str> {
    match value {
        "early_identity" => Ok("early_identity"),
        "usb_install" => Ok("usb_install"),
        "nvs" => Ok("nvs"),
        "hardware" => Ok("hardware"),
        "worker_recovery" => Ok("worker_recovery"),
        "runtime_services" => Ok("runtime_services"),
        "storage_http" => Ok("storage_http"),
        "network" => Ok("network"),
        "worker_control" => Ok("worker_control"),
        "statistics" => Ok("statistics"),
        "runtime_ready" => Ok("runtime_ready"),
        _ => bail!("usb_diagnostics=unknown_startup_stage"),
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

    #[test]
    fn startup_failure_survives_later_completed_stages() {
        // Arrange
        let first = "usb_startup schema=v1 stage=nvs state=failed first_failure=nvs uptime_ms=10 redacted=true";
        let final_line = "usb_startup schema=v1 stage=runtime_ready state=complete first_failure=nvs uptime_ms=20 redacted=true";
        // Act
        let diagnostics = WorkerDiagnostics::parse(&format!("{first}\n{final_line}\n"))
            .expect("closed startup records");
        // Assert
        assert_eq!(
            diagnostics.maybe_startup.expect("latest startup").marker(),
            final_line
        );
    }

    #[test]
    fn arbitrary_startup_fields_cannot_reach_public_output() {
        // Arrange
        let valid = "usb_startup schema=v1 stage=network state=entered first_failure=none uptime_ms=10 redacted=true";
        // Act / Assert
        for invalid in [
            valid.replace("stage=network", "stage=private-value"),
            valid.replace("state=entered", "state=private-value"),
            valid.replace("first_failure=none", "first_failure=private-value"),
            valid.replace("uptime_ms=10", "uptime_ms=private-value"),
            valid.replace("redacted=true", "redacted=false"),
            format!("{valid} extra=private-value"),
        ] {
            assert!(WorkerDiagnostics::parse(&format!("{invalid}\n")).is_err());
        }
    }

    #[test]
    fn startup_progress_cannot_erase_the_first_failure_in_one_boot() {
        // Arrange
        let first = "usb_startup schema=v1 stage=nvs state=failed first_failure=nvs uptime_ms=10 redacted=true";
        let later = "usb_startup schema=v1 stage=network state=entered first_failure=none uptime_ms=20 redacted=true";
        // Act / Assert
        assert!(WorkerDiagnostics::parse(&format!("{first}\n{later}\n")).is_err());
    }
    #[test]
    fn bounded_capture_ending_inside_next_startup_record_keeps_last_complete_progress() {
        // Arrange
        let complete = "usb_startup schema=v1 stage=network state=entered first_failure=none uptime_ms=10 redacted=true";
        let capture = format!("{complete}\nusb_startup schema=v1 stage=net");
        // Act
        let diagnostics = WorkerDiagnostics::parse(&capture).expect("incomplete capture tail");
        // Assert
        assert_eq!(
            diagnostics
                .maybe_startup
                .expect("complete startup")
                .marker(),
            complete
        );
    }

    #[test]
    fn malformed_newline_terminated_startup_record_remains_an_error() {
        // Arrange
        let capture = "usb_startup schema=v1 stage=net\n";
        // Act / Assert
        assert!(WorkerDiagnostics::parse(capture).is_err());
    }
    #[test]
    fn wifi_constructor_checkpoints_are_retained_as_distinct_stages() {
        // Arrange
        let text = "usb_memory_checkpoint stage=wifi_driver_prepare free_bytes=100000 largest_block_bytes=64000 reserve_bytes=98304 redacted=true\nusb_memory_checkpoint stage=wifi_driver_prepared free_bytes=40000 largest_block_bytes=32000 reserve_bytes=98304 redacted=true\n";
        // Act
        let diagnostics = WorkerDiagnostics::parse(text).expect("closed Wi-Fi checkpoint pair");
        // Assert
        assert_eq!(diagnostics.memory.len(), 2);
        assert_eq!(diagnostics.memory[0].stage, "wifi_driver_prepare");
        assert_eq!(diagnostics.memory[1].stage, "wifi_driver_prepared");
    }
}
