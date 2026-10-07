// Freeze one restoration attempt with no device effect: clean pushed sources, the active task's enable line,
// the exact package, admitted trust, the pinned Gate restoration page and bundle, the canonical presence
// watcher binary and a fresh detector admission.
import { lstat, mkdir, readFile, realpath, stat } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { authorityCall } from "../fixed-usb-qualification/authority.mjs";
import { admitTrust, canonicalDirectory, cleanPushed, digest, fileDigest, ignored, missing, packageSnapshot, protectedPath, readJson,
  requireCondition, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { parseDetector } from "../hardware-operator/detector.mjs";
import { ATTEMPT_PATTERN, BUNDLE, CONTEXT_SCHEMA, PAGE, pinnedGateCommit, requireRestorationTask } from "./contract.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
// `just` runs through `bazel run`, and on this host a held binary launch can take minutes; a 60 s bound left the
// soak seal stale twice. Same-device identity is proved again by the watcher and by Gate possession.
const DETECTOR_FRESH_MS = 300000;
const WATCHER_RELATIVE_PATH = "bazel-bin/tools/flash/flash";

async function canonicalWatcher(firmwareRoot) {
  const path = await realpath(resolve(firmwareRoot, WATCHER_RELATIVE_PATH));
  const info = await stat(path);
  requireCondition(info.isFile() && (info.mode & 0o111) !== 0, "watcher_binary_missing");
  return { path, sha256: await fileDigest(path) };
}

/** Everything a running campaign must still match; re-inspected before every signing. */
export async function inspectRestorationSources(options, operations = {}) {
  const checkRepo = operations.cleanPushed ?? cleanPushed;
  checkRepo(options.firmwareRoot, options.firmwareCommit);
  checkRepo(options.gateRoot, options.gateCommit);
  await requireRestorationTask(options.firmwareRoot);
  requireCondition(await pinnedGateCommit(options.firmwareRoot) === options.gateCommit, "gate_commit_not_pinned");
  const packaged = await (operations.packageSnapshot ?? packageSnapshot)(options.firmwareRoot, options.manifest, options.firmwareCommit);
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
    watcher: await (operations.canonicalWatcher ?? canonicalWatcher)(options.firmwareRoot) };
}

async function freshDetector(path, now) {
  await protectedPath(path);
  const info = await lstat(path);
  requireCondition(now - info.mtimeMs >= 0 && now - info.mtimeMs <= DETECTOR_FRESH_MS, "restoration_detector_stale");
  return parseDetector(await readFile(path, "utf8"));
}

/** Effect-free admission; writes only `R/context.json` inside a new mode-0700 attempt root. */
export async function preflight(options, operations = {}) {
  const root = resolve(options.privateRoot), parent = await canonicalDirectory(dirname(root));
  requireCondition(ATTEMPT_PATTERN.test(basename(root)), "attempt_name");
  await protectedPath(parent, true);
  await missing(root);
  ignored(options.firmwareRoot, root);
  await protectedPath(options.poolCredentials);
  ignored(options.firmwareRoot, options.poolCredentials);
  const detector = await freshDetector(resolve(options.detector), (operations.now ?? Date.now)());
  const snapshot = await inspectRestorationSources(options, operations);
  const context = { schema: CONTEXT_SCHEMA, ...snapshot, physical_identity_sha256: detector.physical, attempt: basename(root),
    firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: resolve(options.manifest) };
  await (operations.mkdir ?? mkdir)(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  return { restoration_preflight_created: true, device_effects: false, attempt: basename(root), scenarios: 8 };
}

export async function loadRestorationContext(root) {
  await protectedPath(resolve(root, "context.json"));
  const record = await readJson(resolve(root, "context.json"));
  requireCondition(record.context?.schema === CONTEXT_SCHEMA && record.sha256 === digest(JSON.stringify(record.context)), "restoration_context_invalid");
  return record.context;
}

/** Serve and sign only against exactly the sources preflight froze. */
export async function verifyFrozenRestoration(context, authorityDirectory, bun, operations = {}) {
  const observed = await inspectRestorationSources({ firmwareRoot: context.firmware_root, gateRoot: context.gate_root,
    firmwareCommit: context.firmware_commit, gateCommit: context.gate_commit, manifest: context.manifest, authorityDirectory, bun }, operations);
  for (const [key, value] of Object.entries(observed)) {
    requireCondition(JSON.stringify(context[key]) === JSON.stringify(value), "frozen_source_drift");
  }
}
