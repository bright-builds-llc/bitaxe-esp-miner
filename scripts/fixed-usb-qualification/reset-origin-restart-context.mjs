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
  const source = await (operations.inspectSources ?? inspectStartupSources)(options, operations);
  check(
    source.firmware_commit !== prior.context.firmware_commit &&
      source.gate_commit !== prior.context.gate_commit &&
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
    before_source: Object.fromEntries(RUNTIME_KEYS.map((key) => [key, prior.context[key]])),
    before_manifest: prior.context.manifest,
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
  for (const phase of ["before-install", "after-install"]) validateNoMiningContext(restartInnerContext(root, context, phase));
  await writeNew(`${dirname(priorPath)}.restart-assignment.json`, {
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
  root = await canonicalDirectory(root);
  await protectedPath(root, true);
  await protectedPath(resolve(root, "context.json"));
  const saved = await readJson(resolve(root, "context.json")),
    context = saved.context;
  exactObject(saved, ["context", "sha256"]);
  exactObject(context, [
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
  ]);
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
  check(
    (await fileDigest(context.stage_a.path)) === context.stage_a.sha256 &&
      digest(JSON.stringify(prior.context)) === context.stage_a.context_sha256 &&
      equal(context.before_source, Object.fromEntries(RUNTIME_KEYS.map((key) => [key, prior.context[key]]))) &&
      context.before_manifest === prior.context.manifest &&
      equal(context.original_campaign_record, prior.context.original_campaign_record) &&
      context.firmware_commit !== prior.context.firmware_commit &&
      context.gate_commit !== prior.context.gate_commit &&
      context.trust_sha256 === prior.context.trust_sha256 &&
      dirname(root) === dirname(dirname(context.stage_a.path)) &&
      root !== dirname(context.stage_a.path),
    "restart_stage_a_changed",
  );
  await protectedPath(`${dirname(context.stage_a.path)}.restart-assignment.json`);
  check(
    equal(await readJson(`${dirname(context.stage_a.path)}.restart-assignment.json`), {
      schema: "fixed-usb-restart-assignment-v1",
      context_sha256: saved.sha256,
      root,
    }),
    "restart_assignment_changed",
  );
  await protectedPath(context.progress.path);
  check((await fileDigest(context.progress.path)) === context.progress.sha256, "restart_progress_changed");
  validateCadenceProgress(await readJson(context.progress.path));
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
    check(
      RUNTIME_KEYS.every((key) => equal(source[key], context[key])) &&
        (await cadenceValidatorDigest(context.firmware_root)) === context.driver.validator_sha256 &&
        (await fileDigest(resolve(HERE, "reset-origin-restart-client.mjs"))) === context.driver.client_sha256,
      "restart_driver_changed",
    );
  }
  return context;
}
