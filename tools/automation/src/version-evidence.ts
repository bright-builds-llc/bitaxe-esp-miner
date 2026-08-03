import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { flashMonitorCommand, internalCommandSpec, type VersionEvidence } from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessPort } from "./process.js";
import { assertWithinWorkspace } from "./workspace.js";

export type VersionEvidenceOptions = {
  readonly privateRoot: string;
  readonly packageManifest: string;
  readonly wifiCredentials: string;
  readonly port: string;
  readonly projection: string;
  readonly captureTimeoutSeconds: number;
};

type PackageManifest = {
  readonly source_commit: string;
  readonly reference_commit: string;
};

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function parsePackageManifest(document: string): PackageManifest {
  const value: unknown = JSON.parse(document);
  if (typeof value !== "object" || value === null) throw new Error("package manifest must be an object");
  const candidate = value as Record<string, unknown>;
  if (typeof candidate["source_commit"] !== "string" || typeof candidate["reference_commit"] !== "string") {
    throw new Error("package manifest identity is missing");
  }
  return {
    source_commit: candidate["source_commit"],
    reference_commit: candidate["reference_commit"],
  };
}

function matchingSession(document: string): boolean {
  const boot = /runtime_boot_identity session=([0-9a-f]{32})\b/u.exec(document)?.[1];
  const origin = /runtime_origin session=([0-9a-f]{32})\b/u.exec(document)?.[1];
  return boot !== undefined && boot === origin;
}

export function hasPassiveSafeState(document: string): boolean {
  return document.split(/\r?\n/u).some((line) => {
    const bootSafeState = line.includes("safe_state:")
      && line.includes("mining=disabled")
      && line.includes("asic_work_submission=disabled")
      && line.includes("hardware_control=disabled");
    const trustedRuntimeAttestation = line.includes("runtime_boot_attestation ")
      && line.includes("mining=disabled")
      && line.includes("work_submission=disabled")
      && line.includes("hardware_control=disabled")
      && line.includes("redacted=true");
    return bootSafeState || trustedRuntimeAttestation;
  });
}

export async function captureVersionEvidence(
  workspaceRoot: string,
  options: VersionEvidenceOptions,
  processPort: ProcessPort,
  flashProgram: string,
  validatorProgram: string,
): Promise<VersionEvidence> {
  const privateRoot = assertWithinWorkspace(workspaceRoot, options.privateRoot);
  const packageManifestPath = assertWithinWorkspace(workspaceRoot, options.packageManifest);
  const wifiCredentialsPath = assertWithinWorkspace(workspaceRoot, options.wifiCredentials);
  const projectionPath = assertWithinWorkspace(workspaceRoot, options.projection);
  await access(packageManifestPath);
  await access(wifiCredentialsPath);
  try {
    await stat(privateRoot);
    throw new Error("private attempt root must be absent before launch");
  } catch (error) {
    if (error instanceof Error && error.message === "private attempt root must be absent before launch") throw error;
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);

  const packageDocument = await readFile(packageManifestPath, "utf8");
  const manifest = parsePackageManifest(packageDocument);
  const manifestDigest = sha256(packageDocument);
  const requestDigest = sha256(JSON.stringify({
    command: "capture-version-evidence",
    package_manifest_sha256: manifestDigest,
    port: options.port,
    capture_timeout_seconds: options.captureTimeoutSeconds,
  }));

  const outcome = await processPort.run(flashMonitorCommand(flashProgram, {
    board: 205,
    port: options.port,
    manifest: packageManifestPath,
    wifiCredentials: wifiCredentialsPath,
    captureTimeoutSeconds: options.captureTimeoutSeconds,
    evidenceMode: "dual",
    evidenceDir: privateRoot,
  }));
  if (outcome.exitCode !== 0) throw new Error("exact-package flash-monitor failed");

  const privateLogPath = path.join(privateRoot, "flash-monitor.classifier-input.log");
  const monitorDocument = await readFile(privateLogPath, "utf8");
  if (!matchingSession(monitorDocument)) throw new Error("boot and runtime-origin session identity do not match");
  if (!hasPassiveSafeState(monitorDocument)) {
    throw new Error("passive boot capture lacks required safe-state markers");
  }
  const origin = uniqueRuntimeOrigin(monitorDocument);
  await fetchJsonFromSameOrigin(origin, "/api/system/info", path.join(privateRoot, "system-info.private.json"));

  const evidence: VersionEvidence = {
    schema_version: "bitaxe-version-evidence-v1",
    board: 205,
    source_commit: manifest.source_commit,
    reference_commit: manifest.reference_commit,
    package_manifest_sha256: manifestDigest,
    workflow: {
      schema_version: "bitaxe-workflow-identity-v1",
      command: "capture-version-evidence",
      request_sha256: requestDigest,
    },
    boot_observed: true,
    same_origin_api_observed: true,
    mining_state: "disabled",
    hardware_control_state: "disabled",
    redaction_status: "passed",
  };
  await mkdir(path.dirname(projectionPath), { recursive: true });
  await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
  const validation = await processPort.run(internalCommandSpec(validatorProgram, [projectionPath], (value) => value));
  if (validation.exitCode !== 0) throw new Error("Rust version evidence validation failed");
  return evidence;
}
