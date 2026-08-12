import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  internalCommandSpec,
  type AutomationCategory,
  type Ultra205DefaultsEvidence,
} from "./contracts.generated.js";
import type { ProcessPort } from "./process.js";
import {
  captureSystemInfoEvidence,
  SystemInfoEvidenceError,
  type SystemInfoEvidenceOptions,
} from "./system-info-evidence.js";
import type { WebSocketFactory } from "./websocket.js";
import { assertWithinWorkspace } from "./workspace.js";

export type Ultra205DefaultsEvidenceOptions = SystemInfoEvidenceOptions;

type JsonObject = Readonly<Record<string, unknown>>;
type FailureCategory = Extract<AutomationCategory, "hardware_blocked" | "evidence_invalid" | "timeout" | "process_failed">;

const configuredDefaultFieldCount = 27;
const apiVisibleDefaultFieldCount = 23;
const retainedAttestation = "ultra205_config_defaults schema_version=1 matching_fields=27 total_fields=27 all_match=true mineonboot_disabled=true redacted=true";

export class Ultra205DefaultsEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "Ultra205DefaultsEvidenceError";
  }
}

function failure(category: FailureCategory, message: string) {
  return new Ultra205DefaultsEvidenceError(category, message, {
    stage: "ultra205_defaults_capture",
    recovery_complete: true,
    recovery_flash_used: false,
    secondary_recovery_failure: false,
  });
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw failure("evidence_invalid", `${context} must be an object`);
  }
  return value as JsonObject;
}

function parseSeedFixture(document: string): ReadonlyMap<string, string> {
  const values = new Map<string, string>();
  for (const line of document.split(/\r?\n/u)) {
    if (line === "" || line.startsWith("#") || line === "key,type,encoding,value") continue;
    const fields = line.split(",");
    if (fields.length !== 4) throw failure("evidence_invalid", "Ultra 205 seed fixture row is invalid");
    const [key, type, encoding, value] = fields;
    if (key === undefined || type === undefined || encoding === undefined || value === undefined) {
      throw failure("evidence_invalid", "Ultra 205 seed fixture row is incomplete");
    }
    if (type === "namespace") continue;
    if (type !== "data" || !new Set(["string", "u16"]).has(encoding) || values.has(key)) {
      throw failure("evidence_invalid", "Ultra 205 seed fixture identity is invalid");
    }
    values.set(key, value);
  }
  if (values.size !== configuredDefaultFieldCount + 2) {
    throw failure("evidence_invalid", "Ultra 205 seed fixture field count is invalid");
  }
  return values;
}

function fixtureValue(values: ReadonlyMap<string, string>, key: string): string {
  const maybeValue = values.get(key);
  if (maybeValue === undefined) throw failure("evidence_invalid", "Ultra 205 seed fixture is incomplete");
  return maybeValue;
}

function integerFixture(values: ReadonlyMap<string, string>, key: string): number {
  const parsed = Number(fixtureValue(values, key));
  if (!Number.isSafeInteger(parsed)) throw failure("evidence_invalid", "Ultra 205 seed fixture integer is invalid");
  return parsed;
}

function apiDefaultsMatch(snapshot: JsonObject, values: ReadonlyMap<string, string>): boolean {
  const expected: ReadonlyArray<readonly [string, unknown]> = [
    ["hostname", fixtureValue(values, "hostname")],
    ["stratumURL", fixtureValue(values, "stratumurl")],
    ["stratumPort", integerFixture(values, "stratumport")],
    ["stratumTLS", integerFixture(values, "stratumtls")],
    ["stratumCert", fixtureValue(values, "stratumcert")],
    ["stratumUser", fixtureValue(values, "stratumuser")],
    ["stratumSuggestedDifficulty", integerFixture(values, "stratumdiff")],
    ["stratumExtranonceSubscribe", integerFixture(values, "stratumxnsub") !== 0],
    ["fallbackStratumURL", fixtureValue(values, "fbstratumurl")],
    ["fallbackStratumPort", integerFixture(values, "fbstratumport")],
    ["fallbackStratumTLS", integerFixture(values, "fbstratumtls")],
    ["fallbackStratumCert", fixtureValue(values, "fbstratumcert")],
    ["fallbackStratumUser", fixtureValue(values, "fbstratumuser")],
    ["fallbackStratumSuggestedDifficulty", integerFixture(values, "fbstratumdiff")],
    ["fallbackStratumExtranonceSubscribe", integerFixture(values, "fbstratumxnsum") !== 0],
    ["frequency", integerFixture(values, "asicfrequency")],
    ["coreVoltage", integerFixture(values, "asicvoltage")],
    ["ASICModel", fixtureValue(values, "asicmodel")],
    ["boardVersion", fixtureValue(values, "boardversion")],
    ["rotation", integerFixture(values, "rotation")],
    ["autofanspeed", integerFixture(values, "autofanspeed")],
    ["manualFanSpeed", integerFixture(values, "fanspeed")],
    ["overheat_mode", integerFixture(values, "overheat_mode")],
  ];
  return expected.length === apiVisibleDefaultFieldCount
    && expected.every(([field, expectedValue]) => snapshot[field] === expectedValue)
    && snapshot["startMiningOnBoot"] === false
    && snapshot["miningPaused"] === true;
}

async function privateModesValid(root: string): Promise<boolean> {
  if (((await stat(root)).mode & 0o777) !== 0o700) return false;
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const child = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if (!await privateModesValid(child)) return false;
    } else if (entry.isFile() && ((await stat(child)).mode & 0o777) !== 0o600) {
      return false;
    }
  }
  return true;
}

async function writePrivateJson(output: string, value: unknown): Promise<void> {
  await writeFile(output, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8", flag: "wx", mode: 0o600,
  });
  await chmod(output, 0o600);
}

export async function captureUltra205DefaultsEvidence(
  workspaceRoot: string,
  options: Ultra205DefaultsEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  systemInfoValidatorProgram: string,
  defaultsValidatorProgram: string,
  maybeWebSocketFactory?: WebSocketFactory,
): Promise<Ultra205DefaultsEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const projection = assertWithinWorkspace(workspaceRoot, options.projection);
  const fixturePath = path.join(workspaceRoot, "crates/bitaxe-config/fixtures/ultra-205-defaults.csv");
  const fixtureDocument = await readFile(fixturePath, "utf8");
  const fixture = parseSeedFixture(fixtureDocument);
  const systemProjection = path.join(privateRoot, "system-info-projection.private.json");
  let systemInfo;
  try {
    systemInfo = await captureSystemInfoEvidence(
      workspaceRoot,
      { ...options, projection: systemProjection },
      processPort,
      flashProgram,
      systemInfoValidatorProgram,
      maybeWebSocketFactory,
    );
  } catch (error) {
    if (error instanceof SystemInfoEvidenceError) {
      throw new Ultra205DefaultsEvidenceError(error.category, error.message, error.publicValue);
    }
    throw error;
  }
  await chmod(systemProjection, 0o600);

  const api = object(JSON.parse(await readFile(path.join(privateRoot, "api.private.json"), "utf8")), "HTTP defaults snapshot");
  const websocketEnvelope = object(JSON.parse(await readFile(path.join(privateRoot, "websocket.private.json"), "utf8")), "WebSocket defaults envelope");
  const websocket = object(websocketEnvelope["data"], "WebSocket defaults snapshot");
  const retained = await readFile(path.join(privateRoot, "retained-log.private.txt"), "utf8");
  if (!apiDefaultsMatch(api, fixture) || !apiDefaultsMatch(websocket, fixture)) {
    throw failure("evidence_invalid", "API-visible Ultra 205 defaults do not match");
  }
  if (!new Set(retained.split(/\r?\n/u)).has(retainedAttestation)) {
    throw failure("evidence_invalid", "retained Ultra 205 defaults attestation is missing");
  }

  const manifestDocument = await readFile(assertWithinWorkspace(workspaceRoot, options.packageManifest), "utf8");
  const systemInfoDocument = await readFile(systemProjection, "utf8");
  const evidence: Ultra205DefaultsEvidence = {
    schema_version: "bitaxe-ultra205-defaults-evidence-v1",
    board: 205,
    source_commit: systemInfo.source_commit,
    reference_commit: systemInfo.reference_commit,
    package_manifest_sha256: systemInfo.package_manifest_sha256,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "capture-ultra205-defaults-evidence",
      request_sha256: sha256(JSON.stringify({
        manifest: sha256(manifestDocument),
        fixture: sha256(fixtureDocument),
        system_info: sha256(systemInfoDocument),
        timeout: options.captureTimeoutSeconds,
      })),
    },
    detector_admitted: true,
    boot_observed: true,
    system_info: systemInfo,
    defaults: {
      configured_default_field_count: configuredDefaultFieldCount,
      firmware_matching_field_count: configuredDefaultFieldCount,
      firmware_all_defaults_match: true,
      api_visible_default_field_count: apiVisibleDefaultFieldCount,
      http_defaults_match: true,
      websocket_defaults_match: true,
      retained_attestation_matches: true,
      mining_on_boot_disabled: true,
      exact_seed_fixture_sha256: sha256(fixtureDocument),
      system_info_evidence_sha256: sha256(systemInfoDocument),
    },
    mining_state: "disabled",
    hardware_control_state: "disabled",
    cleanup_complete: true,
    private_modes_valid: true,
    redaction_status: "passed",
  };
  const candidate = path.join(privateRoot, "ultra205-defaults-evidence.private.json");
  await writePrivateJson(candidate, evidence);
  if (!await privateModesValid(privateRoot)) {
    throw failure("evidence_invalid", "private Ultra 205 defaults artifacts have invalid modes");
  }
  let validation;
  try {
    validation = await processPort.run(internalCommandSpec(defaultsValidatorProgram, [candidate], (value) => value));
  } catch {
    throw failure("process_failed", "Ultra 205 defaults validator launch failed");
  }
  if (validation.timedOut) throw failure("timeout", "Ultra 205 defaults validation timed out");
  if (validation.exitCode !== 0) throw failure("evidence_invalid", "Ultra 205 defaults validation failed");
  await mkdir(path.dirname(projection), { recursive: true });
  await writeFile(projection, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
  return evidence;
}
