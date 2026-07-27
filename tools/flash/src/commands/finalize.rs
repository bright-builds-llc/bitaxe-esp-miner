use crate::*;

pub(crate) fn run_finalize_evidence(
    command: &FinalizeEvidenceCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let evidence_dir = environment.workspace_path(&command.evidence_dir);
    environment
        .approve_private_evidence_root(&evidence_dir)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=root_admission_failed"))?;
    let paths = evidence::preflight_dual_finalization_paths(&evidence_dir)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=finalize_preflight_failed"))?;
    let private_sha256 = evidence::private_log_sha256(&paths.private_log)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=private_digest_failed"))?;
    if private_sha256 != command.expected_private_sha256 {
        bail!("dual_evidence=failed reason=classified_digest_mismatch");
    }

    let private_json = environment
        .read_to_string(&paths.private_record)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=private_record_unreadable"))?;
    let mut record: EvidenceRecord = serde_json::from_str(&private_json)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=private_record_invalid"))?;
    if record.redaction_mode != "dual"
        || record.commit_ready
        || record.private_monitor_log_path.as_deref() != Some(paths.private_log.as_str())
        || record.private_monitor_log_sha256.as_deref()
            != Some(command.expected_private_sha256.as_str())
        || record.monitor_log_sha256.is_some()
    {
        bail!("dual_evidence=failed reason=private_record_mismatch");
    }
    let capture_state = validate_evidence_record_capture_state(&record)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=private_record_invalid_state"))?;
    let deferred_capture = capture_state == MonitorCaptureState::PendingPrivateClassification;
    if !matches!(capture_state, MonitorCaptureState::Trusted { .. }) && !deferred_capture {
        bail!("dual_evidence=failed reason=private_capture_not_classifiable");
    }

    let finalize_result = (|| -> Result<()> {
        let digests =
            evidence::derive_admitted_log(&paths, command.expected_private_sha256.as_str())?;
        record.command = PROTECTED_OPERATIONAL.to_owned();
        record.flash_command = PROTECTED_OPERATIONAL.to_owned();
        record.monitor_command = PROTECTED_OPERATIONAL.to_owned();
        record.port = "[redacted]".to_owned();
        record.manifest_path = PROTECTED_OPERATIONAL.to_owned();
        record.flash_image_path = PROTECTED_OPERATIONAL.to_owned();
        record.log_path = "flash-monitor.log".to_owned();
        record.monitor_log_path = "flash-monitor.log".to_owned();
        record.private_log_role = None;
        record.private_monitor_log_path = None;
        record.private_monitor_log_sha256 = None;
        record.monitor_log_sha256 = Some(digests.admitted_sha256);
        record.commit_ready = true;
        if deferred_capture {
            apply_monitor_capture_state(
                &mut record,
                &MonitorCaptureState::AdmittedPrivateClassification,
            );
        }
        let admitted_json = serde_json::to_string_pretty(&record)
            .context("failed to serialize admitted evidence")?;
        evidence::write_dual_admitted_text(&paths.admitted_record, &admitted_json)
    })();
    if let Err(error) = finalize_result {
        for path in [&paths.admitted_log, &paths.admitted_record] {
            if let Err(remove_error) = fs::remove_file(path.as_std_path()) {
                if remove_error.kind() != std::io::ErrorKind::NotFound {
                    return Err(remove_error).context("failed to roll back admitted evidence");
                }
            }
        }
        return Err(error).context("dual_evidence=failed reason=finalization_failed");
    }
    emit_line("dual_evidence", "finalized")
}
