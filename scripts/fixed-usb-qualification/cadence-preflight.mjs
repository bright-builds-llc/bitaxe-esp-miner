import { constants } from "node:fs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { copyFile, chmod, mkdir, realpath, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { CADENCE_SCHEMA, CADENCE_LIMITS, cadenceValidatorDigest, requireCadenceTask, validateCadencePolicy, validateCadenceProgress } from "./cadence-contract.mjs";
import { canonicalDirectory, digest, fileDigest, ignored, missing, nonce, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { validateAttempt, requireExhaustedOriginal } from "./iterative-contract.mjs";
import { readPrevious } from "./iterative-preflight.mjs";
import { inspectSources } from "./preflight.mjs";

export async function cadencePreflight(options, operations = {}) {
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
  await protectedPath(options.input);
  const progress = await readJson(options.input); validateCadenceProgress(progress);
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
  await missing(resolve(parent, `ordinal-${attempt.ordinal}.json`));
  const context = { schema: CADENCE_SCHEMA, owner_stack_minimum_bytes: 4096, suggested_difficulty: 1000,
    ...snapshot, qualification_attempt: attempt, required_no_mining_cycles: 4,
    original_campaign_id: previous.original_campaign_id, previous_receipt: previousPath,
    previous_receipt_sha256: await fileDigest(previousPath), expected_charged_ms: previous.total_charged_ms,
    progress_path: resolve(options.input), progress_sha256: await fileDigest(options.input),
    firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: resolve(options.manifest),
    cadence_limits: CADENCE_LIMITS, cadence_observer: { path: observerPath, sha256: await fileDigest(observerPath) },
    cadence_validator_sha256: await cadenceValidatorDigest(options.firmwareRoot) };
  validateCadencePolicy(context);
  await mkdir(root, { mode: 0o700 });
  const retainedObserver = resolve(root, "cadence-observer.bin");
  await copyFile(observerPath, retainedObserver, constants.COPYFILE_EXCL);
  await chmod(retainedObserver, 0o600);
  requireCondition(await fileDigest(retainedObserver) === context.cadence_observer.sha256, "cadence_observer_snapshot");
  await writeNew(resolve(parent, `ordinal-${attempt.ordinal}.json`), { context_sha256: digest(JSON.stringify(context)), attempt_root: root });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  return { cadence_preflight_created: true, ordinal: attempt.ordinal, reserved_on_device: false, maximum_active_ms: 180000, device_effects: false };
}

export async function validateCadenceContext(root, context, { historical = false } = {}) {
  validateCadencePolicy(context); validateAttempt(context.qualification_attempt);
  if (!historical) {
    await requireCadenceTask(context.firmware_root);
    requireCondition(await cadenceValidatorDigest(context.firmware_root) === context.cadence_validator_sha256, "cadence_validator_drift");
    requireCondition(await fileDigest(context.cadence_observer.path) === context.cadence_observer.sha256, "cadence_observer_drift");
  }
  await protectedPath(resolve(root, "cadence-observer.bin"));
  requireCondition(await fileDigest(resolve(root, "cadence-observer.bin")) === context.cadence_observer.sha256, "cadence_observer_snapshot");
  const previousPath = resolve(context.previous_receipt);
  requireCondition(dirname(dirname(previousPath)) === dirname(root) && previousPath === resolve(dirname(previousPath), "result.json") &&
    await fileDigest(previousPath) === context.previous_receipt_sha256, "cadence_previous_changed");
  const previous = await readPrevious(previousPath);
  requireCondition(previous.next_ordinal === context.qualification_attempt.ordinal && previous.total_charged_ms === context.expected_charged_ms &&
    previous.original_campaign_id === context.original_campaign_id, "cadence_ledger_lineage");
  await protectedPath(context.progress_path);
  requireCondition(await fileDigest(context.progress_path) === context.progress_sha256, "cadence_progress_changed");
  validateCadenceProgress(await readJson(context.progress_path));
  requireCondition(previous.context.firmware_commit !== context.firmware_commit || previous.context.gate_commit !== context.gate_commit, "cadence_unchanged_retry");
  const marker = await readJson(resolve(dirname(root), `ordinal-${context.qualification_attempt.ordinal}.json`));
  requireCondition(marker.attempt_root === root && marker.context_sha256 === digest(JSON.stringify(context)), "cadence_ordinal_replay");
}
