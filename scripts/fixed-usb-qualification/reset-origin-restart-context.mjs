import { readRestartStorageFailure, RESTART_STORAGE_FAILURE_SHA256 } from "./reset-origin-restart-storage-failure.mjs";
import { readRestartNetworkFailure, RESTART_NETWORK_FAILURE_SHA256 } from "./reset-origin-restart-network-failure.mjs";
import { mkdir, open, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual as equal } from "node:util";
import {
  BUNDLE,
  canonicalBase64,
  canonicalDirectory,
  digest,
  exactObject,
  fileDigest,
  hex,
  ignored,
  missing,
  nonce,
  protectedPath,
  readJson,
  requireCondition as check,
  writeNew,
} from "./contract.mjs";
import { cadenceValidatorDigest, requireCadenceTask, validateCadenceProgress } from "./cadence-contract.mjs";
import { inspectStartupSources } from "./cadence-startup-context.mjs";
import { readResetOrigin } from "./reset-origin-judge.mjs";
import { inspectOriginalCampaign } from "./no-mining-accounting.mjs";
import { NO_MINING_SCHEMA, validateNoMiningContext } from "./no-mining-context.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { RUNTIME_KEYS } from "./reset-origin-runtime-source.mjs";
import { readRestartInstallFailure, RESTART_INSTALL_FAILURE_SHA256 } from "./reset-origin-restart-install-failure.mjs";
import { RESET_ORIGIN_POLICY } from "./reset-origin-context.mjs";

export const RESTART_SCHEMA = "fixed-usb-reset-origin-restart-context-v1";
export const RESTART_LIMITS = Object.freeze({
  maximum_installations: 1,
  maximum_restarts: 1,
  maximum_ms: 30000,
  maximum_records: 512,
  maximum_bytes: 262144,
  maximum_reopens: 1,
});
const HERE = dirname(fileURLToPath(import.meta.url));
const REQUEST_SCOPE = Symbol("restart-reader-scope");
function requestScope(operations) {
  if (operations[REQUEST_SCOPE]) return operations;
  const cache = new Map(),
    read = operations.readStageA ?? readResetOrigin;
  return Object.assign(Object.create(operations), {
    [REQUEST_SCOPE]: true,
    readStageA(path) {
      const key = resolve(path);
      if (!cache.has(key)) cache.set(key, read(key));
      return cache.get(key);
    },
  });
}
export function rejectRestartCredentials(options) {
  check(
    options.authorityDirectory === undefined && options.poolCredentials === undefined && options.context === undefined,
    "restart_credentials_forbidden",
  );
}
export function restartInnerContext(root, context, phase) {
  check(["before-install", "after-install"].includes(phase), "restart_phase");
  const source = phase === "before-install" ? context.before_source : context;
  return {
    schema: NO_MINING_SCHEMA,
    ...Object.fromEntries(RUNTIME_KEYS.map((key) => [key, source[key]])),
    gate_commit: context.gate_commit,
    gate_bundle_sha256: context.gate_bundle_sha256,
    gate_page_relative_path: context.gate_page_relative_path,
    gate_page_sha256: context.gate_page_sha256,
    trust_sha256: context.trust_sha256,
    supervisor_client_sha256: context.driver.no_mining_client_sha256,
    required_no_mining_cycles: 4,
    mining_authorized: false,
    original_campaign_record: context.original_campaign_record,
    firmware_root: context.firmware_root,
    gate_root: resolve(root, "qualified-artifacts/gate"),
    manifest:
      phase === "before-install" ? context.before_manifest : resolve(root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"),
  };
}
async function retain(path, bytes) {
  await mkdir(dirname(path), { recursive: true, mode: 0o700 });
  const handle = await open(path, "wx", 0o600);
  try {
    await handle.writeFile(bytes);
    await handle.sync();
  } finally {
    await handle.close();
  }
}
async function snapshot(root, context) {
  const entries = [],
    manifest = await readJson(context.manifest);
  async function add(name, source) {
    const bytes = await readFile(source);
    await retain(resolve(root, "qualified-artifacts", name), bytes);
    entries.push({ path: name, sha256: digest(bytes), length: bytes.length });
  }
  await add("firmware/bitaxe-ultra205-package.json", context.manifest);
  for (const item of manifest.artifacts)
    await add(
      `firmware/${item.path}`,
      resolve(item.kind === "partition_table" ? context.firmware_root : dirname(context.manifest), item.path),
    );
  for (const name of ["license-inventory", "provenance-manifest"])
    await add(`firmware/docs/release/${name}.md`, resolve(context.firmware_root, `docs/release/${name}.md`));
  for (const name of [BUNDLE, context.gate_page_relative_path]) await add(`gate/${name}`, resolve(context.gate_root, name));
  await writeNew(resolve(root, "artifact-snapshot.json"), {
    schema: "fixed-usb-qualified-artifacts-v1",
    context_sha256: digest(JSON.stringify(context)),
    files: entries,
  });
  for (const name of ["no-mining-client.mjs", "reset-origin-restart-client.mjs"])
    await retain(resolve(root, "host-clients", name), await readFile(resolve(HERE, name)));
  await retain(resolve(root, "host-clients/process-observer.mjs"), await readFile(context.process_observer.path));
  await verifyArtifactSnapshot(root, context);
}
function assignmentPath(context) {
  return `${dirname(context.stage_a.path)}.restart-assignment${context.restart_attempt === undefined ? "" : `-${context.restart_attempt}`}.json`;
}
function successorAttempt(failed) {
  if (failed.binding.failed_inventory_sha256 === RESTART_INSTALL_FAILURE_SHA256 && failed.context.restart_attempt === undefined) return 2;
  if (failed.binding.failed_inventory_sha256 === RESTART_NETWORK_FAILURE_SHA256 && failed.context.restart_attempt === 2) return 3;
  if (failed.binding.failed_inventory_sha256 === RESTART_STORAGE_FAILURE_SHA256 && failed.context.restart_attempt === 3) return 4;
  check(false, "restart_install_failure_anchor");
}
/** Select only known anchored failures before reading or traversing their lineage. */
export async function readRestartInstallPredecessor(path, operations = {}) {
  path = await canonicalDirectory(path);
  await protectedPath(path, true);
  const sealPath = resolve(path, "failed-inventory.json");
  await protectedPath(sealPath);
  const bytes = await readFile(sealPath),
    hash = digest(bytes);
  check(
    [RESTART_INSTALL_FAILURE_SHA256, RESTART_NETWORK_FAILURE_SHA256, RESTART_STORAGE_FAILURE_SHA256].includes(hash),
    "restart_install_failure_anchor",
  );
  const seal = JSON.parse(bytes.toString("utf8")),
    contextPath = resolve(path, "context.json");
  await protectedPath(contextPath);
  const entry = seal.files?.find((value) => value.path === "context.json" && value.type === "file");
  check(entry && entry.sha256 === (await fileDigest(contextPath)), "restart_failure_context_changed");
  if (hash === RESTART_INSTALL_FAILURE_SHA256) return readRestartInstallFailure(path, operations);
  if (hash === RESTART_NETWORK_FAILURE_SHA256) return readRestartNetworkFailure(path, operations);
  return readRestartStorageFailure(path, operations);
}
async function failedInstallation(path, operations) {
  const failed = await (operations.readInstallFailure ?? readRestartInstallPredecessor)(path, operations);
  successorAttempt(failed);
  return failed;
}
async function requireStatisticsCapability(gateRoot, expectedHash) {
  const bytes = await readFile(resolve(gateRoot, BUNDLE));
  check(digest(bytes) === expectedHash && bytes.includes("statistics_startup schema=v1 state="), "restart_statistics_capability_missing");
}
async function stageA(path, operations) {
  const receipt = await (operations.readStageA ?? readResetOrigin)(path);
  check(
    receipt.result === "observed_stable" &&
      receipt.observation_qualified === true &&
      receipt.mining_authorized === false &&
      receipt.ledger.next_ordinal === 17 &&
      receipt.ledger.total_charged_ms === 1380000 &&
      !receipt.ledger.pending,
    "restart_stage_a_required",
  );
  return receipt;
}
/** Effect-free preparation; reserves the sole successor before creating any child. */
export async function restartPreflight(options, operations = {}) {
  operations = requestScope(operations);
  rejectRestartCredentials(options);
  const root = resolve(options.privateRoot);
  await protectedPath(dirname(root), true);
  await missing(root);
  for (const key of ["firmwareRoot", "gateRoot"]) options[key] = await canonicalDirectory(options[key]);
  await requireCadenceTask(options.firmwareRoot);
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const priorPath = resolve(options.stageAResult),
    prior = await stageA(priorPath, operations);
  check(dirname(root) === dirname(dirname(priorPath)) && root !== dirname(priorPath), "restart_stage_a_relation");
  await protectedPath(options.input);
  validateCadenceProgress(await readJson(options.input));
  await protectedPath(options.observerScript);
  check(options.observerScript.endsWith(".mjs"), "restart_observer_source");
  const failed =
    options.supersedeInstallFailure === undefined
      ? undefined
      : await failedInstallation(resolve(options.supersedeInstallFailure), operations);
  if (failed)
    check(
      (await readJson(options.input)).evidence_sha256.includes(failed.binding.failed_inventory_sha256) &&
        equal(failed.context.stage_a, {
          path: priorPath,
          sha256: await fileDigest(priorPath),
          context_sha256: digest(JSON.stringify(prior.context)),
        }) &&
        dirname(failed.root) === dirname(root) &&
        failed.root !== root,
      "restart_install_failure_relation",
    );
  const beforeSource = failed?.context ?? prior.context;
  const source = await (operations.inspectSources ?? inspectStartupSources)(options, operations);
  if (failed) await requireStatisticsCapability(options.gateRoot, source.gate_bundle_sha256);
  check(
    source.firmware_commit !== beforeSource.firmware_commit &&
      source.gate_commit !== prior.context.gate_commit &&
      (!failed ||
        source.gate_commit !== failed.context.gate_commit ||
        ["gate_bundle_sha256", "gate_page_sha256", "gate_page_relative_path"].every((key) => source[key] === failed.context[key])) &&
      source.trust_sha256 === prior.context.trust_sha256,
    "restart_new_pair_required",
  );
  const original = await inspectOriginalCampaign(options.originalCampaignRecord);
  check(equal(original, prior.context.original_campaign_record), "restart_original_campaign_changed");
  const context = {
    schema: RESTART_SCHEMA,
    restart_id: nonce(),
    request_nonce: nonce(),
    ...source,
    firmware_root: options.firmwareRoot,
    gate_root: options.gateRoot,
    manifest: resolve(options.manifest),
    stage_a: { path: priorPath, sha256: await fileDigest(priorPath), context_sha256: digest(JSON.stringify(prior.context)) },
    before_source: Object.fromEntries(RUNTIME_KEYS.map((key) => [key, beforeSource[key]])),
    before_manifest: failed ? resolve(failed.root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json") : prior.context.manifest,
    ...(failed
      ? {
          restart_attempt: successorAttempt(failed),
          install_failure_predecessor: failed.binding,
          before_install_failure: failed.known_failure,
          statistics_startup_required: true,
        }
      : {}),
    original_campaign_record: original,
    expected_next_ordinal: 17,
    expected_charged_ms: 1380000,
    driver: {
      source_commit: options.firmwareCommit,
      validator_sha256: await cadenceValidatorDigest(options.firmwareRoot),
      client_sha256: await fileDigest(resolve(HERE, "reset-origin-restart-client.mjs")),
      no_mining_client_sha256: await fileDigest(resolve(HERE, "no-mining-client.mjs")),
    },
    process_observer: { path: resolve(options.observerScript), sha256: await fileDigest(options.observerScript) },
    progress: { path: resolve(options.input), sha256: await fileDigest(options.input) },
    observation_policy: RESET_ORIGIN_POLICY,
    limits: RESTART_LIMITS,
    installation_directory: "install-001",
    mining_authorized: false,
  };
  if (failed)
    check(
      context.restart_id !== failed.context.restart_id && context.request_nonce !== failed.context.request_nonce,
      "restart_successor_nonce_reused",
    );
  for (const phase of ["before-install", "after-install"]) validateNoMiningContext(restartInnerContext(root, context, phase));
  await writeNew(assignmentPath(context), {
    schema: "fixed-usb-restart-assignment-v1",
    context_sha256: digest(JSON.stringify(context)),
    root,
  });
  await (operations.mkdir ?? mkdir)(root, { mode: 0o700 });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  for (const phase of ["before-install", "after-install"]) await mkdir(resolve(root, phase), { mode: 0o700 });
  await snapshot(root, context);
  return { restart_preflight_created: true, mining_authorized: false, installations_consumed: 0, restarts_consumed: 0 };
}
export async function loadRestartContext(root, { historical = false, operations = {} } = {}) {
  operations = requestScope(operations);
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  await protectedPath(resolve(root, "context.json"));
  const saved = await readJson(resolve(root, "context.json")),
    context = saved.context;
  exactObject(saved, ["context", "sha256"]);
  exactObject(
    context,
    [
      "schema",
      "restart_id",
      "request_nonce",
      ...RUNTIME_KEYS,
      "firmware_root",
      "gate_root",
      "manifest",
      "stage_a",
      "before_source",
      "before_manifest",
      "original_campaign_record",
      "expected_next_ordinal",
      "expected_charged_ms",
      "driver",
      "process_observer",
      "progress",
      "observation_policy",
      "limits",
      "installation_directory",
      "mining_authorized",
    ],
    ["restart_attempt", "install_failure_predecessor", "before_install_failure", "statistics_startup_required"],
  );
  exactObject(context.driver, ["source_commit", "validator_sha256", "client_sha256", "no_mining_client_sha256"]);
  exactObject(context.stage_a, ["path", "sha256", "context_sha256"]);
  for (const value of [context.process_observer, context.progress]) exactObject(value, ["path", "sha256"]);
  check(
    saved.sha256 === digest(JSON.stringify(context)) &&
      context.schema === RESTART_SCHEMA &&
      canonicalBase64(context.restart_id, 16) &&
      canonicalBase64(context.request_nonce, 16) &&
      context.request_nonce !== context.restart_id &&
      context.mining_authorized === false &&
      context.installation_directory === "install-001" &&
      context.expected_next_ordinal === 17 &&
      context.expected_charged_ms === 1380000 &&
      context.driver.source_commit === context.firmware_commit &&
      Object.entries(context.driver).every(([key, value]) => hex(value, key === "source_commit" ? 40 : 64)) &&
      equal(context.observation_policy, RESET_ORIGIN_POLICY) &&
      equal(context.limits, RESTART_LIMITS),
    "restart_context",
  );
  const prior = await stageA(context.stage_a.path, operations);
  const successor =
    context.restart_attempt !== undefined ||
    context.install_failure_predecessor !== undefined ||
    context.before_install_failure !== undefined ||
    context.statistics_startup_required !== undefined;
  let failed;
  if (successor) {
    check([2, 3, 4].includes(context.restart_attempt) && context.statistics_startup_required === true, "restart_successor_shape");
    exactObject(context.install_failure_predecessor, ["root", "failed_inventory_sha256"]);
    failed = await failedInstallation(context.install_failure_predecessor.root, operations);
    check(
      context.restart_attempt === successorAttempt(failed) &&
        equal(failed.binding, context.install_failure_predecessor) &&
        equal(failed.known_failure, context.before_install_failure) &&
        equal(failed.context.stage_a, context.stage_a) &&
        dirname(failed.root) === dirname(root) &&
        failed.root !== root &&
        context.restart_id !== failed.context.restart_id &&
        context.request_nonce !== failed.context.request_nonce,
      "restart_install_failure_lineage",
    );
  }
  const beforeSource = failed?.context ?? prior.context;
  const beforeManifest = failed
    ? resolve(failed.root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json")
    : prior.context.manifest;
  check(
    (await fileDigest(context.stage_a.path)) === context.stage_a.sha256 &&
      digest(JSON.stringify(prior.context)) === context.stage_a.context_sha256 &&
      equal(context.before_source, Object.fromEntries(RUNTIME_KEYS.map((key) => [key, beforeSource[key]]))) &&
      context.before_manifest === beforeManifest &&
      equal(context.original_campaign_record, prior.context.original_campaign_record) &&
      context.firmware_commit !== beforeSource.firmware_commit &&
      context.gate_commit !== prior.context.gate_commit &&
      (!failed ||
        context.gate_commit !== failed.context.gate_commit ||
        ["gate_bundle_sha256", "gate_page_sha256", "gate_page_relative_path"].every((key) => context[key] === failed.context[key])) &&
      context.trust_sha256 === prior.context.trust_sha256 &&
      dirname(root) === dirname(dirname(context.stage_a.path)) &&
      root !== dirname(context.stage_a.path),
    "restart_stage_a_changed",
  );
  await protectedPath(assignmentPath(context));
  check(
    equal(await readJson(assignmentPath(context)), {
      schema: "fixed-usb-restart-assignment-v1",
      context_sha256: saved.sha256,
      root,
    }),
    "restart_assignment_changed",
  );
  await protectedPath(context.progress.path);
  check((await fileDigest(context.progress.path)) === context.progress.sha256, "restart_progress_changed");
  const progress = await readJson(context.progress.path);
  validateCadenceProgress(progress);
  if (successor) check(progress.evidence_sha256.includes(failed.binding.failed_inventory_sha256), "restart_successor_progress");
  await verifyArtifactSnapshot(root, context);
  for (const [name, hash] of [
    ["no-mining-client.mjs", context.driver.no_mining_client_sha256],
    ["reset-origin-restart-client.mjs", context.driver.client_sha256],
    ["process-observer.mjs", context.process_observer.sha256],
  ]) {
    await protectedPath(resolve(root, "host-clients", name));
    check((await fileDigest(resolve(root, "host-clients", name))) === hash, "restart_host_source_changed");
  }
  for (const phase of ["before-install", "after-install"]) validateNoMiningContext(restartInnerContext(root, context, phase));
  if (!historical) {
    await requireCadenceTask(context.firmware_root);
    for (const name of ["result.json", "failed-inventory.json", "restart-failure.json"]) await missing(resolve(root, name));
    const source = await (operations.inspectSources ?? inspectStartupSources)(
      {
        firmwareRoot: context.firmware_root,
        gateRoot: context.gate_root,
        firmwareCommit: context.firmware_commit,
        gateCommit: context.gate_commit,
        manifest: context.manifest,
      },
      operations,
    );
    if (context.statistics_startup_required) await requireStatisticsCapability(context.gate_root, source.gate_bundle_sha256);
    check(
      RUNTIME_KEYS.every((key) => equal(source[key], context[key])) &&
        (await cadenceValidatorDigest(context.firmware_root)) === context.driver.validator_sha256 &&
        (await fileDigest(resolve(HERE, "reset-origin-restart-client.mjs"))) === context.driver.client_sha256,
      "restart_driver_changed",
    );
  }
  return context;
}
