// Freeze one soak attempt: clean pushed sources, exact package, admitted trust, pinned Gate bundle,
// canonical observer and judge binaries, and a fresh detector admission. No device effect occurs here.
import { lstat, mkdir, readFile, realpath, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall } from "../fixed-usb-qualification/authority.mjs";
import { admitTrust, BUNDLE, canonicalDirectory, cleanPushed, digest, fileDigest, ignored, missing, packageSnapshot, PAGE,
  protectedPath, readJson, requireCondition, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { parseDetector } from "../hardware-operator/detector.mjs";
import { requireSoakTask, SOAK_CONTEXT_SCHEMA, SOAK_RENEWALS, SOAK_SUGGESTED_DIFFICULTY } from "./contract.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const DETECTOR_FRESH_MS = 60000;

async function canonicalBinary(firmwareRoot, name) {
  const path = await realpath(resolve(firmwareRoot, `bazel-bin/tools/http-transport/${name}`));
  const info = await stat(path);
  requireCondition(info.isFile() && (info.mode & 0o111) !== 0, "soak_binary_missing");
  return { path, sha256: await fileDigest(path) };
}

/** Everything a running soak must still match; re-inspected before signing. */
export async function inspectSoakSources(options, operations = {}) {
  const checkRepo = operations.cleanPushed ?? cleanPushed;
  checkRepo(options.firmwareRoot, options.firmwareCommit);
  checkRepo(options.gateRoot, options.gateCommit);
  await requireSoakTask(options.firmwareRoot);
  const packaged = await packageSnapshot(options.firmwareRoot, options.manifest, options.firmwareCommit);
  const trustPath = resolve(options.firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json");
  await protectedPath(options.authorityDirectory, true);
  const call = operations.authorityCall ?? authorityCall;
  const authorityTrust = await call(options.gateRoot, options.authorityDirectory, "public-trust", undefined, options.bun);
  admitTrust(await readJson(trustPath), authorityTrust);
  const bundle = await readFile(resolve(options.gateRoot, BUNDLE));
  requireCondition(bundle.includes(options.gateCommit), "gate_bundle_stale");
  return { ...packaged, firmware_commit: options.firmwareCommit, gate_commit: options.gateCommit,
    gate_bundle_sha256: digest(bundle), gate_page_relative_path: PAGE, gate_page_sha256: await fileDigest(resolve(options.gateRoot, PAGE)),
    trust_sha256: await fileDigest(trustPath), authority_trust_sha256: digest(JSON.stringify(authorityTrust)),
    supervisor_client_sha256: await fileDigest(resolve(SCRIPT_ROOT, "client.mjs")),
    soak_observer: await (operations.canonicalBinary ?? canonicalBinary)(options.firmwareRoot, "soak_observer"),
    soak_judge: await (operations.canonicalBinary ?? canonicalBinary)(options.firmwareRoot, "soak_judge") };
}

async function freshDetector(path, now) {
  await protectedPath(path);
  const info = await lstat(path);
  requireCondition(now - info.mtimeMs >= 0 && now - info.mtimeMs <= DETECTOR_FRESH_MS, "soak_detector_stale");
  return parseDetector(await readFile(path, "utf8"));
}

export async function preflight(options, operations = {}) {
  const root = resolve(options.privateRoot), parent = await canonicalDirectory(dirname(root));
  await protectedPath(parent, true);
  await missing(root);
  ignored(options.firmwareRoot, root);
  const detector = await freshDetector(resolve(options.detector), (operations.now ?? Date.now)());
  const snapshot = await inspectSoakSources(options, operations);
  const context = { schema: SOAK_CONTEXT_SCHEMA, ...snapshot, physical_identity_sha256: detector.physical,
    suggested_difficulty: SOAK_SUGGESTED_DIFFICULTY, renewals: SOAK_RENEWALS, hardware_profile: "upstream-default",
    firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: resolve(options.manifest) };
  await (operations.mkdir ?? mkdir)(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  return { soak_preflight_created: true, device_effects: false, hardware_profile: "upstream-default", renewals: SOAK_RENEWALS };
}

export async function loadSoakContext(root) {
  await protectedPath(resolve(root, "context.json"));
  const record = await readJson(resolve(root, "context.json"));
  requireCondition(record.context?.schema === SOAK_CONTEXT_SCHEMA && record.sha256 === digest(JSON.stringify(record.context)), "soak_context_invalid");
  return record.context;
}

/** Serve and sign only against exactly the sources preflight froze. */
export async function verifyFrozenSoak(context, authorityDirectory, bun, operations = {}) {
  const observed = await inspectSoakSources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root,
    firmwareCommit: context.firmware_commit, gateCommit: context.gate_commit, manifest: context.manifest, authorityDirectory, bun }, operations);
  for (const [key, value] of Object.entries(observed)) {
    requireCondition(JSON.stringify(context[key]) === JSON.stringify(value), "frozen_source_drift");
  }
}
