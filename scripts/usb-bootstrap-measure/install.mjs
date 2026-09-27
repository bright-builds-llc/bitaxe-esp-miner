import { observeCaptureExit, observeDetectorExit } from "./capture-exit.mjs";
import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { readFreshDetector, validateCommandObservation } from "../str005-noise-serial/install.mjs";
import { flashArguments } from "../str005-noise-serial/operator-execution.mjs";
import { observeCommand } from "../str005-noise-serial/operator.mjs";
import { processSnapshot, requireGone, requireNoHolders, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { load, contextHash, legacyView } from "./context.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { fail } from "./failure.mjs";
import { CONTEXT_V3, check, object, schema, sha256 } from "./values.mjs";
export const argumentsFor = (root, context, port) => [...flashArguments(root, legacyView(context), 0, port), "--capture-bootstrap-timing"];
async function released(root, context) {
  const last = (await readJournal(root, context)).at(-1); baseline(last?.state, true);
  const before = (await proof(root, "accounting-before.json")).value;
  check(last.phase === "before" && last.sequence > before.observedSequence && last.state.preservation.baseline_id === before.state.preservation.baseline_id,
    "bootstrap_install_baseline"); return last;
}
export async function claim(root, context, input, operations = {}) {
  object(input, ["index"]); check(input.index === 0, "bootstrap_install_index"); await missing(resolve(root, "failure.json")); await missing(resolve(root, "install-0"));
  const last = await released(root, context), detector = await readFreshDetector(root, 0, operations);
  const previous = (await proof(context.predecessor.root, context.schema === CONTEXT_V3 ? "install-0.claim.json" : "install-4.claim.json")).value;
  check(detector.physical === previous.detector.physical, "bootstrap_physical_changed");
  const owner = await proof(root, "install-0.host-root.json"), armed = await proof(root, "install-0.observer-armed.json");
  check(sameProcess(owner.value, armed.value) && (await (operations.processSnapshot ?? processSnapshot)()).some(row => sameProcess(row, owner.value)), "bootstrap_install_owner");
  const value = { schema: schema("install-claim"), contextSha256: contextHash(context), index: 0, beforeSequence: last.sequence, beforeStateSha256: sha256(canonical(last)),
    atUnixMs: Date.now(), detector, ownerSha256: owner.sha256, armedSha256: armed.sha256, argv: argumentsFor(root, context, detector.port) };
  await writeNew(resolve(root, "install-0.claim.json"), value);
  return { install_claimed: true, index: 0, program: "just", context_sha256: contextHash(context), claim_sha256: (await proof(root, "install-0.claim.json")).sha256 };
}
export async function admit(root, mode, permit, operations = {}) {
  object(permit, ["kind", "contextSha256", "claimSha256"]); check(permit.kind === "execute" && ["detect", "flash"].includes(mode), "bootstrap_execute");
  const context = await load(root, { operations }); check(contextHash(context) === permit.contextSha256, "bootstrap_execute_context");
  await missing(resolve(root, "failure.json")); await released(root, context); await requireSupervisor(root, context, operations);
  if (mode === "detect") { check(permit.claimSha256 === null, "bootstrap_execute_claim"); return { context, argv: ["detect-ultra205"] }; }
  const p = await proof(root, "install-0.claim.json"), value = p.value, owner = await proof(root, "install-0.host-root.json");
  check(p.sha256 === permit.claimSha256 && value.contextSha256 === permit.contextSha256 && owner.sha256 === value.ownerSha256 &&
    owner.value.pid === (operations.pid ?? process.pid) && canonical(value.argv) === canonical(argumentsFor(root, context, value.detector.port)), "bootstrap_execute_claim");
  const fresh = await readFreshDetector(root, 0, operations); check(canonical(fresh) === canonical(value.detector), "bootstrap_detector_changed");
  requireNoHolders(fresh.port, operations); await missing(resolve(root, "install-0"));
  await requireSupervisor(root, context, operations);
  const observation = (await proof(root, "install-0.detect.observation.json")).value;
  check(Date.now() >= observation.finished_at_unix_ms && Date.now() - observation.finished_at_unix_ms <= 60000, "bootstrap_detector_stale");
  return { context, argv: value.argv };
}
export async function requireSupervisor(root, context, operations = {}) {
  const server = (await proof(root, "server-owner.json")).value;
  check(server.contextSha256 === contextHash(context) && server.origin === `http://127.0.0.1:${server.port}`, "bootstrap_supervisor");
  const rows = await (operations.processSnapshot ?? processSnapshot)();
  check(rows.some(row => sameProcess(row, server.owner) && !/[ZT]/u.test(row.state ?? "")), "bootstrap_supervisor");
}
/** Actual nonzero capture exit stays recorded; only completed exact-image writes allow recovery. */
export async function inspectCapture(root, context) {
  const hash = contextHash(context), c = await proof(root, "install-0.claim.json"), e = await proof(root, "install-0.exit.json"), o = await proof(root, "install-0.observation.json");
  check(c.value.contextSha256 === hash && c.value.index === 0 && e.value.contextSha256 === hash && e.value.index === 0 && e.value.observationSha256 === o.sha256,
    "bootstrap_capture_join");
  check(o.value.failures.length === 0 || (e.value.code === 1 && canonical(o.value.failures) === canonical([{ category: "noise_command_failed" }])), "bootstrap_capture_observer");
  validateCommandObservation({ ...o.value, failures: [] });
  const owner = await proof(root, "install-0.host-root.json");
  check(c.value.ownerSha256 === owner.sha256 && e.value.ownerSha256 === owner.sha256 && o.value.seen.some(row => sameProcess(row, owner.value)), "bootstrap_capture_owner");
  const verdict = await proof(root, "install-0/flash-command-evidence.json"), f = verdict.value, a = f.fixed_serial_assessment;
  check(f.command_kind === "flash-monitor" && f.flash_status === "completed" && f.board === "205" && f.firmware_commit === context.package.firmware_commit &&
    f.observed_firmware_commit === context.package.firmware_commit && f.nvs_seed_status === "not_provided" && f.redaction_mode === "commit-redacted" &&
    a?.execution_present === true && f.capture_timeout_seconds === 30, "bootstrap_write_unproved");
  check(a.stable_boot === true && Array.isArray(a.issues) && a.issues.every(issue => issue === "error_diagnostic"), "bootstrap_candidate_ambiguous");
  const rows = await readJournal(root, context), before = (await proof(root, "accounting-before.json")).value, admitted = rows[c.value.beforeSequence - 1];
  baseline(admitted?.state, true);
  check(admitted.phase === "before" && c.value.beforeSequence > before.observedSequence &&
    admitted.state.preservation.baseline_id === before.state.preservation.baseline_id && c.value.beforeStateSha256 === sha256(canonical(admitted)) &&
    canonical(c.value.argv) === canonical(argumentsFor(root, context, c.value.detector.port)), "bootstrap_capture_claim");
  const log = await readFile(resolve(root, "install-0/flash-monitor.log"));
  if (f.monitor_log_sha256 !== undefined) check(sha256(log) === f.monitor_log_sha256, "bootstrap_capture_changed");
  const qualified = e.value.code === 0 && a.safe_baseline_confirmed && a.startup_complete && !a.startup_failed && a.stable_boot &&
    !a.retained_failure_history && a.issues?.length === 0 && f.commit_ready === true;
  return { exitCode: e.value.code, flashVerdictSha256: verdict.sha256, qualified: Boolean(qualified), log: log.toString("utf8"), claim: c.value, owners: o.value.seen };
}
export async function install(root, index = 0, operations = {}) {
  check(index === 0, "bootstrap_install_index");
  const context = await load(root, { operations }); await released(root, context);
  let maybeDetectorExit, maybeCaptureExit;
  const producer = { ...operations, childProgram: operations.installChildProgram ?? fileURLToPath(new URL("./install-child.mjs", import.meta.url)), spawn(...args) {
    const child = spawn(...args);
    if (args[1][2] === "detect") maybeDetectorExit = observeDetectorExit(child, root, context).then(() => null, error => error);
    if (args[1][2] === "flash") { maybeCaptureExit = observeCaptureExit(child, root, context).then(() => null, error => error); }
    return child;
  } };
  try {
    await observeCommand(root, context, "detect", 0, producer); if (maybeDetectorExit) { const error = await maybeDetectorExit; if (error) throw error; }
    try { await observeCommand(root, context, "flash", 0, producer); }
    catch (error) { await fail(root, context, "capture", "capture", Object.assign(error, { code: "bootstrap_capture_unqualified" })); }
    if (maybeCaptureExit) { const error = await maybeCaptureExit; if (error) throw error; }
    const captured = await inspectCapture(root, context); await requireGone(captured.owners, operations);
    if (!captured.qualified) await fail(root, context, "capture", "capture", { code: "bootstrap_capture_unqualified" }, captured.flashVerdictSha256);
    const server = (await proof(root, "server-owner.json")).value;
    const response = await fetch(`${server.origin}/install/review`, { method: "POST", headers: { Origin: server.origin, "Content-Type": "application/json" }, body: "{}", signal: AbortSignal.timeout(5000) });
    check(response.ok, "bootstrap_install_review"); return response.json();
  } catch (error) { await fail(root, context, "operator", "install", error); throw error; }
}
