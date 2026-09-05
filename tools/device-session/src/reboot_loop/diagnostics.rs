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

/// Exact closed maintenance trace received over the admitted Worker CDC channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbMaintenanceTrace {
    sequence: u32,
    marker: String,
}

impl UsbMaintenanceTrace {
    /// Returns the validated closed marker without arbitrary transport text.
    pub fn marker(&self) -> &str {
        &self.marker
    }

    fn parse(fields: &[&str]) -> Result<Self> {
        if fields.len() != 11 || fields[1] != "schema=v1" || fields[10] != "redacted=true" {
            bail!("usb_diagnostics=malformed_maintenance_trace");
        }
        let sequence = number(fields[2], "seq=")?;
        if sequence == 0
            || !matches!(
                field(fields[3], "event=")?,
                "coding_1200"
                    | "coding_115200"
                    | "coding_other"
                    | "dtr0_rts0"
                    | "dtr0_rts1"
                    | "dtr1_rts0"
                    | "dtr1_rts1"
                    | "safe_stop_complete"
                    | "safe_stop_failed"
                    | "detached"
                    | "expiry"
                    | "queue_loss"
                    | "ready_enqueue"
                    | "commit_enqueue"
                    | "phy_invoked"
                    | "phy_returned"
            )
            || !trace_phase(field(fields[4], "before=")?)
            || !trace_phase(field(fields[5], "after=")?)
            || !matches!(
                field(fields[6], "action=")?,
                "none" | "request_safe_stop" | "emit_ready" | "commit_restart"
            )
            || !matches!(field(fields[7], "expired=")?, "true" | "false")
            || number(fields[8], "remaining_ms=")? > 5_000
            || !matches!(
                field(fields[9], "outcome=")?,
                "none"
                    | "ok"
                    | "unavailable_transport"
                    | "disconnected"
                    | "partial_write"
                    | "timeout"
                    | "install"
                    | "handoff"
            )
        {
            bail!("usb_diagnostics=invalid_maintenance_trace");
        }
        let marker = fields.join(" ");
        if marker.len() >= 256 {
            bail!("usb_diagnostics=maintenance_trace_line_bound");
        }
        Ok(Self { sequence, marker })
    }
}

fn trace_phase(phase: &str) -> bool {
    matches!(
        phase,
        "idle" | "dtr_asserted" | "safe_stop_pending" | "ready" | "committed"
    )
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct WorkerDiagnostics {
    pub maybe_identity: Option<UsbRuntimeIdentity>,
    pub memory: Vec<UsbMemoryCheckpoint>,
    pub worker_start_failed: bool,
    pub maintenance_trace: Vec<UsbMaintenanceTrace>,
}

impl WorkerDiagnostics {
    pub fn parse(text: &str) -> Result<Self> {
        let mut diagnostics = Self::default();
        for line in text.lines() {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            match fields.first().copied() {
                Some("usb_runtime_identity") => diagnostics.identity(&fields)?,
                Some("usb_memory_checkpoint") => diagnostics.memory(&fields)?,
                Some("usb_maintenance_trace") => diagnostics.trace(&fields)?,
                Some("bwg_worker_start_failure") => diagnostics.worker_start_failed = true,
                _ => {}
            }
        }
        Ok(diagnostics)
    }

    fn trace(&mut self, fields: &[&str]) -> Result<()> {
        let trace = UsbMaintenanceTrace::parse(fields)?;
        let mut position = match self
            .maintenance_trace
            .binary_search_by_key(&trace.sequence, |prior| prior.sequence)
        {
            Ok(position) => {
                if self.maintenance_trace[position] != trace {
                    bail!("usb_diagnostics=maintenance_trace_changed");
                }
                return Ok(());
            }
            Err(position) => position,
        };
        if self.maintenance_trace.len() == 16 {
            if position == 0 {
                return Ok(());
            }
            self.maintenance_trace.remove(0);
            position -= 1;
        }
        self.maintenance_trace.insert(position, trace);
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
    fn trace(sequence: u32) -> String {
        format!("usb_maintenance_trace schema=v1 seq={sequence} event=commit_enqueue before=committed after=committed action=none expired=false remaining_ms=0 outcome=partial_write redacted=true")
    }

    #[test]
    fn failure_trace_is_available_without_runtime_identity() {
        // Arrange
        let text = trace(1);
        // Act
        let diagnostic = WorkerDiagnostics::parse(&text).expect("closed trace");
        // Assert
        assert!(diagnostic.maybe_identity.is_none());
        assert_eq!(diagnostic.maintenance_trace[0].marker(), text);
    }

    #[test]
    fn trace_rejects_unbounded_or_private_fields() {
        // Arrange
        let text = trace(1);
        // Act / Assert
        for invalid in [
            text.replace("partial_write", "private-value"),
            text.replace("remaining_ms=0", "remaining_ms=5001"),
            format!("{text} payload=secret"),
        ] {
            assert!(WorkerDiagnostics::parse(&invalid).is_err());
        }
    }

    #[test]
    fn trace_capture_retains_only_the_latest_ring_window() {
        // Arrange
        let text = (1..=17).map(trace).collect::<Vec<_>>().join("\n");
        // Act
        let diagnostic = WorkerDiagnostics::parse(&text).expect("bounded rolling trace");
        // Assert
        assert_eq!(diagnostic.maintenance_trace.len(), 16);
        assert_eq!(diagnostic.maintenance_trace[0].sequence, 2);
        assert_eq!(diagnostic.maintenance_trace[15].sequence, 17);
    }
    #[test]
    fn overlapping_trace_bursts_preserve_same_boot_reconnect_classification() {
        // Arrange
        use bitaxe_api::boot_identity::{ResetReasonCategory, WorkerUsbBootMarker};
        let first = (1..=16).map(trace).collect::<Vec<_>>().join("\n");
        let second = (5..=20).map(trace).collect::<Vec<_>>().join("\n");
        let text = format!(
            "{}\n{first}\n{}\n{second}\n{}\n",
            WorkerUsbBootMarker::new(2, ResetReasonCategory::Panic, 100).marker(),
            WorkerUsbBootMarker::new(2, ResetReasonCategory::Panic, 200).marker(),
            trace(1)
        );

        // Act
        let observed = crate::reboot_loop::classify_capture(text.as_bytes(), 2)
            .expect("overlapping same-boot trace");

        // Assert
        assert_eq!(
            observed.category(),
            crate::reboot_loop::UsbRebootLoopCategory::UsbStackReset
        );
        assert_eq!(observed.maintenance_trace().len(), 16);
        assert_eq!(observed.maintenance_trace()[0].sequence, 5);
        assert_eq!(observed.maintenance_trace()[15].sequence, 20);
    }

    #[test]
    fn conflicting_retained_trace_sequence_fails_closed() {
        // Arrange
        let text = format!(
            "{}\n{}\n",
            trace(5),
            trace(5).replace("partial_write", "timeout")
        );
        // Act / Assert
        assert!(WorkerDiagnostics::parse(&text).is_err());
    }
}
