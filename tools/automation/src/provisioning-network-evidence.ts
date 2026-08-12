import { createHash } from "node:crypto";
import { chmod, mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  flashMonitorCommand,
  internalCommandSpec,
  type AutomationCategory,
  type ProvisioningNetworkEvidence,
} from "./contracts.generated.js";
import {
  MacOsProvisioningClient,
  type HostWifiAdmission,
  type ProvisioningClientObservation,
} from "./provisioning-client.js";
import type { ProcessPort } from "./process.js";
import { hasPassiveSafeState } from "./version-evidence.js";
import { assertWithinWorkspace } from "./workspace.js";

export type ProvisioningNetworkEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

export type ProvisioningClientPort = {
  readonly admit: () => Promise<HostWifiAdmission>;
  readonly observe: (admission: HostWifiAdmission) => Promise<ProvisioningClientObservation>;
  readonly cleanup: (admission: HostWifiAdmission) => Promise<boolean>;
};

type FailureCategory = Extract<AutomationCategory,
  | "evidence_invalid"
  | "hardware_blocked"
  | "process_failed"
  | "recovery_failed"
  | "service_recovery_failed"
  | "timeout">;
type RecoveryFacts = {
  readonly host_network_restored: boolean;
  readonly device_recovery_complete: boolean;
  readonly recovery_flash_used: boolean;
  readonly secondary_recovery_failure: boolean;
};
type JsonObject = Readonly<Record<string, unknown>>;

const noRecovery: RecoveryFacts = {
  host_network_restored: false,
  device_recovery_complete: false,
  recovery_flash_used: false,
  secondary_recovery_failure: false,
};

export class ProvisioningNetworkEvidenceError extends Error {
  public constructor(
    public readonly category: FailureCategory,
    message: string,
    public readonly publicValue: Readonly<Record<string, unknown>>,
  ) {
    super(message);
    this.name = "ProvisioningNetworkEvidenceError";
  }

  public withRecovery(recovery: RecoveryFacts): ProvisioningNetworkEvidenceError {
    return new ProvisioningNetworkEvidenceError(this.category, this.message, {
      ...this.publicValue,
      ...recovery,
    });
  }
}

function failure(category: FailureCategory, message: string): ProvisioningNetworkEvidenceError {
  return new ProvisioningNetworkEvidenceError(category, message, {
    stage: "provisioning_network_capture",
    ...noRecovery,
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

function string(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") {
    throw failure("evidence_invalid", `${context} field is invalid`);
  }
  return candidate;
}

async function createPrivateRoot(root: string): Promise<void> {
  try {
    await stat(root);
    throw failure("evidence_invalid", "private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof ProvisioningNetworkEvidenceError) throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(root, { recursive: true, mode: 0o700 });
  await chmod(root, 0o700);
}

async function writePrivate(output: string, contents: string): Promise<void> {
  await writeFile(output, contents, { encoding: "utf8", flag: "wx", mode: 0o600 });
  await chmod(output, 0o600);
}

async function privateModesValid(candidate: string): Promise<boolean> {
  const metadata = await stat(candidate);
  const expected = metadata.isDirectory() ? 0o700 : 0o600;
  if ((metadata.mode & 0o777) !== expected) return false;
  if (!metadata.isDirectory()) return true;
  for (const entry of await readdir(candidate)) {
    if (!await privateModesValid(path.join(candidate, entry))) return false;
  }
  return true;
}

async function runFlashMonitor(
  processPort: ProcessPort,
  flashProgram: string,
  options: ProvisioningNetworkEvidenceOptions,
  manifest: string,
  maybeCredentials: string | undefined,
): Promise<string> {
  let outcome;
  try {
    outcome = await processPort.run(flashMonitorCommand(flashProgram, {
      board: 205,
      port: options.port,
      manifest,
      ...(maybeCredentials === undefined ? {} : { wifiCredentials: maybeCredentials }),
      captureTimeoutSeconds: options.captureTimeoutSeconds,
    }));
  } catch {
    throw failure("process_failed", "provisioning flash-monitor child launch failed");
  }
  if (outcome.timedOut) throw failure("timeout", "provisioning flash-monitor child timed out");
  if (outcome.exitCode !== 0) throw failure("hardware_blocked", "provisioning flash-monitor child was not ready");
  return outcome.stdout;
}

function exactSafeBuild(
  document: string,
  sourceCommit: string,
  referenceCommit: string,
  appElfSha256: string,
): boolean {
  return hasPassiveSafeState(document)
    && [sourceCommit, referenceCommit, appElfSha256].every((identity) => document.includes(identity));
}

async function attemptRecovery(
  processPort: ProcessPort,
  flashProgram: string,
  options: ProvisioningNetworkEvidenceOptions,
  manifestPath: string,
  credentialsPath: string,
  privateRoot: string,
  identities: readonly [string, string, string],
): Promise<boolean> {
  try {
    const document = await runFlashMonitor(
      processPort,
      flashProgram,
      options,
      manifestPath,
      credentialsPath,
    );
    await writePrivate(path.join(privateRoot, "recovery-flash-monitor.private.log"), document);
    return exactSafeBuild(document, ...identities) && document.includes("wifi_status=connected");
  } catch {
    return false;
  }
}

export async function captureProvisioningNetworkEvidence(
  workspaceRoot: string,
  options: ProvisioningNetworkEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
  maybeClient?: ProvisioningClientPort,
): Promise<ProvisioningNetworkEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const manifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const credentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await createPrivateRoot(privateRoot);
  const manifestDocument = await readFile(manifestPath, "utf8");
  await stat(credentialsPath);
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const sourceCommit = string(manifest, "source_commit", "package manifest");
  const referenceCommit = string(manifest, "reference_commit", "package manifest");
  const appElfSha256 = string(manifest, "app_elf_sha256", "package manifest");
  const identities = [sourceCommit, referenceCommit, appElfSha256] as const;
  const client = maybeClient ?? new MacOsProvisioningClient(processPort);
  let admission: HostWifiAdmission;
  try {
    admission = await client.admit();
  } catch {
    throw failure("hardware_blocked", "host Wi-Fi baseline is ineligible");
  }
  let effectStarted = false;
  let hostRestored = false;
  let recoveryAttempted = false;
  let recoveryComplete = false;

  try {
    effectStarted = true;
    const initial = await runFlashMonitor(
      processPort,
      flashProgram,
      options,
      manifestPath,
      undefined,
    );
    await writePrivate(path.join(privateRoot, "ap-flash-monitor.private.log"), initial);
    if (!exactSafeBuild(initial, ...identities)
      || !initial.includes("wifi_status=credentials_missing ap_enabled=true captive_dns=started")) {
      throw failure("evidence_invalid", "safe configuration-network boot evidence is missing");
    }

    let observation: ProvisioningClientObservation;
    try {
      observation = await client.observe(admission);
    } catch {
      throw failure("hardware_blocked", "configuration-network client observation failed");
    }
    await writePrivate(
      path.join(privateRoot, "system-info.private.json"),
      `${JSON.stringify(observation.systemInfo, null, 2)}\n`,
    );
    const systemInfo = observation.systemInfo;
    const apiPostconditionMatches = systemInfo["wifiStatus"] === "credentials_missing"
      && systemInfo["apEnabled"] === 1
      && systemInfo["startMiningOnBoot"] === false;
    const exactBuildIdentityMatches = systemInfo["sourceCommit"] === sourceCommit
      && systemInfo["referenceCommit"] === referenceCommit
      && systemInfo["appElfSha256"] === appElfSha256;
    if (!apiPostconditionMatches || !exactBuildIdentityMatches) {
      throw failure("service_recovery_failed", "configuration-network API postcondition did not match");
    }

    hostRestored = await client.cleanup(admission);
    if (!hostRestored) throw failure("recovery_failed", "host Wi-Fi restoration failed");
    recoveryAttempted = true;
    recoveryComplete = await attemptRecovery(
      processPort,
      flashProgram,
      options,
      manifestPath,
      credentialsPath,
      privateRoot,
      identities,
    );
    if (!recoveryComplete) throw failure("recovery_failed", "exact-package device recovery failed");

    const evidence: ProvisioningNetworkEvidence = {
      schema_version: "bitaxe-provisioning-network-evidence-v1",
      board: 205,
      source_commit: sourceCommit,
      reference_commit: referenceCommit,
      package_manifest_sha256: sha256(manifestDocument),
      workflow: {
        schema_version: "bitaxe-workflow-identity-v1",
        command: "capture-provisioning-network-evidence",
        request_sha256: sha256(JSON.stringify({
          manifest: sha256(manifestDocument),
          timeout: options.captureTimeoutSeconds,
          probe: "provisioning-network-v1",
        })),
      },
      detector_admitted: true,
      boot_observed: true,
      provisioning: {
        host_platform_macos: true,
        single_wifi_interface: true,
        initial_wifi_powered_on: true,
        initial_wifi_unassociated: true,
        baseline_candidate_count: 0,
        configuration_candidate_count: observation.candidateCount,
        association_observed: observation.associationObserved,
        dhcp_observed: observation.dhcpObserved,
        dns_query_count: observation.dnsQueryCount,
        wildcard_dns_answer_matches_gateway: observation.wildcardDnsAnswerMatchesGateway,
        dns_ttl_seconds: observation.dnsTtlSeconds,
        captive_redirect_observed: observation.captiveRedirectObserved,
        captive_redirect_root: observation.captiveRedirectRoot,
        captive_redirect_body_matches: observation.captiveRedirectBodyMatches,
        api_postcondition_matches: true,
        exact_build_identity_matches: true,
      },
      mining_state: "disabled",
      hardware_control_state: "disabled",
      host_network_restored: true,
      device_recovery_complete: true,
      cleanup_complete: true,
      recovery_flash_used: true,
      private_modes_valid: true,
      redaction_status: "passed",
    };
    const candidatePath = path.join(privateRoot, "provisioning-network-evidence.private.json");
    await writePrivate(candidatePath, `${JSON.stringify(evidence, null, 2)}\n`);
    if (!await privateModesValid(privateRoot)) {
      throw failure("evidence_invalid", "private provisioning-network artifact modes are invalid");
    }
    let validation;
    try {
      validation = await processPort.run(internalCommandSpec(validatorProgram, [candidatePath], (value) => value));
    } catch {
      throw failure("process_failed", "provisioning-network validator launch failed");
    }
    if (validation.timedOut) throw failure("timeout", "provisioning-network validation timed out");
    if (validation.exitCode !== 0) throw failure("evidence_invalid", "provisioning-network validation failed");
    await mkdir(path.dirname(projectionPath), { recursive: true });
    await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
    });
    return evidence;
  } catch (error) {
    const primary = error instanceof ProvisioningNetworkEvidenceError
      ? error
      : failure("evidence_invalid", "provisioning-network evidence processing failed");
    if (!effectStarted) throw primary;
    if (!hostRestored) hostRestored = await client.cleanup(admission);
    if (!recoveryAttempted) {
      recoveryAttempted = true;
      recoveryComplete = await attemptRecovery(
        processPort,
        flashProgram,
        options,
        manifestPath,
        credentialsPath,
        privateRoot,
        identities,
      );
    }
    throw primary.withRecovery({
      host_network_restored: hostRestored,
      device_recovery_complete: recoveryComplete,
      recovery_flash_used: recoveryAttempted,
      secondary_recovery_failure: !hostRestored || !recoveryComplete,
    });
  }
}
