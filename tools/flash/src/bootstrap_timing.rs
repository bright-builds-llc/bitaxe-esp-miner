use crate::*;
use bitaxe_device_session::BootstrapTiming;

pub(crate) struct TimingExport {
    path: Utf8PathBuf,
    pub(crate) timing: BootstrapTiming,
}

pub(crate) fn prepare(command: &CliCommand) -> Result<Option<TimingExport>> {
    let CliCommand::FlashMonitor(command) = command else {
        return Ok(None);
    };
    if !command.capture_bootstrap_timing {
        return Ok(None);
    }
    if command.factory_reset
        || command.common.dry_run
        || !command.common.redact_evidence
        || command.wifi_credentials.is_some()
        || command.network_reconnect_probe
        || command.thermal_fault_stimulus_intent.is_some()
        || command.self_test_intent.is_some()
    {
        bail!("bootstrap_timing=blocked reason=invocation");
    }
    let root = command
        .common
        .evidence_dir
        .as_ref()
        .context("bootstrap_timing=evidence_dir_required")?;
    let workspace = detect_workspace_dir()?;
    approve_local_private_evidence_root(&workspace, root)?;
    let root = if root.is_absolute() {
        root.clone()
    } else {
        workspace.join(root)
    };
    fs::create_dir_all(&root)?;
    set_private_directory_mode(&root)?;
    let path = root.join("bootstrap-host-timing-v1.json");
    if fs::symlink_metadata(&path).is_ok() {
        bail!("bootstrap_timing=blocked reason=sidecar_exists");
    }
    Ok(Some(TimingExport {
        path,
        timing: BootstrapTiming::default(),
    }))
}
impl TimingExport {
    pub(crate) fn write_preparation_failure(&self) -> Result<()> {
        self.timing.preparation_failed();
        self.write()
    }
    pub(crate) fn write(&self) -> Result<()> {
        write_private_new_bytes(
            &self.path,
            &serde_json::to_vec_pretty(&self.timing.snapshot())?,
        )
        .context("bootstrap_timing=failed reason=evidence_write_failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timing_requires_private_redacted_ordinary_effect() {
        // Arrange
        let base = [
            "flash",
            "flash-monitor",
            "--capture-bootstrap-timing",
            "--evidence-dir",
            "scratch/timing",
            "--redact-evidence",
        ];
        // Act / Assert
        assert!(parse_cli(base).is_ok());
        for extra in ["--dry-run", "--factory-reset"] {
            let mut args = base.to_vec();
            args.push(extra);
            assert!(parse_cli(args).is_err());
        }
        assert!(parse_cli(["flash", "flash-monitor", "--capture-bootstrap-timing"]).is_err());
    }
    #[test]
    fn export_is_exclusive_and_preserves_first_failure() {
        // Arrange
        let dir = tempfile::tempdir().expect("temporary directory");
        let export = TimingExport {
            path: Utf8PathBuf::from_path_buf(dir.path().join("timing.json")).expect("UTF8"),
            timing: BootstrapTiming::default(),
        };
        export.timing.preparation_failed();
        // Act
        export.write().expect("first export");
        let bytes = fs::read(&export.path).expect("read sidecar");
        // Assert
        assert!(export.write().is_err());
        assert_eq!(fs::read(&export.path).expect("unchanged"), bytes);
        assert_eq!(
            fs::metadata(&export.path)
                .expect("mode")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
