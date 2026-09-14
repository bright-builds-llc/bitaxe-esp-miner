import { constants } from "node:fs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { copyFile, chmod, mkdir, realpath, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { CADENCE_SCHEMA, CADENCE_LIMITS, CADENCE_DIAGNOSTICS_VERSION, cadenceValidatorDigest, requireCadenceTask, validateCadencePolicy, validateCadenceProgress } from "./cadence-contract.mjs";
import { canonicalDirectory, digest, fileDigest, ignored, missing, nonce, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { validateAttempt, requireExhaustedOriginal } from "./iterative-contract.mjs";
import { readPrevious } from "./iterative-preflight.mjs";
import { inspectSources } from "./preflight.mjs";
import { readStartupRecovery } from "./cadence-startup-recovery.mjs";
import { readUnissued } from "./cadence-unissued.mjs";
import { preminingClosurePath, readPremining } from "./cadence-premining.mjs";

function assignmentPath(root, context) {
  const preparation = context.preparation_attempt;
  requireCondition(preparation === undefined || (Number.isSafeInteger(preparation) && preparation >= 2), "cadence_preparation_shape");
  const sources = [context.unissued_predecessor, context.premining_predecessor, context.startup_predecessor].filter(value => value !== undefined);
  requireCondition(sources.length === (preparation === undefined ? 0 : 1) && sources.every(value => value !== null && typeof value === "object" && !Array.isArray(value)), "cadence_preparation_shape");
  return resolve(dirname(root), `ordinal-${context.qualification_attempt.ordinal}${preparation === undefined ? "" : `-preparation-${preparation}`}.json`);
}

function requireUnissuedLineage(prior, context, previousPath, previousHash) {
  const old = prior.context;
  requireCondition(old.previous_receipt === previousPath && old.previous_receipt_sha256 === previousHash &&
    old.qualification_attempt.ordinal === context.qualification_attempt.ordinal && old.expected_charged_ms === context.expected_charged_ms &&
    old.original_campaign_id === context.original_campaign_id && old.qualification_attempt.id !== context.qualification_attempt.id &&
    context.preparation_attempt === (old.preparation_attempt ?? 1) + 1, "cadence_unissued_lineage");
  requireCondition(old.firmware_commit !== context.firmware_commit || old.gate_commit !== context.gate_commit, "cadence_unchanged_preparation");
}

async function requireStartupLineage(recovery, context, previousPath) {
  const recovered = recovery.context;
  const failedPath = resolve(recovered.failed_root, "context.json"); await protectedPath(failedPath);
  const failed = (await readJson(failedPath)).context;
  requireCondition(recovered.previous_receipt === previousPath && recovered.previous_receipt_sha256 === context.previous_receipt_sha256 &&
    recovery.ledger.next_ordinal === context.qualification_attempt.ordinal && recovery.ledger.total_charged_ms === context.expected_charged_ms &&
    context.qualification_attempt.ordinal === 17 && context.preparation_attempt === 2 && context.qualification_attempt.id !== failed.qualification_attempt.id,
    "cadence_startup_lineage");
  for (const key of ["firmware_commit", "gate_commit", "manifest_sha256", "app_elf_sha256", "reference_commit", "artifacts", "update_segments",
    "gate_bundle_sha256", "gate_page_relative_path", "gate_page_sha256", "trust_sha256"]) requireCondition(
      JSON.stringify(context[key]) === JSON.stringify(recovered[key]), "cadence_startup_recovered_pair_changed");
}

export async function cadencePreflight(options, operations = {}) {
  requireCondition([options.supersedeUnissued, options.supersedePremining, options.supersedeStartup].filter(Boolean).length <= 1, "cadence_supersession_exclusive");
  requireCondition(options.suggestedDifficulty === "1000" && options.observerBinary, "cadence_arguments");
  const root = resolve(options.privateRoot), parent = dirname(root);
  await protectedPath(parent, true); await missing(root);
  for (const key of ["firmwareRoot", "gateRoot", "authorityDirectory"]) options[key] = await canonicalDirectory(options[key]);
  await requireCadenceTask(options.firmwareRoot);
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const previousPath = resolve(options.previousReceipt);
  requireCondition(dirname(dirname(previousPath)) === parent && previousPath === resolve(dirname(previousPath), "result.json"), "cadence_existing_chain_required");
  const previous = await (operations.readPrevious ?? readPrevious)(previousPath);
  requireCondition(previous.context && previous.cleanup_confirmed === true, "cadence_completed_predecessor");
  requireExhaustedOriginal(previous.original_budget);
  await (operations.verifyArtifactSnapshot ?? verifyArtifactSnapshot)(dirname(previousPath), previous.context);
  const unissuedPath = options.supersedeUnissued ? resolve(options.supersedeUnissued) : undefined;
  const unissued = unissuedPath ? await readUnissued(unissuedPath, operations) : undefined;
  const preminingPath = options.supersedePremining ? resolve(options.supersedePremining) : undefined;
  const premining = preminingPath ? await readPremining(preminingPath, operations) : undefined;
  const startupPath = options.supersedeStartup ? resolve(options.supersedeStartup) : undefined;
  const startup = startupPath ? await readStartupRecovery(startupPath, operations) : undefined;
  if (startup) requireCondition(dirname(dirname(startupPath)) === parent && dirname(startupPath) !== root, "cadence_startup_sibling_required");
  const superseded = unissued ?? premining;
  if (premining) requireCondition(dirname(premining.root) === parent && premining.root !== root, "cadence_premining_sibling_required");
  if (unissued) requireCondition(dirname(unissued.root) === parent && unissued.root !== root, "cadence_unissued_sibling_required");
  await protectedPath(options.input);
  const progress = await readJson(options.input); validateCadenceProgress(progress, Boolean(unissued));
  const snapshot = await (operations.inspectSources ?? inspectSources)({ ...options, cadence: true }, operations);
  requireCondition(previous.context.firmware_commit !== snapshot.firmware_commit || previous.context.gate_commit !== snapshot.gate_commit,
    "cadence_unchanged_retry");
  const observerPath = await realpath(options.observerBinary), observerStat = await stat(observerPath);
  const canonicalObserver = await realpath(resolve(options.firmwareRoot, "bazel-bin/tools/http-transport/cadence_observer"));
  requireCondition(observerPath === canonicalObserver, "cadence_observer_not_canonical");
  requireCondition(observerStat.isFile() && (observerStat.mode & 0o111) !== 0, "cadence_observer_executable");
  const attempt = validateAttempt({ schema: "worker-qualification-attempt-v1", id: nonce(), ordinal: previous.next_ordinal,
    purpose: "normal", maximumActiveMilliseconds: 180000 });
  requireCondition(Number.isSafeInteger(previous.total_charged_ms + 180000), "cadence_ledger_overflow");
  const context = { schema: CADENCE_SCHEMA, cadence_diagnostics_version: CADENCE_DIAGNOSTICS_VERSION, owner_stack_minimum_bytes: 4096, suggested_difficulty: 1000,
    ...snapshot, qualification_attempt: attempt, required_no_mining_cycles: 4,
    original_campaign_id: previous.original_campaign_id, previous_receipt: previousPath,
    previous_receipt_sha256: await fileDigest(previousPath), expected_charged_ms: previous.total_charged_ms,
    progress_path: resolve(options.input), progress_sha256: await fileDigest(options.input),
    firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: resolve(options.manifest),
    cadence_limits: CADENCE_LIMITS, cadence_observer: { path: observerPath, sha256: await fileDigest(observerPath) },
    cadence_validator_sha256: await cadenceValidatorDigest(options.firmwareRoot),
    ...(superseded ? { preparation_attempt: (superseded.context.preparation_attempt ?? 1) + 1,
      [unissued ? "unissued_predecessor" : "premining_predecessor"]: { root: superseded.root,
        closure_sha256: await fileDigest(unissuedPath ?? preminingPath) } } : {}),
    ...(startup ? { preparation_attempt: 2, startup_predecessor: { root: dirname(startupPath), receipt_sha256: await fileDigest(startupPath) } } : {}) };
  validateCadencePolicy(context);
  if (superseded) requireUnissuedLineage(superseded, context, previousPath, context.previous_receipt_sha256);
  if (startup) await requireStartupLineage(startup, context, previousPath);
  const markerPath = assignmentPath(root, context); await missing(markerPath);
  const marker = { context_sha256: digest(JSON.stringify(context)), attempt_root: root };
  // An interrupted pre-mining successor retains its preparation assignment permanently.
  if (premining || startup) await writeNew(markerPath, marker);
  await (operations.mkdir ?? mkdir)(root, { mode: 0o700 });
  const retainedObserver = resolve(root, "cadence-observer.bin");
  await (operations.copyFile ?? copyFile)(observerPath, retainedObserver, constants.COPYFILE_EXCL);
  await chmod(retainedObserver, 0o600);
  requireCondition(await fileDigest(retainedObserver) === context.cadence_observer.sha256, "cadence_observer_snapshot");
  if (!premining && !startup) await writeNew(markerPath, marker);
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  return { cadence_preflight_created: true, ordinal: attempt.ordinal, reserved_on_device: false, maximum_active_ms: 180000, device_effects: false };
}

export async function validateCadenceContext(root, context, { historical = false, operations = {} } = {}) {
  validateCadencePolicy(context); validateAttempt(context.qualification_attempt);
  const markerPath = assignmentPath(root, context);
  if (!historical) {
    await missing(resolve(root, "unissued-closure.json"));
    await missing(resolve(root, "failed-inventory.json"));
    await missing(preminingClosurePath(root));
    await requireCadenceTask(context.firmware_root);
    requireCondition(await cadenceValidatorDigest(context.firmware_root) === context.cadence_validator_sha256, "cadence_validator_drift");
    requireCondition(await fileDigest(context.cadence_observer.path) === context.cadence_observer.sha256, "cadence_observer_drift");
  }
  await protectedPath(resolve(root, "cadence-observer.bin"));
  requireCondition(await fileDigest(resolve(root, "cadence-observer.bin")) === context.cadence_observer.sha256, "cadence_observer_snapshot");
  const previousPath = resolve(context.previous_receipt);
  requireCondition(dirname(dirname(previousPath)) === dirname(root) && previousPath === resolve(dirname(previousPath), "result.json") &&
    await fileDigest(previousPath) === context.previous_receipt_sha256, "cadence_previous_changed");
  const previous = await (operations.readPrevious ?? readPrevious)(previousPath);
  requireCondition(previous.next_ordinal === context.qualification_attempt.ordinal && previous.total_charged_ms === context.expected_charged_ms &&
    previous.original_campaign_id === context.original_campaign_id, "cadence_ledger_lineage");
  await protectedPath(context.progress_path);
  requireCondition(await fileDigest(context.progress_path) === context.progress_sha256, "cadence_progress_changed");
  const progress = await readJson(context.progress_path);
  requireCondition(previous.context.firmware_commit !== context.firmware_commit || previous.context.gate_commit !== context.gate_commit, "cadence_unchanged_retry");
  if (context.unissued_predecessor) {
    const source = context.unissued_predecessor;
    requireCondition(Object.keys(source).length === 2 && typeof source.root === "string" && dirname(source.root) === dirname(root) && source.root !== root,
      "cadence_unissued_sibling_required");
    const closurePath = resolve(source.root, "unissued-closure.json");
    requireCondition(await fileDigest(closurePath) === source.closure_sha256, "cadence_unissued_closure_changed");
    const prior = await readUnissued(closurePath, operations);
    requireUnissuedLineage(prior, context, previousPath, context.previous_receipt_sha256);
  }
  if (context.premining_predecessor) {
    const source = context.premining_predecessor;
    requireCondition(Object.keys(source).length === 2 && typeof source.root === "string" && dirname(source.root) === dirname(root) && source.root !== root,
      "cadence_premining_sibling_required");
    const closurePath = preminingClosurePath(source.root);
    requireCondition(await fileDigest(closurePath) === source.closure_sha256, "cadence_premining_closure_changed");
    const prior = await readPremining(closurePath, operations);
    requireUnissuedLineage(prior, context, previousPath, context.previous_receipt_sha256);
  }
  if (context.startup_predecessor) {
    const source = context.startup_predecessor;
    requireCondition(Object.keys(source).length === 2 && typeof source.root === "string" && dirname(source.root) === dirname(root) && source.root !== root,
      "cadence_startup_sibling_required");
    const path = resolve(source.root, "result.json");
    requireCondition(await fileDigest(path) === source.receipt_sha256, "cadence_startup_receipt_changed");
    const receipt = await readStartupRecovery(path, operations); await requireStartupLineage(receipt, context, previousPath);
  }
  validateCadenceProgress(progress, Boolean(context.unissued_predecessor));
  await protectedPath(markerPath);
  const marker = await readJson(markerPath);
  requireCondition(marker.attempt_root === root && marker.context_sha256 === digest(JSON.stringify(context)), "cadence_ordinal_replay");
}
