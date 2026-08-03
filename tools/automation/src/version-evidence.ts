import { createHash } from "node:crypto";
import { access, chmod, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import { flashMonitorCommand, internalCommandSpec, type VersionEvidence } from "./contracts.generated.js";
import { fetchJsonFromSameOrigin, uniqueRuntimeOrigin } from "./http.js";
import type { ProcessPort } from "./process.js";
import { captureJsonWebSocketFrame, type WebSocketFactory } from "./websocket.js";
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
  readonly semantic_version: string;
  readonly source_commit: string;
  readonly reference_commit: string;
  readonly app_elf_sha256: string;
  readonly build_identity: {
    readonly label: string;
    readonly channel: string;
    readonly source_dirty: boolean;
    readonly release_tag: string | null;
  };
  readonly image_metadata: {
    readonly esp_idf_version: string;
  };
};

type JsonObject = Readonly<Record<string, unknown>>;

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function parsePackageManifest(document: string): PackageManifest {
  const value: unknown = JSON.parse(document);
  if (typeof value !== "object" || value === null) throw new Error("package manifest must be an object");
  const candidate = value as Record<string, unknown>;
  const buildIdentity = requiredObject(candidate, "build_identity", "package manifest");
  const imageMetadata = requiredObject(candidate, "image_metadata", "package manifest");
  return {
    semantic_version: requiredString(candidate, "semantic_version", "package manifest"),
    source_commit: requiredString(candidate, "source_commit", "package manifest"),
    reference_commit: requiredString(candidate, "reference_commit", "package manifest"),
    app_elf_sha256: requiredString(candidate, "app_elf_sha256", "package manifest"),
    build_identity: {
      label: requiredString(buildIdentity, "label", "package manifest build identity"),
      channel: requiredString(buildIdentity, "channel", "package manifest build identity"),
      source_dirty: requiredBoolean(buildIdentity, "source_dirty", "package manifest build identity"),
      release_tag: optionalString(buildIdentity, "release_tag", "package manifest build identity"),
    },
    image_metadata: {
      esp_idf_version: requiredString(imageMetadata, "esp_idf_version", "package manifest image metadata"),
    },
  };
}

function requiredObject(value: JsonObject, field: string, context: string): JsonObject {
  const candidate = value[field];
  if (typeof candidate !== "object" || candidate === null || Array.isArray(candidate)) {
    throw new Error(`${context} ${field} must be an object`);
  }
  return candidate as JsonObject;
}

function requiredString(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") {
    throw new Error(`${context} ${field} must be a non-empty string`);
  }
  return candidate;
}

function requiredBoolean(value: JsonObject, field: string, context: string): boolean {
  const candidate = value[field];
  if (typeof candidate !== "boolean") throw new Error(`${context} ${field} must be a boolean`);
  return candidate;
}

function optionalString(value: JsonObject, field: string, context: string): string | null {
  const candidate = value[field];
  if (candidate === null) return null;
  if (typeof candidate !== "string" || candidate === "") {
    throw new Error(`${context} ${field} must be null or a non-empty string`);
  }
  return candidate;
}

function jsonObject(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${context} must be a JSON object`);
  }
  return value as JsonObject;
}

function versionProjection(manifest: PackageManifest, apiValue: unknown, websocketValue: unknown) {
  const api = jsonObject(apiValue, "system info response");
  const envelope = jsonObject(websocketValue, "live WebSocket frame");
  if (envelope["event"] !== "update") throw new Error("live WebSocket event must be update");
  const websocket = requiredObject(envelope, "data", "live WebSocket frame");
  const apiBuildLabelMatchesManifest = requiredString(api, "version", "system info response")
    === manifest.build_identity.label;
  const apiStaticAssetVersionMatchesManifest = requiredString(api, "axeOSVersion", "system info response")
    === manifest.build_identity.label;
  const apiExtendedProvenanceMatchesManifest = (
    requiredString(api, "semanticVersion", "system info response") === manifest.semantic_version
    && requiredString(api, "sourceCommit", "system info response") === manifest.source_commit
    && requiredString(api, "referenceCommit", "system info response") === manifest.reference_commit
    && requiredString(api, "appElfSha256", "system info response") === manifest.app_elf_sha256
    && requiredString(api, "buildChannel", "system info response") === manifest.build_identity.channel
    && requiredBoolean(api, "sourceDirty", "system info response") === manifest.build_identity.source_dirty
    && optionalString(api, "releaseTag", "system info response") === manifest.build_identity.release_tag
  );
  const apiEspIdfVersionMatchesManifest = requiredString(api, "idfVersion", "system info response")
    === manifest.image_metadata.esp_idf_version;
  const websocketSameBootRevisionObserved = (
    requiredString(websocket, "bootSession", "live WebSocket data")
      === requiredString(api, "bootSession", "system info response")
    && websocket["operatorSnapshotRevision"] === api["operatorSnapshotRevision"]
    && typeof api["operatorSnapshotRevision"] === "number"
  );
  const comparedFields = [
    "version",
    "semanticVersion",
    "sourceCommit",
    "referenceCommit",
    "appElfSha256",
    "buildTimestampUtc",
    "buildChannel",
    "sourceDirty",
    "releaseTag",
    "axeOSVersion",
    "idfVersion",
  ];
  const websocketVersionProjectionMatchesApi = comparedFields.every((field) => websocket[field] === api[field]);
  if (
    !apiBuildLabelMatchesManifest
    || !apiStaticAssetVersionMatchesManifest
    || !apiExtendedProvenanceMatchesManifest
    || !apiEspIdfVersionMatchesManifest
    || !websocketSameBootRevisionObserved
    || !websocketVersionProjectionMatchesApi
  ) {
    throw new Error("live version projection does not match the exact package and same-boot API");
  }
  return {
    api_build_label_matches_manifest: true,
    api_static_asset_version_matches_manifest: true,
    api_extended_provenance_matches_manifest: true,
    api_esp_idf_version_matches_manifest: true,
    websocket_same_boot_revision_observed: true,
    websocket_version_projection_matches_api: true,
  } as const;
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
  maybeWebSocketFactory?: WebSocketFactory,
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
  const api = await fetchJsonFromSameOrigin(
    origin,
    "/api/system/info",
    path.join(privateRoot, "system-info.private.json"),
  );
  const websocket = await captureJsonWebSocketFrame(
    origin,
    "/api/ws/live",
    path.join(privateRoot, "live-websocket.private.json"),
    maybeWebSocketFactory,
  );

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
    version_projection: versionProjection(manifest, api, websocket),
  };
  await mkdir(path.dirname(projectionPath), { recursive: true });
  await writeFile(projectionPath, `${JSON.stringify(evidence, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
  const validation = await processPort.run(internalCommandSpec(validatorProgram, [projectionPath], (value) => value));
  if (validation.exitCode !== 0) throw new Error("Rust version evidence validation failed");
  return evidence;
}
