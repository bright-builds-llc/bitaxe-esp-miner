// Synthetic device/timing fixture. It is not a measurement or accepted hardware evidence.
export function syntheticTiming() {
  const stages = ["reset_command_call_start", "reset_command_call_end", "handoff_start", "handoff_admitted", "monitor_admission_start", "monitor_admitted", "reader_open_start", "reader_opened", "first_nonempty_read", "reader_closed"];
  return { schema: "bootstrap-host-timing-v1", clock: "host_monotonic", origin: "usb_session_acquired", physicalIdentityDigest: "c".repeat(64), sessionNonceSha256: "b".repeat(64),
    events: stages.map((stage, i) => ({ stage, elapsedUs: (i + 1) * 1000 })), resetChildSequence: 1, readerOpenCount: 1, readerReopenCount: 0, firstReadBytes: 92,
    missingStages: [], overflow: false, clockDiscontinuity: false, captureComplete: true, cleanupComplete: true, earliestFailure: null };
}
export function syntheticCapture(context, failure = false) {
  const p = context.package;
  const identity = `usb_runtime_identity schema=v1 firmware_commit=${p.firmware_commit} app_elf_sha256=${p.app_elf_sha256} redacted=true\n`;
  const sample = uptime => `usb_reboot_discriminator schema=v1 boot_ordinal=19 reset_reason=other uptime_ms=${uptime} redacted=true\nusb_startup schema=v1 stage=runtime_ready state=complete first_failure=none uptime_ms=${uptime} redacted=true\n`;
  const observation = "usb_tx_observation schema=v1 slot=first_bootstrap category=bootstrap_diagnostic record_kind=startup_progress outcome=completed stage=completed start_ms=90 end_ms=100 record_bytes=92 queued_bytes=92 queue_calls=1 queue_positive=1 queue_zero=0 last_queue_return=92 drain_calls=1 drain_timeout=0 drain_success=1 drain_other=0 first_drain_ms=91 last_drain_ms=100 max_drain_ms=9 last_drain_return=0 actual_failures=0 replay_attempts=1 replay_completed=0 flags=0 redacted=true\n";
  const failed = observation.replace("outcome=completed stage=completed", "outcome=failed stage=flush_timeout").replace("end_ms=100 ", "end_ms=2090 ")
    .replace("drain_calls=1 drain_timeout=0 drain_success=1", "drain_calls=200 drain_timeout=200 drain_success=0").replace("last_drain_ms=100 max_drain_ms=9 last_drain_return=0", "last_drain_ms=2090 max_drain_ms=10 last_drain_return=263")
    .replace("actual_failures=0", "actual_failures=1");
  const log = identity + sample(2500) + (failure ? "usb_tx_failure schema=v1 stage=flush_timeout elapsed_ms=2000 queued_bytes=92 record_bytes=92 redacted=true\n" + failed +
    failed.replace("slot=first_bootstrap", "slot=first_failure").replace("replay_attempts=1 replay_completed=0", "replay_attempts=2 replay_completed=1") : observation) + sample(4000);
  const verdict = { command_kind: "flash-monitor", flash_status: "completed", board: "205", firmware_commit: p.firmware_commit, observed_firmware_commit: p.firmware_commit,
    nvs_seed_status: "not_provided", redaction_mode: "commit-redacted", capture_timeout_seconds: 30, commit_ready: !failure,
    fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true, startup_complete: true, startup_failed: false, stable_boot: true, retained_failure_history: false, issues: failure ? ["error_diagnostic"] : [] } };
  return { log, verdict };
}
