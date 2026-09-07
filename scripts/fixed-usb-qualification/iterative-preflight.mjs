import { resultSamples } from "./sample-seal.mjs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { canonicalBase64, canonicalDirectory, digest, exactObject, fileDigest, hex, ignored, missing, nonce, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { inspectSources } from "./preflight.mjs";
import { validateCycle, validateState } from "./judge.mjs";
import { maximumActiveMs, PURPOSES, requireExhaustedOriginal, requireIdleLedger, validateAttempt } from "./iterative-contract.mjs";

export function validateIterativePolicy(context, live = false) {
  const v2 = context.schema === "fixed-usb-iterative-context-v2";
  requireCondition(v2 ? context.owner_stack_minimum_bytes === 4096 :
    context.schema === "fixed-usb-iterative-context-v1" && context.owner_stack_minimum_bytes === undefined, "iterative_policy");
  requireCondition(!live || v2, "iterative_policy_upgrade_required");
  return v2;
}
export async function requireIterativeTask(firmwareRoot) {
  const tasks = await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8");
  let active = false, count = 0, total = 0;
  for (const line of tasks.split(/\r?\n/u)) {
    if (line.startsWith("## ")) active = line === "## Active";
    if (line.startsWith("### ") && line.slice(4).split(/\s/u)[0] === "task-worker-preparation-panic-qualification") {
      total += 1;
      if (active) count += 1;
    }
  }
  requireCondition(count === 1 && total === 1, "iterative_active_task_required");
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
    let judged;
    try { judged = judgeIterative(receipt.context, records, fault); }
    catch (error) {
      requireCondition(typeof error.code === "string" && receipt.result === "unverified" && receipt.judgment_failure === error.code, "iterative_result_judgment");
    }
    if (judged) requireCondition(receipt.result === "passed" && JSON.stringify(judged) === JSON.stringify(receipt.judgment), "iterative_result_judgment");
    requireCondition(receipt.next_ordinal === receipt.ledger_after.next_ordinal && receipt.total_charged_ms === receipt.ledger_after.total_charged_ms &&
      receipt.original_campaign_id === receipt.context.original_campaign_id, "iterative_result_ledger_binding");
    requireReleasedState(receipt.final_state, receipt.context);
    requireExhaustedOriginal(receipt.original_budget);
    requireIdleLedger(receipt.ledger_after, receipt.context.qualification_attempt.ordinal + 1,
      receipt.ledger_before.total_charged_ms + receipt.context.qualification_attempt.maximumActiveMilliseconds);
}
export async function iterativePreflight(options, operations = {}) {
  const root = resolve(options.privateRoot), parent = dirname(root);
  await protectedPath(parent, true); await missing(root);
  for (const key of ["firmwareRoot", "gateRoot", "authorityDirectory"]) options[key] = await canonicalDirectory(options[key]);
  (operations.ignored ?? ignored)(options.firmwareRoot, root);
  const previousPath = resolve(options.previousReceipt);
  requireCondition(previousPath === resolve(parent, "bootstrap.json") || dirname(dirname(previousPath)) === parent, "iterative_parent_receipt");
  const previous = await readPrevious(previousPath);
  const progress = await protectedJson(options.input);
  exactObject(progress, ["schema", "review", "reason", "evidence_sha256"]);
  requireCondition(progress.schema === "worker-qualification-progress-v1" && progress.review === "verified" &&
    ["software_correction", "diagnostic_pass", "next_acceptance_window"].includes(progress.reason) &&
    Array.isArray(progress.evidence_sha256) && progress.evidence_sha256.length > 0 && progress.evidence_sha256.length <= 16 &&
    progress.evidence_sha256.every((value) => hex(value, 64)), "iterative_progress_review");
  requireCondition(PURPOSES.includes(options.purpose), "iterative_purpose");
  await requireIterativeTask(options.firmwareRoot);
  const snapshot = await (operations.inspectSources ?? inspectSources)(options, operations);
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
  const cycleSource = options.cyclesFrom ? await reusableCycles(resolve(options.cyclesFrom), snapshot, parent) : undefined;
  const context = { schema: "fixed-usb-iterative-context-v2", owner_stack_minimum_bytes: 4096, ...snapshot, qualification_attempt: attempt,
    required_no_mining_cycles: 4, ...(cycleSource ? { cycle_source: cycleSource.proof } : {}), original_campaign_id: previous.original_campaign_id,
    previous_receipt: previousPath, previous_receipt_sha256: await fileDigest(previousPath),
    progress_sha256: await fileDigest(options.input), progress_path: resolve(options.input),
    expected_charged_ms: previous.total_charged_ms, firmware_root: options.firmwareRoot, gate_root: options.gateRoot, manifest: resolve(options.manifest) };
  await mkdir(root, { mode: 0o700 });
  if (cycleSource) for (const file of cycleSource.files) await writeFile(resolve(root, file.name), file.bytes, { flag: "wx", mode: 0o600 });
  await writeNew(resolve(parent, `ordinal-${ordinal}.json`), { context_sha256: digest(JSON.stringify(context)), attempt_root: root });
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  return { iterative_preflight_created: true, ordinal, purpose: attempt.purpose, maximum_active_ms: attempt.maximumActiveMilliseconds,
    device_effects: false, allowance_reserved_on_device: false };
}
export async function validateIterativeContext(root, context) {
  validateIterativePolicy(context);
  await requireIterativeTask(context.firmware_root);
  validateAttempt(context.qualification_attempt);
  const previous = await readPrevious(context.previous_receipt);
  requireCondition(await fileDigest(context.previous_receipt) === context.previous_receipt_sha256 &&
    previous.next_ordinal === context.qualification_attempt.ordinal && previous.total_charged_ms === context.expected_charged_ms &&
    previous.original_campaign_id === context.original_campaign_id, "iterative_previous_changed");
  await protectedPath(context.progress_path);
  requireCondition(await fileDigest(context.progress_path) === context.progress_sha256, "iterative_progress_changed");
  if (context.cycle_source) {
    const source = await reusableCycles(context.cycle_source.root, context, dirname(root));
    requireCondition(JSON.stringify(source.proof) === JSON.stringify(context.cycle_source), "iterative_cycle_source_changed");
    for (const file of source.proof.receipts) {
      await protectedPath(resolve(root, file.name));
      requireCondition(await fileDigest(resolve(root, file.name)) === file.sha256, "iterative_reused_cycle_changed");
    }
  }
  const marker = await protectedJson(resolve(dirname(root), `ordinal-${context.qualification_attempt.ordinal}.json`));
  requireCondition(marker.attempt_root === root && marker.context_sha256 === digest(JSON.stringify(context)), "iterative_ordinal_replay");
}

async function reusableCycles(root, context, parent) {
  requireCondition(dirname(root) === parent, "iterative_cycle_source_parent");
  const result = await readPrevious(resolve(root, "result.json"));
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
