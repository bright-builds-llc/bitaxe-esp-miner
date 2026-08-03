// Generated from bitaxe-automation-contracts. Do not hand-edit.

export type AutomationCommand =
  | "doctor"
  | "bootstrap-esp"
  | "build-firmware"
  | "package-firmware"
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
  | "capture-correlated-runtime-evidence"
  | "capture-version-evidence";

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
  | "hardware_blocked";

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

const automationCommands = new Set<AutomationCommand>([
  "doctor", "bootstrap-esp", "build-firmware", "package-firmware", "verify-reference",
  "verify-redaction", "verify-production-session", "observe-serial", "verify-flash-durability",
  "verify-firmware-ota", "verify-web-assets-ota", "verify-recovery", "verify-http-api",
  "verify-hardware-surface", "verify-mining", "capture-operator-evidence",
  "verify-settings-durability", "capture-correlated-runtime-evidence", "capture-version-evidence",
]);
const automationStatuses = new Set<AutomationStatus>(["succeeded", "failed", "blocked"]);
const automationCategories = new Set<AutomationCategory>([
  "complete", "invalid_invocation", "contract_mismatch", "workspace_invalid",
  "dependency_unavailable", "policy_blocked", "authorization_blocked", "process_failed",
  "timeout", "evidence_invalid", "hardware_blocked",
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
