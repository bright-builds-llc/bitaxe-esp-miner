import { isDeepStrictEqual } from "node:util";
import { requireRecoveryTraces } from "./recovery-trace.mjs";
import { RECOVERY_SCHEMA, validateRecoveryPhase } from "./recovery-judge.mjs";
import { resultSamples } from "./sample-seal.mjs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { canonicalBase64, canonicalDirectory, digest, exactObject, fileDigest, hex, ignored, missing, nonce, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { inspectSources } from "./preflight.mjs";
import { validateCycle, validateState } from "./judge.mjs";
import { maximumActiveMs, PURPOSES, requireExhaustedOriginal, requireIdleLedger, validateAttempt } from "./iterative-contract.mjs";
import { copyRetainedArtifacts, inspectRetainedSources, retainedManifestPath } from "./runtime-source.mjs";

export function validateIterativePolicy(context, live = false) {
  const recovery = context.schema === RECOVERY_SCHEMA;
  if (recovery) {
    validateRecoveryPhase(context);
    requireCondition(context.required_no_mining_cycles === 4, "recovery_cycle_policy");
    requireCondition(context.recovery_phase === "loss" ? context.recovery_loss_generation === undefined :
      Number.isInteger(context.recovery_loss_generation) && context.recovery_loss_generation > 0 && context.recovery_loss_generation <= 0xffffffff,
    "recovery_generation_policy");
  }
  const v4 = context.schema === "fixed-usb-iterative-context-v4";
  const v3 = context.schema === "fixed-usb-iterative-context-v3";
  const resources = recovery || v4 || v3 || context.schema === "fixed-usb-iterative-context-v2";
  requireCondition(resources ? context.owner_stack_minimum_bytes === 4096 :
    context.schema === "fixed-usb-iterative-context-v1" && context.owner_stack_minimum_bytes === undefined, "iterative_policy");
  requireCondition(recovery || v4 || v3 ? context.suggested_difficulty === 1000 : context.suggested_difficulty === undefined, "iterative_hint_policy");
  if (v4) {
    exactObject(context.qualification_driver, ["profile", "source_commit"]);
    requireCondition(context.qualification_driver.profile === "fixed-usb-retained-runtime-driver-v1" && hex(context.qualification_driver.source_commit, 40), "iterative_driver_policy");
  } else requireCondition(context.qualification_driver === undefined && context.unreserved_continuation === undefined && context.retained_runtime_source === undefined, "iterative_driver_policy");
  requireCondition(!live || recovery || v3 || v4, "iterative_policy_upgrade_required");
  return resources;
}
export async function requireIterativeTask(firmwareRoot, recovery = false) {
  const tasks = await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8");
  const task = recovery ? "task-fixed-usb-hello-resynchronization" : "task-worker-preparation-panic-qualification";
  let active = false, count = 0, total = 0;
  for (const line of tasks.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    if (line.startsWith("### ") && line.slice(4).split(/\s/u)[0] === task) {
      total += 1;
      if (active) count += 1;
    }
  }
  requireCondition(count === 1 && total === 1, recovery ? "recovery_active_task_required" : "iterative_active_task_required");
}
export function requireReleasedState(state, context) {
  validateState(state, context);
  requireCondition(!state.connected && !state.running && state.deviceRestorationConfirmed && state.deviceLeaseInactive &&
    state.serialOwnershipReleased && state.preservation?.device_identity_match === true && state.preservation.settings_match === true &&
    state.preservation.mine_on_boot === false, "iterative_cleanup_required");
}
async function protectedJson(path) { await protectedPath(path); return readJson(path); }
export async function iterativeBootstrap(root, inputPath, operations = {}) {
  await protectedPath(dirname(root), true); await missing(root);
  const input = await protectedJson(inputPath);
  exactObject(input, ["original_context_path", "budget_review_path", "final_state_path"]);
  const record = await protectedJson(input.original_context_path);
  requireCondition(record.sha256 === digest(JSON.stringify(record.context)) && canonicalBase64(record.context.campaign_id, 16) &&
    hex(record.context.firmware_commit, 40) && hex(record.context.gate_commit, 40) && hex(record.context.app_elf_sha256, 64), "original_context_integrity");
  requireCondition(typeof record.context.firmware_root === "string", "original_repository_missing");
  (operations.ignored ?? ignored)(record.context.firmware_root, root);
  const budget = await protectedJson(input.budget_review_path);
  requireCondition(budget.context_sha256 === record.sha256, "original_review_context");
  requireExhaustedOriginal(budget.report);
  const state = await protectedJson(input.final_state_path);
  requireReleasedState(state, record.context);
  const original = {};
  for (const [key, path] of Object.entries(input)) original[key] = { path: resolve(path), sha256: await fileDigest(path) };
  const receipt = { schema: "worker-iterative-bootstrap-v1", original, original_campaign_id: record.context.campaign_id,
    next_ordinal: 1, total_charged_ms: 0, cleanup_confirmed: true };
  await mkdir(root, { mode: 0o700 });
  await writeNew(resolve(root, "bootstrap.json"), { receipt, sha256: digest(JSON.stringify(receipt)) });
  return { iterative_bootstrap_created: true, original_budget_changed: false, device_effects: false };
}
export async function readPrevious(path) {
  const record = await protectedJson(path);
  requireCondition(record.sha256 === digest(JSON.stringify(record.receipt)), "iterative_receipt_integrity");
  const receipt = record.receipt;
  requireCondition(["worker-iterative-bootstrap-v1", "worker-iterative-result-v1", "worker-iterative-result-v2"].includes(receipt.schema) && receipt.cleanup_confirmed === true,
    "iterative_previous_cleanup");
  if (receipt.schema === "worker-iterative-bootstrap-v1") {
    for (const source of Object.values(receipt.original)) {
      await protectedPath(source.path);
      requireCondition(await fileDigest(source.path) === source.sha256, "iterative_original_evidence_changed");
    }
    requireCondition(receipt.next_ordinal === 1 && receipt.total_charged_ms === 0, "iterative_bootstrap_ledger");
  } else {
    await validateCompletedReceipt(path, receipt, await resultSamples(dirname(path), receipt));
  }
  return receipt;
}
export async function validateCompletedReceipt(path, receipt, records) {
  requireCondition(receipt.cleanup_confirmed === true, "iterative_previous_cleanup");
    requireCondition(receipt.context_sha256 === digest(JSON.stringify(receipt.context)), "iterative_result_context");
    const frozen = await protectedJson(resolve(dirname(path), "context.json"));
    requireCondition(frozen.sha256 === receipt.context_sha256 && digest(JSON.stringify(frozen.context)) === frozen.sha256, "iterative_result_context");
    if (["fixed-usb-iterative-context-v4", RECOVERY_SCHEMA].includes(receipt.context.schema)) await validateIterativeContext(dirname(path), receipt.context, { historical: true });
    if (receipt.context.schema === RECOVERY_SCHEMA) requireCondition(isDeepStrictEqual(records.at(-1)?.state, receipt.final_state), "recovery_final_journal_binding");
    const earliest = records.find((record) => record.state.failure);
    if (earliest) {
      const evidence = receipt.first_failure_evidence;
      exactObject(evidence, ["details", "sha256"]);
      exactObject(evidence.details, ["schema", "ordinal", "sequence", "browser", "serial", "admission"]);
      requireCondition(evidence.details.schema === "worker-iterative-first-failure-v1" &&
        evidence.details.ordinal === receipt.context.qualification_attempt.ordinal, "iterative_first_failure_binding");
      requireCondition(evidence && evidence.details.browser === receipt.first_failure && evidence.details.sequence === earliest.sequence &&
        evidence.details.browser === earliest.state.failure && evidence.details.serial === (earliest.state.serialFailureCategory ?? null) &&
        evidence.details.admission === (earliest.state.admissionFailureStage ?? null), "iterative_first_failure_binding");
      const failurePath = resolve(dirname(path), "first-failure.json");
      await protectedPath(failurePath);
      requireCondition(await fileDigest(failurePath) === evidence.sha256 &&
        JSON.stringify(await readJson(failurePath)) === JSON.stringify(evidence.details), "iterative_first_failure_changed");
    } else requireCondition(receipt.first_failure === null && receipt.first_failure_evidence === null, "iterative_first_failure_binding");
    let fault;
    try { fault = await protectedJson(resolve(dirname(path), "iterative.fault.json")); }
    catch (error) { if (error.code !== "ENOENT") throw error; }
    const { judgeIterative } = await import("./iterative-judge.mjs");
    let judged, traceEvidence = [];
    try {
      traceEvidence = await requireRecoveryTraces(dirname(path), receipt.context, records, fault);
      judged = judgeIterative(receipt.context, records, fault, traceEvidence);
    }
    catch (error) {
      requireCondition(typeof error.code === "string" && receipt.result === "unverified" && receipt.judgment_failure === error.code, "iterative_result_judgment");
    }
    if (receipt.context.schema === RECOVERY_SCHEMA) requireCondition(
      JSON.stringify(receipt.recovery_trace_evidence) === JSON.stringify(traceEvidence), "recovery_trace_evidence_changed");
    if (judged) requireCondition(receipt.result === "passed" && JSON.stringify(judged) === JSON.stringify(receipt.judgment), "iterative_result_judgment");
    requireCondition(receipt.next_ordinal === receipt.ledger_after.next_ordinal && receipt.total_charged_ms === receipt.ledger_after.total_charged_ms &&
      receipt.original_campaign_id === receipt.context.original_campaign_id, "iterative_result_ledger_binding");
    requireReleasedState(receipt.final_state, receipt.context);
    requireExhaustedOriginal(receipt.original_budget);
    requireIdleLedger(receipt.ledger_after, receipt.context.qualification_attempt.ordinal + 1,
      receipt.ledger_before.total_charged_ms + receipt.context.qualification_attempt.maximumActiveMilliseconds);
}
/** Preserve the existing ledger chain while admitting a new loss or its separately authorized resume. */
export function validateRecoveryTransition(phase, previous, snapshot, progress) {
  requireCondition(["loss", "resume"].includes(phase), "recovery_phase");
  requireCondition(["worker-iterative-result-v1", "worker-iterative-result-v2"].includes(previous.schema) &&
    previous.cleanup_confirmed === true && previous.context && ["passed", "unverified"].includes(previous.result), "recovery_completed_predecessor");
  exactObject(progress, ["schema", "review", "reason", "evidence_sha256"]);
  requireCondition(progress.schema === "worker-qualification-progress-v1" && progress.review === "verified" &&
    Array.isArray(progress.evidence_sha256) && progress.evidence_sha256.length > 0 && progress.evidence_sha256.length <= 16 &&
    progress.evidence_sha256.every(value => hex(value, 64)), "iterative_progress_review");
  const same = ["firmware_commit", "gate_commit", "app_elf_sha256"].every(key => previous.context[key] === snapshot[key]);
  if (phase === "loss") {
    requireCondition(!same && progress.reason === "software_correction", "recovery_loss_progress");
    return;
  }
  requireCondition(same && previous.result === "passed" && previous.context.schema === RECOVERY_SCHEMA &&
    previous.context.recovery_phase === "loss" && previous.context.qualification_attempt.purpose === "diagnostic" &&
    progress.reason === "next_acceptance_window", "recovery_resume_progress");
  requireCondition(Number.isInteger(previous.judgment?.generation) && previous.judgment.generation > 0 &&
    previous.judgment.generation <= 0xffffffff, "recovery_loss_generation_missing");
}

/** Freeze one v5 allowance without creating a campaign or reserving any device budget. */
export async function recoveryPreflight(options, operations = {}) {
  requireCondition(["loss", "resume"].includes(options.recoveryPhase) && options.suggestedDifficulty === "1000" &&
    options.purpose === undefined && options.retainedRuntimeFrom === undefined && options.qualificationSourceCommit === undefined, "recovery_arguments");
  const root = resolve(options.privateRoot), parent = dirname(root);
  await protectedPath(parent, true); await missing(root);
  for (const key of ["firmwareRoot", "gateRoot", "authorityDirectory"]) options[key] = await canonicalDirectory(options[key]);
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const previousPath = resolve(options.previousReceipt);
  requireCondition(dirname(dirname(previousPath)) === parent && previousPath === resolve(dirname(previousPath), "result.json"), "recovery_existing_chain_required");
  const previous = await (operations.readPrevious ?? readPrevious)(previousPath);
  const progress = await protectedJson(options.input);
  await requireIterativeTask(options.firmwareRoot, true);
  const snapshot = await (operations.inspectSources ?? inspectSources)(options, operations);
  validateRecoveryTransition(options.recoveryPhase, previous, snapshot, progress);
  requireCondition(canonicalBase64(previous.original_campaign_id, 16) && Number.isSafeInteger(previous.total_charged_ms) &&
    previous.total_charged_ms >= 0 && Number.isSafeInteger(previous.total_charged_ms + 30000), "recovery_ledger_binding");
  const attempt = validateAttempt({ schema: "worker-qualification-attempt-v1", id: nonce(), ordinal: previous.next_ordinal,
    purpose: "diagnostic", maximumActiveMilliseconds: 30000 });
  requireCondition(options.recoveryPhase === "resume" ? options.cyclesFrom === undefined || resolve(options.cyclesFrom) === dirname(previousPath) :
    options.cyclesFrom === undefined, "recovery_cycle_source");
  const cycleSource = options.recoveryPhase === "resume" ? await reusableCycles(dirname(previousPath), snapshot, parent, operations) : undefined;
  const context = { schema: RECOVERY_SCHEMA, recovery_phase: options.recoveryPhase, owner_stack_minimum_bytes: 4096,
    suggested_difficulty: 1000, ...snapshot, qualification_attempt: attempt, required_no_mining_cycles: 4,
    ...(options.recoveryPhase === "resume" ? { recovery_loss_generation: previous.judgment.generation } : {}),
    ...(cycleSource ? { cycle_source: cycleSource.proof } : {}), original_campaign_id: previous.original_campaign_id,
    previous_receipt: previousPath, previous_receipt_sha256: await fileDigest(previousPath), progress_sha256: await fileDigest(options.input),
    progress_path: resolve(options.input), expected_charged_ms: previous.total_charged_ms, firmware_root: options.firmwareRoot,
    gate_root: options.gateRoot, manifest: resolve(options.manifest) };
  validateIterativePolicy(context, true);
  await missing(resolve(parent, `ordinal-${attempt.ordinal}.json`));
  await mkdir(root, { mode: 0o700 });
  if (cycleSource) for (const file of cycleSource.files) await writeFile(resolve(root, file.name), file.bytes, { flag: "wx", mode: 0o600 });
  await writeNew(resolve(parent, `ordinal-${attempt.ordinal}.json`), { context_sha256: digest(JSON.stringify(context)), attempt_root: root });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  return { recovery_preflight_created: true, phase: options.recoveryPhase, ordinal: attempt.ordinal, maximum_active_ms: 30000,
    device_effects: false, allowance_reserved_on_device: false };
}

export async function iterativePreflight(options, operations = {}) {
  requireCondition(options.suggestedDifficulty === "1000", "iterative_hint_policy");
  const root = resolve(options.privateRoot), parent = dirname(root);
  await protectedPath(parent, true); await missing(root);
  for (const key of ["firmwareRoot", "gateRoot", "authorityDirectory"]) options[key] = await canonicalDirectory(options[key]);
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const previousPath = resolve(options.previousReceipt);
  requireCondition(previousPath === resolve(parent, "bootstrap.json") || dirname(dirname(previousPath)) === parent, "iterative_parent_receipt");
  const previous = await (operations.readPrevious ?? readPrevious)(previousPath);
  const progress = await protectedJson(options.input);
  exactObject(progress, ["schema", "review", "reason", "evidence_sha256"]);
  requireCondition(progress.schema === "worker-qualification-progress-v1" && progress.review === "verified" &&
    ["software_correction", "diagnostic_pass", "next_acceptance_window"].includes(progress.reason) &&
    Array.isArray(progress.evidence_sha256) && progress.evidence_sha256.length > 0 && progress.evidence_sha256.length <= 16 &&
    progress.evidence_sha256.every((value) => hex(value, 64)), "iterative_progress_review");
  requireCondition(PURPOSES.includes(options.purpose), "iterative_purpose");
  await requireIterativeTask(options.firmwareRoot);
  const retained = options.retainedRuntimeFrom !== undefined || options.qualificationSourceCommit !== undefined;
  let retainedSource;
  if (retained) {
    requireCondition(options.retainedRuntimeFrom && options.qualificationSourceCommit && previous.result === "passed" &&
      previous.context?.schema === "fixed-usb-iterative-context-v4" && previous.context.unreserved_continuation &&
      previous.context.qualification_attempt.purpose === "foreground_loss" && options.purpose === "heartbeat_loss" &&
      resolve(options.retainedRuntimeFrom) === dirname(previousPath) &&
      options.cyclesFrom && resolve(options.cyclesFrom) === resolve(options.retainedRuntimeFrom) &&
      options.qualificationSourceCommit === previous.context.qualification_driver.source_commit, "iterative_retained_successor");
    retainedSource = await (operations.inspectRetainedSources ?? inspectRetainedSources)(resolve(options.retainedRuntimeFrom), options, operations);
  }
  const snapshot = retainedSource?.snapshot ?? await (operations.inspectSources ?? inspectSources)(options, operations);
  const ordinal = previous.next_ordinal;
  requireCondition(Number.isSafeInteger(previous.total_charged_ms + maximumActiveMs(options.purpose)), "iterative_ledger_exhausted");
  const attempt = validateAttempt({ schema: "worker-qualification-attempt-v1", id: nonce(), ordinal,
    purpose: options.purpose, maximumActiveMilliseconds: maximumActiveMs(options.purpose) });
  if (previous.schema !== "worker-iterative-bootstrap-v1") {
    const prior = previous.context;
    if (prior.schema === "fixed-usb-iterative-context-v1") requireCondition(options.purpose === "diagnostic", "iterative_policy_upgrade_required");
    const same = ["firmware_commit", "gate_commit", "app_elf_sha256"].every((key) => prior[key] === snapshot[key]);
    if (previous.result === "passed") {
      const nextPurpose = { diagnostic: "normal", normal: "foreground_loss", foreground_loss: "heartbeat_loss" }[prior.qualification_attempt.purpose];
      requireCondition(same ? options.purpose === nextPurpose && progress.reason === (prior.qualification_attempt.purpose === "diagnostic" ? "diagnostic_pass" : "next_acceptance_window") :
        options.purpose === "diagnostic" && progress.reason === "software_correction", "iterative_next_progress");
    }
    if (previous.result !== "passed") requireCondition(options.purpose === "diagnostic" && !same && progress.reason === "software_correction", "iterative_retry_progress");
    if (["foreground_loss", "heartbeat_loss"].includes(options.purpose)) requireCondition(same && previous.result === "passed" &&
      prior.qualification_attempt.purpose === (options.purpose === "foreground_loss" ? "normal" : "foreground_loss"), "iterative_final_sequence");
  } else requireCondition(options.purpose === "diagnostic", "iterative_initial_purpose");
  const cycleSource = options.cyclesFrom ? await reusableCycles(resolve(options.cyclesFrom), snapshot, parent, operations) : undefined;
  const context = { schema: retained ? "fixed-usb-iterative-context-v4" : "fixed-usb-iterative-context-v3", owner_stack_minimum_bytes: 4096, suggested_difficulty: 1000, ...snapshot, qualification_attempt: attempt,
    ...(retained ? { qualification_driver: structuredClone(previous.context.qualification_driver), retained_runtime_source: {
      root: resolve(options.retainedRuntimeFrom), context_sha256: retainedSource.sourceContextSha256, artifact_snapshot_sha256: retainedSource.artifactSnapshotSha256 } } : {}),
    required_no_mining_cycles: 4, ...(cycleSource ? { cycle_source: cycleSource.proof } : {}), original_campaign_id: previous.original_campaign_id,
    previous_receipt: previousPath, previous_receipt_sha256: await fileDigest(previousPath),
    progress_sha256: await fileDigest(options.input), progress_path: resolve(options.input),
    expected_charged_ms: previous.total_charged_ms, firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: retained ? retainedManifestPath(root) : resolve(options.manifest) };
  await mkdir(root, { mode: 0o700 });
  if (cycleSource) for (const file of cycleSource.files) await writeFile(resolve(root, file.name), file.bytes, { flag: "wx", mode: 0o600 });
  await writeNew(resolve(parent, `ordinal-${ordinal}.json`), { context_sha256: digest(JSON.stringify(context)), attempt_root: root });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  if (retained) await (operations.copyRetainedArtifacts ?? copyRetainedArtifacts)(resolve(options.retainedRuntimeFrom), root, context);
  return { iterative_preflight_created: true, ordinal, purpose: attempt.purpose, maximum_active_ms: attempt.maximumActiveMilliseconds,
    device_effects: false, allowance_reserved_on_device: false };
}
export async function validateIterativeContext(root, context, { historical = false } = {}) {
  validateIterativePolicy(context);
  if (!historical) await requireIterativeTask(context.firmware_root, context.schema === RECOVERY_SCHEMA);
  validateAttempt(context.qualification_attempt);
  const parent = dirname(root), previousPath = resolve(context.previous_receipt);
  requireCondition(previousPath === resolve(parent, "bootstrap.json") || dirname(dirname(previousPath)) === parent, "iterative_parent_receipt");
  requireCondition(await fileDigest(previousPath) === context.previous_receipt_sha256, "iterative_previous_changed");
  const previous = await readPrevious(context.previous_receipt);
  requireCondition(previous.next_ordinal === context.qualification_attempt.ordinal && previous.total_charged_ms === context.expected_charged_ms &&
    previous.original_campaign_id === context.original_campaign_id, "iterative_previous_changed");
  await protectedPath(context.progress_path);
  requireCondition(await fileDigest(context.progress_path) === context.progress_sha256, "iterative_progress_changed");
  if (context.schema === RECOVERY_SCHEMA) {
    requireCondition(previousPath === resolve(dirname(previousPath), "result.json"), "recovery_existing_chain_required");
    validateRecoveryTransition(context.recovery_phase, previous, context, await protectedJson(context.progress_path));
    requireCondition(context.recovery_phase === "loss" ? context.recovery_loss_generation === undefined :
      context.recovery_loss_generation === previous.judgment.generation, "recovery_generation_lineage");
    requireCondition(context.recovery_phase === "loss" ? context.cycle_source === undefined :
      context.cycle_source?.root === dirname(previousPath), "recovery_cycle_source");
  }
  if (context.cycle_source) {
    const source = await reusableCycles(context.cycle_source.root, context, dirname(root));
    requireCondition(JSON.stringify(source.proof) === JSON.stringify(context.cycle_source), "iterative_cycle_source_changed");
    for (const file of source.proof.receipts) {
      await protectedPath(resolve(root, file.name));
      requireCondition(await fileDigest(resolve(root, file.name)) === file.sha256, "iterative_reused_cycle_changed");
    }
  }
  if (context.unreserved_continuation) {
    requireCondition(context.schema === "fixed-usb-iterative-context-v4" && context.retained_runtime_source === undefined, "iterative_continuation_profile");
    const { validateUnreservedContinuation } = await import("./unreserved.mjs");
    await validateUnreservedContinuation(root, context);
  } else {
    const marker = await protectedJson(resolve(dirname(root), `ordinal-${context.qualification_attempt.ordinal}.json`));
    requireCondition(marker.attempt_root === root && marker.context_sha256 === digest(JSON.stringify(context)), "iterative_ordinal_replay");
    if (context.schema === "fixed-usb-iterative-context-v4") {
      exactObject(context.retained_runtime_source, ["root", "context_sha256", "artifact_snapshot_sha256"]);
      const source = context.retained_runtime_source;
      requireCondition(source.root === dirname(context.previous_receipt) && previous.result === "passed" &&
        previous.context.schema === "fixed-usb-iterative-context-v4" && previous.context.unreserved_continuation &&
        previous.context.qualification_attempt.purpose === "foreground_loss" && context.qualification_attempt.purpose === "heartbeat_loss" &&
        JSON.stringify(context.qualification_driver) === JSON.stringify(previous.context.qualification_driver) &&
        source.context_sha256 === previous.context_sha256 && source.artifact_snapshot_sha256 === await fileDigest(resolve(source.root, "artifact-snapshot.json")), "iterative_retained_lineage");
    }
  }
}

async function reusableCycles(root, context, parent, operations = {}) {
  requireCondition(dirname(root) === parent, "iterative_cycle_source_parent");
  const result = await (operations.readPrevious ?? readPrevious)(resolve(root, "result.json"));
  requireCondition(["worker-iterative-result-v1", "worker-iterative-result-v2"].includes(result.schema) &&
    ["firmware_commit", "gate_commit", "app_elf_sha256"].every((key) => result.context[key] === context[key]), "iterative_cycle_runtime_changed");
  const record = await protectedJson(resolve(root, "context.json"));
  requireCondition(record.sha256 === digest(JSON.stringify(record.context)) && record.sha256 === result.context_sha256, "iterative_cycle_context");
  let previous; const files = [], receipts = [];
  for (let cycle = 1; cycle <= 4; cycle += 1) {
    const name = `cycle-${cycle}.json`, path = resolve(root, name);
    await protectedPath(path); const bytes = await readFile(path);
    previous = validateCycle(JSON.parse(bytes), context, previous);
    files.push({ name, bytes }); receipts.push({ name, sha256: digest(bytes) });
  }
  return { proof: { root, context_sha256: record.sha256, receipts }, files };
}
