// Generated from bitaxe-automation-contracts. Do not hand-edit.

export type AutomationCommand =
  | "doctor"
  | "bootstrap-esp"
  | "build-firmware"
  | "package-firmware"
  | "package-rollback-probe"
  | "verify-reference"
  | "verify-redaction"
  | "verify-production-session"
  | "observe-serial"
  | "verify-flash-durability"
  | "verify-firmware-ota"
  | "verify-web-assets-ota"
  | "verify-recovery"
  | "verify-http-api"
  | "verify-hardware-surface"
  | "verify-mining"
  | "capture-operator-evidence"
  | "verify-settings-durability"
  | "verify-theme-durability"
  | "capture-correlated-runtime-evidence"
  | "capture-version-evidence"
  | "capture-operator-snapshot-evidence"
  | "capture-runtime-health-evidence"
  | "capture-system-info-evidence"
  | "capture-ultra205-defaults-evidence"
  | "capture-settings-patch-evidence"
  | "capture-log-buffer-evidence"
  | "capture-partition-layout-evidence"
  | "capture-sdkconfig-rollback-evidence";

export type AutomationStatus = "succeeded" | "failed" | "blocked";

export type AutomationCategory =
  | "complete"
  | "invalid_invocation"
  | "contract_mismatch"
  | "workspace_invalid"
  | "dependency_unavailable"
  | "policy_blocked"
  | "authorization_blocked"
  | "process_failed"
  | "timeout"
  | "evidence_invalid"
  | "hardware_blocked"
  | "package_invalid"
  | "interruption_not_observed"
  | "probe_boot_failed"
  | "rollback_not_observed"
  | "recovery_failed";

export type AutomationResult = {
  schema_version: "bitaxe-automation-result-v1";
  command: AutomationCommand;
  status: AutomationStatus;
  category: AutomationCategory;
  public?: unknown;
};

export type WorkflowIdentity = {
  schema_version: "bitaxe-workflow-identity-v1";
  command: AutomationCommand;
  request_sha256: string;
};

export type VersionEvidence = {
  schema_version: "bitaxe-version-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  boot_observed: true;
  same_origin_api_observed: true;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  redaction_status: "passed";
  version_projection?: VersionProjectionEvidence;
};

export type VersionProjectionEvidence = {
  api_build_label_matches_manifest: true;
  api_static_asset_version_matches_manifest: true;
  api_extended_provenance_matches_manifest: true;
  api_esp_idf_version_matches_manifest: true;
  websocket_same_boot_revision_observed: true;
  websocket_version_projection_matches_api: true;
};

export type OperatorSnapshotEpochEvidence = {
  boot_session_sha256: string;
  http_snapshot_observed: true;
  websocket_snapshot_observed: true;
  same_boot_session: true;
  http_revision: number;
  websocket_revision: number;
  websocket_revision_not_earlier: true;
  retained_log_marker_matches_http: true;
  retained_log_marker_matches_websocket: true;
  substantive_fields_present: true;
  stable_fields_match: true;
  safe_operator_state_confirmed: true;
  substantive_projection_sha256: string;
};

export type DeviceSessionEvidence = Readonly<Record<string, unknown>>;

export type ThemeDurabilityEvidence = {
  schema_version: "bitaxe-theme-durability-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  restart_session: DeviceSessionEvidence;
  theme_get_observed: true;
  theme_post_readback: true;
  normal_restart_observed: true;
  post_restart_persistence: true;
  restoration_complete: true;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type OperatorSnapshotEvidence = {
  schema_version: "bitaxe-operator-snapshot-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  baseline_epoch: OperatorSnapshotEpochEvidence;
  post_restart_epoch: OperatorSnapshotEpochEvidence;
  distinct_boot_sessions: true;
  restart_session: DeviceSessionEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type RuntimeHealthEvidence = {
  schema_version: "bitaxe-runtime-health-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  runtime_health: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type SystemInfoEvidence = {
  schema_version: "bitaxe-system-info-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  system_info: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type Ultra205DefaultsObservationEvidence = {
  configured_default_field_count: number;
  firmware_matching_field_count: number;
  firmware_all_defaults_match: boolean;
  api_visible_default_field_count: number;
  http_defaults_match: boolean;
  websocket_defaults_match: boolean;
  retained_attestation_matches: boolean;
  mining_on_boot_disabled: boolean;
  exact_seed_fixture_sha256: string;
  system_info_evidence_sha256: string;
};

export type Ultra205DefaultsEvidence = {
  schema_version: "bitaxe-ultra205-defaults-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  system_info: SystemInfoEvidence;
  defaults: Ultra205DefaultsObservationEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  private_modes_valid: true;
  redaction_status: "passed";
};

export type SettingsPatchEvidence = {
  schema_version: "bitaxe-settings-patch-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  settings_patch: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type LogBufferEvidence = {
  schema_version: "bitaxe-log-buffer-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  log_buffer: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type PartitionLayoutEvidence = {
  schema_version: "bitaxe-partition-layout-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  partition_layout: Readonly<Record<string, unknown>>;
  ota_session: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  private_modes_valid: true;
  redaction_status: "passed";
};

export type SdkconfigRollbackObservationEvidence = {
  sdkconfig_sha256: string;
  rollback_enabled: true;
  anti_rollback_disabled: true;
  rollback_probe_isolated: true;
  interrupted_upload_attempt_count: 1;
  interrupted_upload_prefix_bytes: number;
  interruption_protocol_abort_observed: true;
  baseline_boot_session_unchanged: true;
  baseline_boot_ordinal_unchanged: true;
  baseline_build_unchanged: true;
  probe_pending_validation_observed: true;
  probe_running_partition_ota_0: true;
  rollback_running_partition_factory: true;
  final_normal_build_restored: true;
};

export type SdkconfigRollbackEvidence = {
  schema_version: "bitaxe-sdkconfig-rollback-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  rollback_probe_image_sha256: string;
  rollback_probe_metadata_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  rollback: SdkconfigRollbackObservationEvidence;
  probe_boot_session: DeviceSessionEvidence;
  rollback_session: DeviceSessionEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  normal_package_restored: true;
  recovery_flash_used: false;
  private_modes_valid: true;
  redaction_status: "passed";
};

const automationCommands = new Set<AutomationCommand>([
  "doctor", "bootstrap-esp", "build-firmware", "package-firmware", "package-rollback-probe", "verify-reference",
  "verify-redaction", "verify-production-session", "observe-serial", "verify-flash-durability",
  "verify-firmware-ota", "verify-web-assets-ota", "verify-recovery", "verify-http-api",
  "verify-hardware-surface", "verify-mining", "capture-operator-evidence",
  "verify-settings-durability", "verify-theme-durability", "capture-correlated-runtime-evidence", "capture-version-evidence",
  "capture-operator-snapshot-evidence",
  "capture-runtime-health-evidence",
  "capture-system-info-evidence",
  "capture-ultra205-defaults-evidence",
  "capture-settings-patch-evidence",
  "capture-log-buffer-evidence",
  "capture-partition-layout-evidence",
  "capture-sdkconfig-rollback-evidence",
]);
const automationStatuses = new Set<AutomationStatus>(["succeeded", "failed", "blocked"]);
const automationCategories = new Set<AutomationCategory>([
  "complete", "invalid_invocation", "contract_mismatch", "workspace_invalid",
  "dependency_unavailable", "policy_blocked", "authorization_blocked", "process_failed",
  "timeout", "evidence_invalid", "hardware_blocked", "package_invalid",
  "interruption_not_observed", "probe_boot_failed", "rollback_not_observed", "recovery_failed",
]);

export function parseAutomationResult(value: unknown): AutomationResult {
  if (typeof value !== "object" || value === null) throw new Error("automation result must be an object");
  const candidate = value as Record<string, unknown>;
  if (candidate["schema_version"] !== "bitaxe-automation-result-v1") throw new Error("automation result schema mismatch");
  if (
    typeof candidate["command"] !== "string" || !automationCommands.has(candidate["command"] as AutomationCommand) ||
    typeof candidate["status"] !== "string" || !automationStatuses.has(candidate["status"] as AutomationStatus) ||
    typeof candidate["category"] !== "string" || !automationCategories.has(candidate["category"] as AutomationCategory)
  ) {
    throw new Error("automation result fields are invalid");
  }
  return candidate as AutomationResult;
}

declare const commandSpecBrand: unique symbol;

export type CommandSpec<Result> = {
  readonly program: string;
  readonly args: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
  readonly result: (value: unknown) => Result;
  readonly [commandSpecBrand]: true;
};

type CommonFlashOptions = {
  board?: 205;
  port?: string;
  dryRun?: boolean;
  redactEvidence?: boolean;
  evidenceDir?: string;
};

type PackageSelection =
  | { image?: undefined; manifest?: string }
  | { image: string; manifest: string };

export type FlashOptions = CommonFlashOptions & PackageSelection & {
  wifiCredentials?: string;
};

export type MonitorOptions = CommonFlashOptions & {
  captureTimeoutSeconds?: number;
};

export type FlashMonitorOptions = CommonFlashOptions & PackageSelection & {
  wifiCredentials?: string;
  captureTimeoutSeconds?: number;
} & (
    | { evidenceMode?: undefined }
    | { evidenceMode: "dual"; evidenceDir: string; redactEvidence?: false }
  );

function flag(name: string, value: string | number | boolean | undefined): string[] {
  if (value === undefined || value === false) return [];
  if (value === true) return [`--${name}`];
  return [`--${name}`, String(value)];
}

function commonOptions(options: CommonFlashOptions): string[] {
  return [
    ...flag("board", options.board),
    ...flag("port", options.port),
    ...flag("dry-run", options.dryRun),
    ...flag("redact-evidence", options.redactEvidence),
    ...flag("evidence-dir", options.evidenceDir),
  ];
}

export function internalCommandSpec<Result>(
  program: string,
  args: string[],
  result: (value: unknown) => Result,
  environment?: Readonly<Record<string, string>>,
): CommandSpec<Result> {
  const spec = environment === undefined ? { program, args, result } : { program, args, result, environment };
  return spec as unknown as CommandSpec<Result>;
}

export function flashCommand(program: string, options: FlashOptions): CommandSpec<unknown> {
  return internalCommandSpec(program, ["flash", ...commonOptions(options), ...flag("image", options.image), ...flag("manifest", options.manifest), ...flag("wifi-credentials", options.wifiCredentials)], (value) => value);
}

export function monitorCommand(program: string, options: MonitorOptions): CommandSpec<unknown> {
  return internalCommandSpec(program, ["monitor", ...commonOptions(options), ...flag("capture-timeout-seconds", options.captureTimeoutSeconds)], (value) => value);
}

export function flashMonitorCommand(program: string, options: FlashMonitorOptions): CommandSpec<unknown> {
  return internalCommandSpec(program, ["flash-monitor", ...commonOptions(options), ...flag("image", options.image), ...flag("manifest", options.manifest), ...flag("wifi-credentials", options.wifiCredentials), ...flag("capture-timeout-seconds", options.captureTimeoutSeconds), ...flag("evidence-mode", options.evidenceMode)], (value) => value);
}
