import { mkdir, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual } from "node:util";
import {
  admitTrust,
  BUNDLE,
  PAGE,
  canonicalBase64,
  canonicalDirectory,
  cleanPushed,
  digest,
  exactObject,
  fileDigest,
  ignored,
  missing,
  nonce,
  packageSnapshot,
  protectedPath,
  readJson,
  requireCondition,
  writeNew,
} from "./contract.mjs";
import { cadenceValidatorDigest, requireCadenceTask, validateCadenceProgress } from "./cadence-contract.mjs";
import { inspectOriginalCampaign, readFrozenCampaign } from "./no-mining-accounting.mjs";
import { NO_MINING_SCHEMA, validateNoMiningContext } from "./no-mining-context.mjs";
import { readStartupFailure } from "./cadence-startup-failure.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";

export const STARTUP_RECOVERY_SCHEMA = "fixed-usb-cadence-startup-recovery-context-v1";
const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const SOURCE_KEYS = [
  "manifest_sha256",
  "app_elf_sha256",
  "reference_commit",
  "artifacts",
  "update_segments",
  "firmware_commit",
  "gate_commit",
  "gate_bundle_sha256",
  "gate_page_relative_path",
  "gate_page_sha256",
  "trust_sha256",
  "supervisor_client_sha256",
];
export function forbidStartupCredentials(options) {
  requireCondition(
    options.authorityDirectory === undefined && options.poolCredentials === undefined,
    "startup_recovery_credentials_forbidden",
  );
}
export async function inspectStartupSources(options, operations = {}) {
  await requireCadenceTask(options.firmwareRoot);
  const check = operations.cleanPushed ?? cleanPushed;
  check(options.firmwareRoot, options.firmwareCommit);
  check(options.gateRoot, options.gateCommit);
  const packaged = await packageSnapshot(options.firmwareRoot, options.manifest, options.firmwareCommit);
  const trustPath = resolve(options.firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"),
    trust = await readJson(trustPath);
  admitTrust(trust, trust);
  const bundle = await readFile(resolve(options.gateRoot, BUNDLE));
  requireCondition(bundle.includes(options.gateCommit), "startup_gate_bundle_stale");
  return {
    ...packaged,
    firmware_commit: options.firmwareCommit,
    gate_commit: options.gateCommit,
    gate_bundle_sha256: digest(bundle),
    gate_page_relative_path: PAGE,
    gate_page_sha256: await fileDigest(resolve(options.gateRoot, PAGE)),
    trust_sha256: await fileDigest(trustPath),
    supervisor_client_sha256: await fileDigest(resolve(SCRIPT_ROOT, "no-mining-client.mjs")),
  };
}
function innerContext(root, context) {
  return {
    schema: NO_MINING_SCHEMA,
    ...Object.fromEntries(SOURCE_KEYS.map((key) => [key, context[key]])),
    required_no_mining_cycles: 4,
    mining_authorized: false,
    original_campaign_record: context.original_campaign_record,
    firmware_root: context.firmware_root,
    gate_root: resolve(root, "qualified-artifacts/gate"),
    manifest: resolve(root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"),
  };
}
async function snapshotSources(root, context) {
  const manifest = await readJson(context.manifest),
    entries = [];
  async function retain(name, source) {
    const path = resolve(root, "qualified-artifacts", name);
    await mkdir(dirname(path), { recursive: true, mode: 0o700 });
    const bytes = await readFile(source);
    await writeNewBytes(path, bytes);
    entries.push({ path: name, sha256: digest(bytes), length: bytes.length });
  }
  await retain("firmware/bitaxe-ultra205-package.json", context.manifest);
  for (const item of manifest.artifacts)
    await retain(
      `firmware/${item.path}`,
      resolve(item.kind === "partition_table" ? context.firmware_root : dirname(context.manifest), item.path),
    );
  for (const name of ["license-inventory", "provenance-manifest"])
    await retain(`firmware/docs/release/${name}.md`, resolve(context.firmware_root, `docs/release/${name}.md`));
  await retain(`gate/${BUNDLE}`, resolve(context.gate_root, BUNDLE));
  await retain(`gate/${PAGE}`, resolve(context.gate_root, PAGE));
  await writeNew(resolve(root, "artifact-snapshot.json"), {
    schema: "fixed-usb-qualified-artifacts-v1",
    context_sha256: digest(JSON.stringify(context)),
    files: entries,
  });
  await verifyArtifactSnapshot(root, context);
}
async function writeNewBytes(path, bytes) {
  const { open } = await import("node:fs/promises");
  const file = await open(path, "wx", 0o600);
  try {
    await file.writeFile(bytes);
    await file.sync();
  } finally {
    await file.close();
  }
}
export async function startupRecoveryPreflight(options, operations = {}) {
  forbidStartupCredentials(options);
  const root = resolve(options.privateRoot);
  await protectedPath(dirname(root), true);
  await missing(root);
  for (const key of ["firmwareRoot", "gateRoot", "predecessorRoot"]) options[key] = await canonicalDirectory(options[key]);
  await requireCadenceTask(options.firmwareRoot);
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const failure = await readStartupFailure(options.predecessorRoot, operations);
  requireCondition(
    root !== failure.root &&
      dirname(root) === dirname(failure.root) &&
      (options.previousReceipt === undefined || resolve(options.previousReceipt) === failure.context.previous_receipt),
    "startup_recovery_lineage",
  );
  await protectedPath(options.input);
  validateCadenceProgress(await readJson(options.input));
  const source = await (operations.inspectStartupSources ?? inspectStartupSources)(options, operations);
  requireCondition(
    source.firmware_commit !== failure.context.firmware_commit || source.gate_commit !== failure.context.gate_commit,
    "startup_recovery_unchanged_pair",
  );
  const original = await inspectOriginalCampaign(options.originalCampaignRecord);
  requireCondition(
    (await readFrozenCampaign({ original_campaign_record: original })) === failure.previous.original_campaign_id,
    "startup_original_campaign_mismatch",
  );
  const context = {
    schema: STARTUP_RECOVERY_SCHEMA,
    recovery_id: nonce(),
    ...source,
    firmware_root: options.firmwareRoot,
    gate_root: options.gateRoot,
    manifest: resolve(options.manifest),
    failed_root: failure.root,
    failed_inventory_sha256: failure.failed_inventory_sha256,
    previous_receipt: failure.context.previous_receipt,
    previous_receipt_sha256: failure.context.previous_receipt_sha256,
    expected_next_ordinal: 17,
    expected_charged_ms: 1380000,
    original_campaign_record: original,
    progress_path: resolve(options.input),
    progress_sha256: await fileDigest(options.input),
    startup_validator_sha256: await cadenceValidatorDigest(options.firmwareRoot),
    installation_directory: "install-001",
    mining_authorized: false,
  };
  context.no_mining_context = innerContext(root, context);
  validateNoMiningContext(context.no_mining_context);
  await writeNew(`${failure.root}.startup-recovery-assignment.json`, {
    schema: "worker-cadence-startup-recovery-assignment-v1",
    failed_inventory_sha256: failure.failed_inventory_sha256,
    context_sha256: digest(JSON.stringify(context)),
    recovery_root: root,
  });
  await (operations.mkdir ?? mkdir)(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await snapshotSources(root, context);
  return { startup_recovery_preflight_created: true, device_effects: false, mining_authorized: false, install_consumed: false };
}
export async function loadStartupRecoveryContext(root, options = {}) {
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  const { historical = false, operations = {} } = options;
  const path = resolve(root, "context.json");
  await protectedPath(path);
  const saved = await readJson(path),
    context = saved.context;
  exactObject(saved, ["context", "sha256"]);
  exactObject(context, [
    "schema",
    "recovery_id",
    ...SOURCE_KEYS,
    "firmware_root",
    "gate_root",
    "manifest",
    "failed_root",
    "failed_inventory_sha256",
    "previous_receipt",
    "previous_receipt_sha256",
    "expected_next_ordinal",
    "expected_charged_ms",
    "original_campaign_record",
    "progress_path",
    "progress_sha256",
    "startup_validator_sha256",
    "installation_directory",
    "mining_authorized",
    "no_mining_context",
  ]);
  requireCondition(
    saved.sha256 === digest(JSON.stringify(context)) &&
      canonicalBase64(context.recovery_id, 16) &&
      context.schema === STARTUP_RECOVERY_SCHEMA &&
      context.mining_authorized === false &&
      context.expected_next_ordinal === 17 &&
      context.expected_charged_ms === 1380000 &&
      context.installation_directory === "install-001" &&
      context.failed_root !== root &&
      dirname(context.failed_root) === dirname(root),
    "startup_recovery_context",
  );
  validateNoMiningContext(context.no_mining_context);
  requireCondition(isDeepStrictEqual(context.no_mining_context, innerContext(root, context)), "startup_inner_context_binding");
  const assignmentPath = `${context.failed_root}.startup-recovery-assignment.json`;
  await protectedPath(assignmentPath);
  requireCondition(
    isDeepStrictEqual(await readJson(assignmentPath), {
      schema: "worker-cadence-startup-recovery-assignment-v1",
      failed_inventory_sha256: context.failed_inventory_sha256,
      context_sha256: saved.sha256,
      recovery_root: root,
    }),
    "startup_recovery_assignment_changed",
  );
  const failure = await readStartupFailure(context.failed_root, operations);
  requireCondition(
    context.failed_inventory_sha256 === failure.failed_inventory_sha256 &&
      context.previous_receipt === failure.context.previous_receipt &&
      context.previous_receipt_sha256 === failure.context.previous_receipt_sha256 &&
      (context.firmware_commit !== failure.context.firmware_commit || context.gate_commit !== failure.context.gate_commit),
    "startup_recovery_failure_changed",
  );
  requireCondition((await readFrozenCampaign(context)) === failure.previous.original_campaign_id, "startup_original_campaign_mismatch");
  await protectedPath(context.progress_path);
  requireCondition((await fileDigest(context.progress_path)) === context.progress_sha256, "startup_progress_changed");
  validateCadenceProgress(await readJson(context.progress_path));
  await verifyArtifactSnapshot(root, context);
  if (!historical) {
    await missing(resolve(root, "result.json"));
    await missing(resolve(root, "failed-inventory.json"));
    await missing(resolve(root, "recovery-failure.json"));
    await requireCadenceTask(context.firmware_root);
    requireCondition((await cadenceValidatorDigest(context.firmware_root)) === context.startup_validator_sha256, "startup_validator_drift");
    const source = await (operations.inspectStartupSources ?? inspectStartupSources)(
      {
        firmwareRoot: context.firmware_root,
        gateRoot: context.gate_root,
        firmwareCommit: context.firmware_commit,
        gateCommit: context.gate_commit,
        manifest: context.manifest,
      },
      operations,
    );
    requireCondition(
      SOURCE_KEYS.every((key) => isDeepStrictEqual(context[key], source[key])),
      "startup_source_drift",
    );
  }
  return context;
}
