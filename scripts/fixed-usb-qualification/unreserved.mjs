import { lstat, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { digest, exactObject, fileDigest, hex, missing, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { parseSamples } from "./sample-seal.mjs";
import { validateCycle, validateState } from "./judge.mjs";
import { requireExhaustedOriginal, requireIdleLedger, validateAttempt } from "./iterative-contract.mjs";
import { readPrevious, requireReleasedState, validateIterativeContext } from "./iterative-preflight.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { validateDiagnosticExport } from "./diagnostic-export.mjs";

const ESSENTIAL = ["context.json", "issued.json", "consumed.json", "first-failure.json", "iterative.samples.jsonl", "artifact-snapshot.json", "cooling.json"];
const ABSENT = ["result.json", "iterative.fault.json", "sealed.samples.jsonl", "sample-seal-intent.json", "sample-seal-recovery.json", "post-seal-conflict.json"];
async function readProtected(path) {
  await protectedPath(path);
  requireCondition((await lstat(path)).size <= 33554432, "unreserved_input_bound");
  return readJson(path);
}
async function cycles(root, context) {
  let previous; const files = [];
  for (let cycle = 1; cycle <= 4; cycle += 1) {
    const name = `cycle-${cycle}.json`, path = resolve(root, name);
    await protectedPath(path); const bytes = await readFile(path);
    previous = validateCycle(JSON.parse(bytes), context, previous);
    files.push({ name, bytes, sha256: digest(bytes) });
  }
  return { files, baseline_id: previous.baseline_id };
}
async function inspectOrigin(origin, inputPath, operations = {}, historical = false) {
  await protectedPath(origin, true);
  const record = await readProtected(resolve(origin, "context.json")), context = record.context;
  requireCondition(record.sha256 === digest(JSON.stringify(context)) && context.schema === "fixed-usb-iterative-context-v3" &&
    context.unreserved_continuation === undefined && context.qualification_driver === undefined, "unreserved_origin_context");
  validateAttempt(context.qualification_attempt);
  await (operations.validateContext ?? validateIterativeContext)(origin, context, { historical });
  for (const name of ABSENT) await missing(resolve(origin, name));
  const issuance = await readProtected(resolve(origin, "issued.json")), consumption = await readProtected(resolve(origin, "consumed.json"));
  const a = context.qualification_attempt;
  requireCondition(a.purpose === "foreground_loss" && issuance.schema === "worker-iterative-issuance-v1" &&
    issuance.context_sha256 === record.sha256 && issuance.ordinal === a.ordinal && issuance.private_payload_persisted === false &&
    consumption.ordinal === a.ordinal && consumption.delivery_attempted === true, "unreserved_issuance");
  requireIdleLedger(issuance.ledger_before, a.ordinal, context.expected_charged_ms);
  const input = await readProtected(inputPath);
  exactObject(input, ["ledger_after", "final_state", "diagnostic_export_path", "cleanup_evidence_path", "progress_evidence_sha256"]);
  requireCondition(Array.isArray(input.progress_evidence_sha256) && input.progress_evidence_sha256.length >= 1 &&
    input.progress_evidence_sha256.length <= 16 && input.progress_evidence_sha256.every((value) => hex(value, 64)), "unreserved_progress_evidence");
  requireIdleLedger(input.ledger_after, a.ordinal, context.expected_charged_ms);
  requireCondition(isDeepStrictEqual(input.ledger_after, issuance.ledger_before), "unreserved_ledger_changed");
  requireReleasedState(input.final_state, context);
  const prior = await (operations.readPrevious ?? readPrevious)(context.previous_receipt);
  requireExhaustedOriginal(prior.original_budget);
  requireCondition(prior.result === "passed" && prior.context.qualification_attempt.purpose === "normal" &&
    prior.context.qualification_attempt.ordinal + 1 === a.ordinal &&
    ["firmware_commit", "gate_commit", "app_elf_sha256", "suggested_difficulty"].every((key) => prior.context[key] === context[key]), "unreserved_previous_normal");
  const q = input.final_state.qualification;
  requireCondition(q?.attempt?.ordinal === a.ordinal - 1 && q.attempt.purpose === "normal" && q.attempt.maximum_active_ms === 180000 &&
    q.generation === prior.final_state.qualification?.generation && q.attempt.complete === true && q.safe_stop_complete === true &&
    q.gate_closed_ms !== null && q.shutdown_started_ms !== null && q.mine_on_boot === false, "unreserved_retired_observation");
  const journalPath = resolve(origin, "iterative.samples.jsonl"); await protectedPath(journalPath);
  const records = parseSamples(await readFile(journalPath), context);
  requireCondition(records.every((entry) => !entry.state.running && entry.state.qualification?.attempt?.ordinal !== a.ordinal), "unreserved_work_observed");
  requireCondition(isDeepStrictEqual(records.at(-1).state, input.final_state), "unreserved_final_state_changed");
  const earliest = records.find((entry) => entry.state.failure);
  const failure = await readProtected(resolve(origin, "first-failure.json"));
  requireCondition(earliest && earliest.state.failure === "start_failed" && earliest.state.serialFailureCategory === "command_rejected" &&
    isDeepStrictEqual(failure, { schema: "worker-iterative-first-failure-v1", ordinal: a.ordinal, sequence: earliest.sequence,
      browser: "start_failed", serial: "command_rejected", admission: earliest.state.admissionFailureStage ?? null }), "unreserved_first_failure");
  const diagnostic = await readProtected(input.diagnostic_export_path);
  exactObject(diagnostic, ["schema", "context_sha256", "export", "hardware_authority"]);
  requireCondition(diagnostic.schema === "fixed-usb-diagnostic-export-v1" && diagnostic.context_sha256 === record.sha256 && diagnostic.hardware_authority === false, "unreserved_diagnostic_context");
  exactObject(diagnostic.export, ["schema", "observations"]);
  requireCondition(diagnostic.export.schema === "worker-diagnostic-export-v1" && Array.isArray(diagnostic.export.observations) &&
    diagnostic.export.observations.length <= 40, "unreserved_diagnostic_shape");
  // History uses the sealed bytes already validated at creation, never a later Gate checkout.
  const validated = historical ? diagnostic.export :
    await (operations.validateDiagnostics ?? ((value) => validateDiagnosticExport(value, context.gate_root)))(diagnostic.export);
  requireCondition(validated.observations.some((value) => isDeepStrictEqual(value,
    { category: "control_failure", authoritative: false, error: "admission_required" })), "unreserved_admission_required_missing");
  const cleanup = await readProtected(input.cleanup_evidence_path);
  const flags = ["browser_connection_closed", "serial_nodes_released", "supervisor_reaped", "listener_released", "restoration_confirmed"];
  exactObject(cleanup, ["schema", "context_sha256", ...flags, "source_observation_sha256"]);
  const observationPath = resolve(origin, "unreserved-recovery-observation.json");
  const observation = await readProtected(observationPath);
  exactObject(observation, [...flags, "next_ordinal", "total_charged_ms", "pending", "last_completed_ordinal", "fault_executed"]);
  requireCondition(cleanup.schema === "worker-unreserved-cleanup-v1" && cleanup.context_sha256 === record.sha256 &&
    flags.every((key) => cleanup[key] === true && observation[key] === true) && observation.fault_executed === false &&
    cleanup.source_observation_sha256 === await fileDigest(observationPath), "unreserved_cleanup_evidence");
  const observedLedger = { schema: "worker-qualification-ledger-v1", next_ordinal: observation.next_ordinal,
    total_charged_ms: observation.total_charged_ms, pending: observation.pending, last_completed_ordinal: observation.last_completed_ordinal };
  requireCondition(isDeepStrictEqual(observedLedger, input.ledger_after), "unreserved_observed_ledger_changed");
  const cycleProof = await cycles(origin, context);
  requireCondition(cycleProof.baseline_id === input.final_state.preservation.baseline_id, "unreserved_cycle_continuity");
  await (operations.verifySnapshot ?? verifyArtifactSnapshot)(origin, context);
  const paths = [...ESSENTIAL.map((name) => resolve(origin, name)), ...cycleProof.files.map(({ name }) => resolve(origin, name)),
    observationPath, resolve(inputPath), resolve(input.diagnostic_export_path), resolve(input.cleanup_evidence_path), context.previous_receipt, context.progress_path];
  const evidence = [];
  for (const path of [...new Set(paths)]) { await protectedPath(path); evidence.push({ path, sha256: await fileDigest(path) }); }
  return { context, input, cycleProof, evidence };
}
export async function createUnreservedContinuation(options, operations = {}) {
  const origin = resolve(options.originRoot), root = resolve(options.privateRoot), inputPath = resolve(options.input);
  requireCondition(root !== origin && dirname(root) === dirname(origin), "unreserved_sibling_required");
  await missing(root); await missing(resolve(origin, "unreserved-continuation.json"));
  const inspected = await inspectOrigin(origin, inputPath, operations);
  const helper = operations.inspectSources && operations.copyArtifacts ? {} : await import("./runtime-source.mjs");
  const inspectedSources = await (operations.inspectSources ?? helper.inspectRetainedSources)(origin, {
    ...options, firmwareRoot: inspected.context.firmware_root, gateRoot: inspected.context.gate_root,
    firmwareCommit: inspected.context.firmware_commit, gateCommit: inspected.context.gate_commit,
    manifest: resolve(origin, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"),
  }, operations);
  requireCondition(isDeepStrictEqual(inspectedSources.sourceContext, inspected.context), "unreserved_retained_origin");
  requireCondition(hex(options.qualificationSourceCommit, 40), "unreserved_driver_commit");
  const rejection = { schema: "worker-unreserved-rejection-v1", status: "unverified_not_reserved", origin_root: origin,
    context_sha256: digest(JSON.stringify(inspected.context)), input_path: inputPath, evidence: inspected.evidence,
    ledger_before: inspected.input.ledger_after, ledger_after: inspected.input.ledger_after, charge_added_ms: 0, refund_ms: 0 };
  const rejectionPath = resolve(origin, "unreserved-rejection.json");
  const rejectionHash = digest(`${JSON.stringify(rejection, null, 2)}\n`);
  const context = { ...inspected.context, ...inspectedSources.snapshot, schema: "fixed-usb-iterative-context-v4",
    manifest: resolve(root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"),
    qualification_driver: { profile: "fixed-usb-retained-runtime-driver-v1", source_commit: options.qualificationSourceCommit },
    unreserved_continuation: { origin_root: origin, rejection_sha256: rejectionHash } };
  const expected = { ...inspected.context, schema: "fixed-usb-iterative-context-v4", manifest: context.manifest,
    qualification_driver: context.qualification_driver, unreserved_continuation: context.unreserved_continuation };
  requireCondition(isDeepStrictEqual(context, expected), "unreserved_context_changed");
  await writeNew(rejectionPath, rejection);
  const marker = { schema: "worker-unreserved-continuation-v1", continuation_root: root, context_sha256: digest(JSON.stringify(context)),
    rejection_sha256: context.unreserved_continuation.rejection_sha256 };
  await writeNew(resolve(origin, "unreserved-continuation.json"), marker);
  await mkdir(root, { mode: 0o700 });
  for (const file of inspected.cycleProof.files) await writeFile(resolve(root, file.name), file.bytes, { flag: "wx", mode: 0o600 });
  await (operations.copyArtifacts ?? helper.copyRetainedArtifacts)(origin, root, context);
  await writeNew(resolve(root, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  return { continuation_created: true, ordinal: context.qualification_attempt.ordinal, status: rejection.status, charge_added_ms: 0, refund_ms: 0, device_effects: false };
}
export async function validateUnreservedContinuation(root, context, operations = {}) {
  exactObject(context.unreserved_continuation, ["origin_root", "rejection_sha256"]);
  const origin = context.unreserved_continuation.origin_root;
  requireCondition(context.schema === "fixed-usb-iterative-context-v4" && root !== origin && dirname(root) === dirname(origin), "unreserved_sibling_required");
  const rejectionPath = resolve(origin, "unreserved-rejection.json"), rejection = await readProtected(rejectionPath);
  exactObject(rejection, ["schema", "status", "origin_root", "context_sha256", "input_path", "evidence", "ledger_before", "ledger_after", "charge_added_ms", "refund_ms"]);
  requireCondition(await fileDigest(rejectionPath) === context.unreserved_continuation.rejection_sha256 &&
    rejection.schema === "worker-unreserved-rejection-v1" && rejection.status === "unverified_not_reserved" && rejection.charge_added_ms === 0 && rejection.refund_ms === 0, "unreserved_rejection_changed");
  requireCondition(Array.isArray(rejection.evidence) && rejection.evidence.length > 0 && rejection.evidence.length <= 32, "unreserved_evidence_shape");
  const seen = new Set();
  for (const file of rejection.evidence) {
    exactObject(file, ["path", "sha256"]);
    requireCondition(typeof file.path === "string" && !seen.has(file.path) && hex(file.sha256, 64), "unreserved_evidence_shape");
    seen.add(file.path);
    await protectedPath(file.path);
    requireCondition(await fileDigest(file.path) === file.sha256, "unreserved_evidence_changed");
  }
  exactObject(context.qualification_driver, ["profile", "source_commit"]);
  requireCondition(context.qualification_driver.profile === "fixed-usb-retained-runtime-driver-v1" && hex(context.qualification_driver.source_commit, 40), "unreserved_driver_commit");
  const inspected = await inspectOrigin(origin, rejection.input_path, operations, true);
  requireCondition(isDeepStrictEqual(inspected.evidence, rejection.evidence) && rejection.origin_root === origin &&
    rejection.context_sha256 === digest(JSON.stringify(inspected.context)) && isDeepStrictEqual(rejection.ledger_before, inspected.input.ledger_after) &&
    isDeepStrictEqual(rejection.ledger_after, inspected.input.ledger_after), "unreserved_evidence_changed");
  const expected = { ...inspected.context, schema: "fixed-usb-iterative-context-v4", manifest: context.manifest,
    qualification_driver: context.qualification_driver, unreserved_continuation: context.unreserved_continuation };
  requireCondition(isDeepStrictEqual(context, expected) && context.manifest === resolve(root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"), "unreserved_context_changed");
  const marker = await readProtected(resolve(origin, "unreserved-continuation.json"));
  requireCondition(isDeepStrictEqual(marker, { schema: "worker-unreserved-continuation-v1", continuation_root: root,
    context_sha256: digest(JSON.stringify(context)), rejection_sha256: context.unreserved_continuation.rejection_sha256 }), "unreserved_exclusive_marker");
  const copied = await cycles(root, context);
  requireCondition(isDeepStrictEqual(copied.files.map(({ name, sha256 }) => ({ name, sha256 })), inspected.cycleProof.files.map(({ name, sha256 }) => ({ name, sha256 }))), "unreserved_copies_changed");
  await (operations.verifySnapshot ?? verifyArtifactSnapshot)(root, context);
}
