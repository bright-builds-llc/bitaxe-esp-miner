import { resolve, dirname } from "node:path";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { installedRestartFixture, recordRestartFixture, restartPacket } from "./reset-origin-restart-fixtures.mjs";
import { restartInnerContext } from "./reset-origin-restart-context.mjs";
import { recoveryState } from "./cadence-startup-fixtures.mjs";
import { resetDiagnostics } from "./reset-origin-fixtures.mjs";
import { saveRestartAccounting } from "./reset-origin-restart-state.mjs";
import { inspectResetOriginObservation } from "./reset-origin-observation-review.mjs";
import { judgeRestart, readRestartResult } from "./reset-origin-restart-judge.mjs";
import { RUNTIME_KEYS } from "./reset-origin-runtime-source.mjs";

async function completeRestart(f) {
  const root = f.root,
    context = f.context,
    hash = digest(JSON.stringify(context)),
    scope = resolve(root, "after-install"),
    inner = restartInnerContext(root, context, "after-install");
  await writeNew(resolve(root, "restart-server-claim.json"), { schema: "fixed-usb-restart-server-claim-v1", context_sha256: hash });
  await writeNew(resolve(root, "install-phase-advanced.json"), { schema: "fixed-usb-restart-phase-v1", context_sha256: hash });
  await recordRestartFixture(scope, inner, recoveryState(inner));
  await saveRestartAccounting(scope, inner, {
    stage: "before",
    ledger: f.ledger,
    original_budget: f.original,
    state: recoveryState(inner),
  });
  const diagnostics = (uptime) =>
    resetDiagnostics(context, uptime).map((d) => (d.category === "boot" ? { ...d, boot_ordinal: 7, reset_reason: "software_cpu" } : d));
  const start = {
    schema: "fixed-usb-reset-origin-start-v1",
    context_sha256: hash,
    hostMonotonicMs: 200,
    observed_sequence: 1,
    primeObservations: diagnostics(1000),
  };
  await writeNew(resolve(scope, "reset-origin-start.json"), start);
  await writeNew(resolve(scope, "diagnostic-export-0000.json"), {
    schema: "fixed-usb-reset-origin-batch-v1",
    context_sha256: hash,
    sequence: 0,
    hostMonotonicMs: 100,
    observations: diagnostics(1000),
  });
  for (let n = 1; n <= 260; n++)
    await writeNew(resolve(scope, `diagnostic-export-${String(n).padStart(4, "0")}.json`), {
      schema: "fixed-usb-reset-origin-batch-v1",
      context_sha256: hash,
      sequence: n,
      hostMonotonicMs: 200 + n * 500,
      observations: diagnostics(1000 + n * 500),
    });
  await recordRestartFixture(scope, inner, recoveryState(inner));
  const end = { schema: "fixed-usb-reset-origin-end-v1", context_sha256: hash, hostMonotonicMs: 130200, observed_sequence: 2 };
  await writeNew(resolve(scope, "reset-origin-end.json"), end);
  await writeNew(resolve(root, "pre-restart-observation.json"), {
    schema: "fixed-usb-pre-restart-observation-v1",
    context_sha256: hash,
    observed: await inspectResetOriginObservation(scope, context, start, end),
  });
  await recordRestartFixture(scope, inner, recoveryState(inner));
  await writeNew(resolve(root, "restart-consumed.json"), {
    schema: "fixed-usb-restart-consumed-v1",
    context_sha256: hash,
    request_nonce_sha256: digest(context.request_nonce),
    expected_boot_ordinal: 7,
    before_sequence: 3,
    baseline_id: recoveryState(inner).preservation.baseline_id,
    hostMonotonicMs: 130300,
  });
  await recordRestartFixture(scope, inner, { ...recoveryState(inner), status: "restarting", deviceBaselineConfirmed: false });
  const packet = restartPacket(context),
    ready = { ...recoveryState(inner), restart: packet.summary };
  await recordRestartFixture(scope, inner, ready);
  await writeNew(resolve(root, "restart-observation.json"), {
    schema: "fixed-usb-restart-observation-v1",
    context_sha256: hash,
    claim_sha256: await fileDigest(resolve(root, "restart-consumed.json")),
    after_sequence: 5,
    hostMonotonicMs: 130450,
    evidence: packet,
  });
  await recordRestartFixture(scope, inner, ready);
  await saveRestartAccounting(scope, inner, { stage: "after", ledger: f.ledger, original_budget: f.original, state: ready });
  await recordRestartFixture(scope, inner, { ...recoveryState(inner, true), restart: packet.summary });
  await writeNew(resolve(root, "restart-finished.json"), {
    schema: "fixed-usb-restart-finished-v1",
    context_sha256: hash,
    final_sequence: 7,
  });
  const cleanup = resolve(root, "host-cleanup.json");
  await writeNew(cleanup, {
    schema: "worker-restart-host-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
  await judgeRestart(root, cleanup, f.operations);
}

export async function cadenceRestartFixture(t) {
  const f = await installedRestartFixture(t);
  await completeRestart(f);
  const receiptPath = resolve(f.root, "result.json"),
    input = resolve(f.base, "cadence-restart-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [await fileDigest(receiptPath)],
  });
  // This trusted-reader fixture exercises real Stage-B reader/inventory validation. Exact production anchors are tested separately in the pure evidence validator.
  const readCadenceRestart = async (path) => ({
    root: dirname(path),
    receipt: await readRestartResult(path, f.operations),
    failed: f.failedContext,
    failed_root: f.failedRoot,
    stage_a: await f.operations.readStageA(f.context.stage_a.path),
    binding: { root: dirname(path), receipt_sha256: await fileDigest(path) },
  });
  const source = {
    ...Object.fromEntries(RUNTIME_KEYS.map((key) => [key, f.context[key]])),
    supervisor_client_sha256: await fileDigest(new URL("./client.mjs", import.meta.url)),
    authority_trust_sha256: f.context.trust_sha256,
  };
  const options = {
    ...f.options,
    privateRoot: resolve(f.base, "attempts/cadence-after-restart"),
    authorityDirectory: resolve(f.base, "authority"),
    previousReceipt: f.failedContext.previous_receipt,
    input,
    suggestedDifficulty: "1000",
    observerBinary: f.observer,
    supersedeRestart: receiptPath,
  };
  const operations = { ...f.operations, readCadenceRestart, inspectSources: async () => source };
  return { ...f, options, operations, source, receiptPath, prior: await readCadenceRestart(receiptPath) };
}
