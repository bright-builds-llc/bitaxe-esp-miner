import { createHash } from "node:crypto";

type EvidenceIdentity = {
  readonly source_commit: string;
  readonly reference_commit: string;
  readonly app_elf_sha256: string;
  readonly package_manifest_sha256: string;
};

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export function createSelfTestEvidence(
  identity: EvidenceIdentity,
  manifestDocument: string,
  planSha256: string,
  passLog: string,
): Record<string, unknown> {
  return {
    schema_version: "bitaxe-self-test-evidence-v1",
    board: 205,
    attempt_ordinal: 3,
    source_commit: identity.source_commit,
    reference_commit: identity.reference_commit,
    app_elf_sha256: identity.app_elf_sha256,
    package_manifest_sha256: identity.package_manifest_sha256,
    plan_sha256: planSha256,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "self-test-campaign",
      request_sha256: sha256(JSON.stringify({
        manifest: sha256(manifestDocument), plan: planSha256, attempt: 3,
      })),
    },
    detector_admitted: true,
    psram_available: passLog.includes("psram_status=available"),
    failure: {
      stable_load_ms: 5_000,
      planned_evaluation_failure: true,
      safe_stop_complete: true,
      failed_state_observed: true,
      cancel_checkpoint_safe: true,
      physical_long_press_observed: true,
      cancellation_receipt_observed: true,
      cancellation_restart_observed: true,
    },
    pass: {
      frequency_mhz: 485,
      core_voltage_mv: 1_200,
      difficulty: 16,
      warmup_celsius: 55,
      target_celsius: 65,
      maximum_celsius: 70,
      measurement_ms: 30_000,
      total_hashrate_passed: true,
      domain_count: 4,
      domain_evaluation_passed: true,
      electrical_checks_passed: true,
      fan_check_passed: true,
      watchdog_advanced: true,
      safe_stop_complete: true,
      pass_receipt_observed: true,
      automatic_restart_observed: true,
    },
    restoration: {
      settings_snapshot_captured_before_write: true,
      local_credentials_used_in_memory: true,
      settings_restored: true,
      mine_on_boot_disabled: true,
      production_mining_never_started: !passLog.includes("production_mining_session=active"),
      pool_traffic_absent: !passLog.includes("pool_transport="),
    },
    cleanup_complete: true,
    private_modes_valid: true,
    redaction_status: "passed",
  };
}
