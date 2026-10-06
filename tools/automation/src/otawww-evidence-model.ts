import { createHash } from "node:crypto";

import { factoryImageDigest } from "./flash-child-diagnostics.js";
import { wwwProbeLabel, wwwProbeSchema, type WwwProbeMetadata } from "./package.js";

export const wwwRoute = "/api/system/OTAWWW";
export const wwwSuccessBody = "WWW update complete\n";
export const wwwFinishedLine = "www_update_status=Finished...";
export const wwwProtocolErrorLine = "www_update_status=Protocol Error";
export const unavailableAssetVersion = "Unavailable";
/** The embedded recovery page is the only served page that posts to OTAWWW. */
export const recoveryPageMarker = wwwRoute;

const digestPattern = /^[a-f0-9]{64}$/u;
const commitPattern = /^[a-f0-9]{40}$/u;
const settingsFields = [
  "hostname", "ssid", "stratumURL", "stratumPort", "stratumUser", "stratumProtocol",
  "fallbackStratumURL", "fallbackStratumPort", "fallbackStratumUser", "frequency", "coreVoltage",
  "autofanspeed", "fanspeed", "temptarget", "displayTimeout", "rotation", "invertscreen",
] as const;

type JsonObject = Readonly<Record<string, unknown>>;

export type OtawwwInputs = {
  readonly source: string;
  readonly reference: string;
  readonly app: string;
  readonly buildLabel: string;
  readonly factoryDigest: string;
  readonly probe: WwwProbeMetadata;
};

export class OtawwwInputError extends Error {
  public constructor(message: string) {
    super(message);
    this.name = "OtawwwInputError";
  }
}

export function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function object(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new OtawwwInputError(`${context} must be an object`);
  }
  return value as JsonObject;
}

function text(value: JsonObject, field: string, context: string): string {
  const candidate = value[field];
  if (typeof candidate !== "string" || candidate === "") throw new OtawwwInputError(`${context} ${field} is invalid`);
  return candidate;
}

function packageWwwDigest(manifest: JsonObject): string {
  const artifacts = manifest["artifacts"];
  if (!Array.isArray(artifacts)) throw new OtawwwInputError("package manifest artifacts are invalid");
  const matches = artifacts.filter((candidate) => object(candidate, "artifact")["kind"] === "www_spiffs_image");
  if (matches.length !== 1) throw new OtawwwInputError("package manifest www image is invalid");
  return text(object(matches[0], "www artifact"), "sha256", "www artifact");
}

/** Binds the package, its `www.bin` and the probe image into one exact, clean identity. */
export function parseOtawwwInputs(
  manifestDocument: string,
  probeDocument: string,
  packageWww: Buffer,
  probeWww: Buffer,
): OtawwwInputs {
  const manifest = object(JSON.parse(manifestDocument), "package manifest");
  const source = text(manifest, "source_commit", "package manifest");
  const reference = text(manifest, "reference_commit", "package manifest");
  const app = text(manifest, "app_elf_sha256", "package manifest");
  const identity = object(manifest["build_identity"], "package build identity");
  const buildLabel = text(identity, "label", "package build identity");
  if (!commitPattern.test(source) || !commitPattern.test(reference) || identity["source_dirty"] !== false) {
    throw new OtawwwInputError("package provenance is not clean and exact");
  }
  if (!digestPattern.test(app) || packageWwwDigest(manifest) !== sha256(packageWww)) {
    throw new OtawwwInputError("package www image does not match its manifest");
  }
  const probe = object(JSON.parse(probeDocument), "www probe metadata") as unknown as WwwProbeMetadata;
  if (
    probe.schema_version !== wwwProbeSchema
    || probe.source_commit !== source
    || probe.reference_commit !== reference
    || probe.build_label !== buildLabel
    || probe.probe_label !== wwwProbeLabel(buildLabel)
    || probe.package_www_sha256 !== sha256(packageWww)
    || probe.probe_www_sha256 !== sha256(probeWww)
    || probe.probe_www_bytes !== probeWww.length
    || probeWww.length !== packageWww.length
    || probe.probe_www_sha256 === probe.package_www_sha256
    || probe.package_version_txt_sha256 !== sha256(`${buildLabel}\n`)
    || probe.probe_version_txt_sha256 !== sha256(`${probe.probe_label}\n`)
    || !digestPattern.test(probe.index_html_sha256)
  ) {
    throw new OtawwwInputError("www probe is not bound to the exact package");
  }
  let factoryDigest: string;
  try {
    factoryDigest = factoryImageDigest(manifest as Record<string, unknown>);
  } catch {
    throw new OtawwwInputError("package factory image identity is invalid");
  }
  return { source, reference, app, buildLabel, factoryDigest, probe };
}

/** Private digest of the NVS-backed settings that an OTAWWW must never change. */
export function settingsDigest(info: JsonObject): string {
  return sha256(JSON.stringify(settingsFields.map((field) => [field, info[field] ?? null])));
}

export function retainedLine(logs: string, expected: string): boolean {
  return logs.split(/\r?\n/u).some((line) => line === expected);
}

export function isRecoveryPage(body: string): boolean {
  return body.includes(recoveryPageMarker) && body.toLowerCase().includes("<html");
}
