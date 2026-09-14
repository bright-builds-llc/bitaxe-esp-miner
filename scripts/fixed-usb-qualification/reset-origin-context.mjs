import { mkdir, open, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual } from "node:util";
import {
  admitTrust,
  canonicalBase64,
  canonicalDirectory,
  cleanPushed,
  digest,
  exactObject,
  fileDigest,
  git,
  hex,
  ignored,
  missing,
  protectedPath,
  readJson,
  requireCondition as check,
  nonce,
  within,
  writeNew,
} from "./contract.mjs";
import { cadenceValidatorDigest, requireCadenceTask } from "./cadence-contract.mjs";
import { loadStartupRecoveryContext } from "./cadence-startup-context.mjs";
import { inventory, proof } from "./cadence-premining-evidence.mjs";
import { inspectOriginalCampaign, readFrozenCampaign } from "./no-mining-accounting.mjs";
import { NO_MINING_SCHEMA, validateNoMiningContext } from "./no-mining-context.mjs";
import { readPrevious } from "./iterative-preflight.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";

export const RESET_ORIGIN_SCHEMA = "fixed-usb-reset-origin-observation-context-v1";
export const RESET_ORIGIN_POLICY = Object.freeze({
  minimum_span_ms: 120000,
  maximum_span_ms: 135000,
  maximum_gap_ms: 6000,
  minimum_boot_advances: 30,
  minimum_startup_advances: 120,
  maximum_records: 4096,
  maximum_batches: 1024,
});
export const RESET_ORIGIN_SOURCE_SEAL = "0ed37434d6bb8587ea51dec6b1e3cf41128ee555635058bd74fd2c84b702e834";
const RUNTIME_KEYS = [
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
const DRIVER_FILES = [
  "reset-origin-context.mjs",
  "reset-origin-judge.mjs",
  "reset-origin-server.mjs",
  "reset-origin-client.mjs",
  "reset-origin-observation.mjs",
];
const ALLOWED_CHANGES = new Set([
  "TASKS.md",
  "scripts/BUILD.bazel",
  "docs/hardware/cpu0-telemetry-cadence-qualification.md",
  "docs/hardware/reset-origin-recovery.md",
  "docs/parity/evidence/20260914-cpu0-cadence-recovery-startup-blocker.md",
  "docs/parity/evidence/20260914-reset-origin-observation.md",
  "scripts/fixed-usb-qualification/main.mjs",
  ...[
    ...DRIVER_FILES,
    "reset-origin-context.test.mjs",
    "reset-origin-judge.test.mjs",
    "reset-origin-server.test.mjs",
    "reset-origin-client.test.mjs",
    "reset-origin-observation.test.mjs",
    "reset-origin-fixtures.mjs",
  ].map((name) => `scripts/fixed-usb-qualification/${name}`),
]);
const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export function forbidResetOriginEffects(options) {
  check(options.authorityDirectory === undefined && options.poolCredentials === undefined, "reset_origin_credentials_forbidden");
}
async function runtimeSource(root, operations) {
  root = await canonicalDirectory(root);
  const seal = await proof(resolve(root, "failed-inventory.json"));
  check(
    seal.sha256 === (operations.expectedResetOriginSourceSeal ?? RESET_ORIGIN_SOURCE_SEAL) &&
      seal.value.schema === "cpu0-cadence-startup-recovery-failed-inventory-v1" &&
      seal.value.outcome === "stop_impossible_contract" &&
      seal.value.qualification_pass === false &&
      seal.value.device_recovery_claimed === false &&
      seal.value.new_hardware_authorized === false,
    "reset_origin_source_anchor",
  );
  const context = await loadStartupRecoveryContext(root, { historical: true, operations });
  const snapshot = await verifyArtifactSnapshot(root, context);
  check(
    seal.value.context_sha256 === digest(JSON.stringify(context)) &&
      seal.value.artifact_snapshot_sha256 === snapshot.receipt_sha256 &&
      isDeepStrictEqual(seal.value.inventory, await inventory(root)),
    "reset_origin_source_changed",
  );
  return {
    context,
    binding: {
      root,
      failed_inventory_sha256: seal.sha256,
      context_sha256: digest(JSON.stringify(context)),
      artifact_snapshot_sha256: snapshot.receipt_sha256,
    },
  };
}
function validatePlan(value) {
  exactObject(value, ["schema", "review", "reason", "evidence_sha256"]);
  check(
    value.schema === "worker-qualification-progress-v1" &&
      value.review === "verified" &&
      ["prospective_observation", "software_correction"].includes(value.reason) &&
      Array.isArray(value.evidence_sha256) &&
      value.evidence_sha256.length > 0 &&
      value.evidence_sha256.length <= 16 &&
      value.evidence_sha256.every((value) => hex(value, 64)),
    "reset_origin_plan",
  );
}
async function verifyDriver(context, operations) {
  const checkRepo = operations.cleanPushed ?? cleanPushed,
    readGit = operations.git ?? git;
  const driver = context.qualification_driver;
  checkRepo(context.firmware_root, driver.source_commit);
  checkRepo(context.gate_root, context.gate_commit);
  check(
    readGit(context.firmware_root, ["merge-base", context.firmware_commit, driver.source_commit]) === context.firmware_commit,
    "reset_origin_driver_ancestry",
  );
  const changes = readGit(context.firmware_root, ["diff", "--name-status", "--no-renames", context.firmware_commit, driver.source_commit]);
  for (const line of changes.split("\n").filter(Boolean)) {
    const [status, path, extra] = line.split("\t");
    check(
      ["A", "M"].includes(status) &&
        extra === undefined &&
        ALLOWED_CHANGES.has(path) &&
        readGit(context.firmware_root, ["ls-tree", driver.source_commit, "--", path]).startsWith("100644 blob "),
      "reset_origin_driver_forbidden_change",
    );
  }
  check(
    (await cadenceValidatorDigest(context.firmware_root)) === driver.validator_sha256 &&
      (await fileDigest(resolve(context.firmware_root, "scripts/fixed-usb-qualification/reset-origin-client.mjs"))) ===
        driver.client_sha256,
    "reset_origin_driver_drift",
  );
  const trustPath = resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json");
  check(
    (await fileDigest(trustPath)) === context.trust_sha256 &&
      (await fileDigest(resolve(SCRIPT_ROOT, "no-mining-client.mjs"))) === context.supervisor_client_sha256,
    "reset_origin_public_input_drift",
  );
  const trust = await readJson(trustPath);
  admitTrust(trust, trust);
}
function inner(root, context) {
  return {
    schema: NO_MINING_SCHEMA,
    ...Object.fromEntries(RUNTIME_KEYS.map((key) => [key, context[key]])),
    mining_authorized: false,
    required_no_mining_cycles: 4,
    original_campaign_record: context.original_campaign_record,
    firmware_root: context.firmware_root,
    gate_root: resolve(root, "qualified-artifacts/gate"),
    manifest: context.manifest,
  };
}
async function copyRuntime(sourceRoot, root, context) {
  const source = await readJson(resolve(sourceRoot, "artifact-snapshot.json"));
  for (const entry of source.files) {
    const from = within(resolve(sourceRoot, "qualified-artifacts"), resolve(sourceRoot, "qualified-artifacts", entry.path));
    const to = within(resolve(root, "qualified-artifacts"), resolve(root, "qualified-artifacts", entry.path));
    await protectedPath(from);
    const bytes = await readFile(from);
    check(bytes.length === entry.length && digest(bytes) === entry.sha256, "reset_origin_copy_changed");
    await mkdir(dirname(to), { recursive: true, mode: 0o700 });
    const file = await open(to, "wx", 0o600);
    try {
      await file.writeFile(bytes);
      await file.sync();
    } finally {
      await file.close();
    }
  }
  await writeNew(resolve(root, "artifact-snapshot.json"), {
    schema: "fixed-usb-qualified-artifacts-v1",
    context_sha256: digest(JSON.stringify(context)),
    files: source.files,
  });
  await verifyArtifactSnapshot(root, context);
}
export async function resetOriginPreflight(options, operations = {}) {
  forbidResetOriginEffects(options);
  const root = resolve(options.privateRoot);
  await protectedPath(dirname(root), true);
  await missing(root);
  for (const key of ["firmwareRoot", "gateRoot", "predecessorRoot"]) options[key] = await canonicalDirectory(options[key]);
  await requireCadenceTask(options.firmwareRoot);
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const source = await runtimeSource(options.predecessorRoot, operations),
    runtime = source.context;
  check(
    root !== source.binding.root &&
      dirname(root) === dirname(source.binding.root) &&
      options.firmwareRoot === runtime.firmware_root &&
      options.gateRoot === runtime.gate_root &&
      (options.firmwareCommit === undefined || options.firmwareCommit === runtime.firmware_commit) &&
      (options.gateCommit === undefined || options.gateCommit === runtime.gate_commit) &&
      (options.manifest === undefined ||
        resolve(options.manifest) === resolve(source.binding.root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json")) &&
      (options.previousReceipt === undefined || resolve(options.previousReceipt) === runtime.previous_receipt),
    "reset_origin_runtime_selection",
  );
  await protectedPath(options.input);
  validatePlan(await readJson(options.input));
  const previous = await (operations.readPrevious ?? readPrevious)(runtime.previous_receipt),
    original = await inspectOriginalCampaign(options.originalCampaignRecord);
  check(
    (await readFrozenCampaign({ original_campaign_record: original })) === previous.original_campaign_id,
    "reset_origin_campaign_mismatch",
  );
  const context = {
    schema: RESET_ORIGIN_SCHEMA,
    observation_id: nonce(),
    ...Object.fromEntries(RUNTIME_KEYS.map((key) => [key, runtime[key]])),
    firmware_root: options.firmwareRoot,
    gate_root: options.gateRoot,
    manifest: resolve(root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"),
    runtime_source: source.binding,
    qualification_driver: {
      profile: "reset-origin-observation-driver-v1",
      source_commit: options.qualificationSourceCommit,
      validator_sha256: await cadenceValidatorDigest(options.firmwareRoot),
      client_sha256: await fileDigest(resolve(options.firmwareRoot, "scripts/fixed-usb-qualification/reset-origin-client.mjs")),
    },
    previous_receipt: runtime.previous_receipt,
    previous_receipt_sha256: runtime.previous_receipt_sha256,
    expected_next_ordinal: 17,
    expected_charged_ms: 1380000,
    original_campaign_record: original,
    plan_path: resolve(options.input),
    plan_sha256: await fileDigest(options.input),
    observation_policy: RESET_ORIGIN_POLICY,
    mining_authorized: false,
    restart_authorized: false,
    installation_authorized: false,
  };
  check(hex(context.qualification_driver.source_commit, 40), "reset_origin_driver_commit");
  context.no_mining_context = inner(root, context);
  validateNoMiningContext(context.no_mining_context);
  await verifyDriver(context, operations);
  await writeNew(`${source.binding.root}.reset-origin-assignment.json`, {
    schema: "fixed-usb-reset-origin-assignment-v1",
    context_sha256: digest(JSON.stringify(context)),
    observation_root: root,
  });
  await (operations.mkdir ?? mkdir)(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await copyRuntime(source.binding.root, root, context);
  return {
    observation_preflight_created: true,
    mining_authorized: false,
    restart_authorized: false,
    installation_authorized: false,
    device_effects: false,
  };
}
export async function loadResetOriginContext(root, { historical = false, operations = {} } = {}) {
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  const saved = await proof(resolve(root, "context.json")),
    context = saved.value.context;
  exactObject(saved.value, ["context", "sha256"]);
  exactObject(context, [
    "schema",
    "observation_id",
    ...RUNTIME_KEYS,
    "firmware_root",
    "gate_root",
    "manifest",
    "runtime_source",
    "qualification_driver",
    "previous_receipt",
    "previous_receipt_sha256",
    "expected_next_ordinal",
    "expected_charged_ms",
    "original_campaign_record",
    "plan_path",
    "plan_sha256",
    "observation_policy",
    "mining_authorized",
    "restart_authorized",
    "installation_authorized",
    "no_mining_context",
  ]);
  exactObject(context.runtime_source, ["root", "failed_inventory_sha256", "context_sha256", "artifact_snapshot_sha256"]);
  exactObject(context.qualification_driver, ["profile", "source_commit", "validator_sha256", "client_sha256"]);
  check(
    saved.value.sha256 === digest(JSON.stringify(context)) &&
      context.schema === RESET_ORIGIN_SCHEMA &&
      canonicalBase64(context.observation_id, 16) &&
      context.mining_authorized === false &&
      context.restart_authorized === false &&
      context.installation_authorized === false &&
      context.expected_next_ordinal === 17 &&
      context.expected_charged_ms === 1380000 &&
      context.qualification_driver.profile === "reset-origin-observation-driver-v1" &&
      hex(context.qualification_driver.source_commit, 40) &&
      hex(context.qualification_driver.validator_sha256, 64) &&
      hex(context.qualification_driver.client_sha256, 64) &&
      isDeepStrictEqual(context.observation_policy, RESET_ORIGIN_POLICY),
    "reset_origin_context",
  );
  check(context.runtime_source.root !== root && dirname(context.runtime_source.root) === dirname(root), "reset_origin_source_relation");
  const source = await runtimeSource(context.runtime_source.root, operations);
  check(
    isDeepStrictEqual(source.binding, context.runtime_source) &&
      RUNTIME_KEYS.every((key) => isDeepStrictEqual(context[key], source.context[key])) &&
      context.previous_receipt === source.context.previous_receipt &&
      context.previous_receipt_sha256 === source.context.previous_receipt_sha256 &&
      context.manifest === resolve(root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"),
    "reset_origin_source_binding",
  );
  const previous = await (operations.readPrevious ?? readPrevious)(context.previous_receipt);
  check(
    previous.next_ordinal === 17 &&
      previous.total_charged_ms === 1380000 &&
      (await readFrozenCampaign(context)) === previous.original_campaign_id,
    "reset_origin_predecessor_accounting",
  );
  validateNoMiningContext(context.no_mining_context);
  check(isDeepStrictEqual(context.no_mining_context, inner(root, context)), "reset_origin_inner_binding");
  const assignment = await proof(`${context.runtime_source.root}.reset-origin-assignment.json`);
  check(
    isDeepStrictEqual(assignment.value, {
      schema: "fixed-usb-reset-origin-assignment-v1",
      context_sha256: saved.value.sha256,
      observation_root: root,
    }),
    "reset_origin_assignment_changed",
  );
  await protectedPath(context.plan_path);
  check((await fileDigest(context.plan_path)) === context.plan_sha256, "reset_origin_plan_changed");
  validatePlan(await readJson(context.plan_path));
  await verifyArtifactSnapshot(root, context);
  if (!historical) {
    await requireCadenceTask(context.firmware_root);
    for (const name of ["result.json", "failed-inventory.json", "reset-origin-failure.json"]) await missing(resolve(root, name));
    await verifyDriver(context, operations);
  }
  return context;
}
export async function verifyResetOriginFrozen(root, context, operations = {}) {
  check(isDeepStrictEqual(await loadResetOriginContext(root, { operations }), context), "reset_origin_context_changed");
}
