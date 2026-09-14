import { readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, fileDigest, missing, protectedPath, readJson, requireCondition as check, writeNew } from "./contract.mjs";
import { inventory, proof } from "./cadence-premining-evidence.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { requireRecordedAccounting } from "./no-mining-accounting.mjs";
import { requireExhaustedOriginal, requireIdleLedger } from "./iterative-contract.mjs";
import { createResetOriginObservation, parseResetOriginDiagnostic } from "./reset-origin-observation.mjs";
import { loadResetOriginContext } from "./reset-origin-context.mjs";

const key = (d) => `${d.category}:${d.stage ?? ""}`;
const healthy = (d) => d.category === "startup" && d.stage === "runtime_ready" && d.state === "complete" && d.first_failure === "none";
const integer = (value) => Number.isSafeInteger(value) && value >= 0;
function diagnostics(values) {
  check(Array.isArray(values) && values.length <= 40, "reset_origin_batch_observations");
  const parsed = values.map(parseResetOriginDiagnostic);
  check(new Set(parsed.map(key)).size === parsed.length, "reset_origin_duplicate_snapshot_keys");
  return parsed;
}
function baseline(state, released = false) {
  check(
    state.status === (released ? "closed" : "ready") &&
      state.connected === !released &&
      state.serialOwnershipReleased === released &&
      !state.running &&
      !state.failure &&
      state.deviceBaselineConfirmed === true &&
      state.deviceLeaseInactive &&
      state.renewalsConfirmed === 0 &&
      state.preservation?.device_identity_match === true &&
      state.preservation.settings_match === true &&
      state.preservation.authorization_high_water_match === true &&
      state.preservation.mine_on_boot === false,
    "reset_origin_baseline",
  );
}
function cleanup(value) {
  exactObject(value, [
    "schema",
    "source",
    "browser_closed",
    "supervisor_exited",
    "supervisor_exit_code",
    "listener_absent",
    "owned_children_absent",
    "serial_holders_absent",
  ]);
  check(
    value.schema === "worker-reset-origin-cleanup-v1" &&
      value.source === "parent-observed" &&
      value.supervisor_exit_code === 0 &&
      ["browser_closed", "supervisor_exited", "listener_absent", "owned_children_absent", "serial_holders_absent"].every(
        (key) => value[key] === true,
      ),
    "reset_origin_cleanup",
  );
}
async function observation(root, context, start, end) {
  const policy = context.observation_policy,
    hash = digest(JSON.stringify(context));
  exactObject(start, ["schema", "context_sha256", "hostMonotonicMs", "observed_sequence", "primeObservations"]);
  exactObject(end, ["schema", "context_sha256", "hostMonotonicMs", "observed_sequence"]);
  check(
    start.schema === "fixed-usb-reset-origin-start-v1" &&
      end.schema === "fixed-usb-reset-origin-end-v1" &&
      start.context_sha256 === hash &&
      end.context_sha256 === hash &&
      integer(start.hostMonotonicMs) &&
      integer(end.hostMonotonicMs) &&
      end.hostMonotonicMs - start.hostMonotonicMs >= policy.minimum_span_ms &&
      end.hostMonotonicMs - start.hostMonotonicMs <= policy.maximum_span_ms,
    "reset_origin_host_window",
  );
  const names = (await readdir(root)).filter((name) => /^diagnostic-export-.*\.json$/u.test(name)).sort();
  check(
    names.length >= 2 &&
      names.length <= policy.maximum_batches + 1 &&
      names.every((name, index) => name === `diagnostic-export-${String(index).padStart(4, "0")}.json`),
    "reset_origin_batch_files",
  );
  const reducer = createResetOriginObservation({
    firmwareCommit: context.firmware_commit,
    appElfSha256: context.app_elf_sha256,
    minimumSpanMs: policy.minimum_span_ms,
    maximumGapMs: policy.maximum_gap_ms,
    startedAtHostMonotonicMs: start.hostMonotonicMs,
  });
  const last = new Map(),
    hashes = [];
  let sequence = 0,
    lastTime = start.hostMonotonicMs,
    firstBoot,
    firstStartup,
    primeBoot,
    primeStartup;
  for (const [index, name] of names.entries()) {
    const saved = await proof(resolve(root, name)),
      batch = saved.value;
    exactObject(batch, ["schema", "context_sha256", "sequence", "hostMonotonicMs", "observations"]);
    check(
      batch.schema === "fixed-usb-reset-origin-batch-v1" &&
        batch.context_sha256 === hash &&
        batch.sequence === index &&
        integer(batch.hostMonotonicMs),
      "reset_origin_batch_binding",
    );
    const values = diagnostics(batch.observations);
    hashes.push({ file: name, sha256: saved.sha256 });
    if (index === 0) {
      check(
        batch.hostMonotonicMs <= start.hostMonotonicMs &&
          start.hostMonotonicMs - batch.hostMonotonicMs <= policy.maximum_gap_ms &&
          equal(values, diagnostics(start.primeObservations)),
        "reset_origin_prime_binding",
      );
      primeBoot = values.find((d) => d.category === "boot");
      primeStartup = values.find(healthy);
      check(values.every((d) => d.category !== "startup" || (d.state !== "failed" && d.first_failure === "none")), "reset_origin_prime_failure");
      check(
        primeBoot &&
          primeStartup &&
          values.some((d) => d.category === "storage_http_status" && d.spiffs_available === "true" && d.http_ready === "true"),
        "reset_origin_prime_missing",
      );
      for (const d of values) {
        last.set(key(d), d);
        if (!["boot", "startup"].includes(d.category))
          reducer.observe({ sequence: ++sequence, hostMonotonicMs: start.hostMonotonicMs, diagnostic: d });
      }
      continue;
    }
    check(
      batch.hostMonotonicMs >= lastTime &&
        batch.hostMonotonicMs <= end.hostMonotonicMs &&
        batch.hostMonotonicMs - lastTime <= policy.maximum_gap_ms,
      "reset_origin_batch_clock",
    );
    lastTime = batch.hostMonotonicMs;
    // A DOM snapshot has no cross-category wire order. Bind this batch's boot
    // before startup observations at the same server receipt time.
    for (const d of [...values].sort((a, b) => Number(b.category === "boot") - Number(a.category === "boot"))) {
      if (equal(last.get(key(d)), d)) continue;
      if (d.category === "boot" && !firstBoot) {
        check(
          d.boot_ordinal === primeBoot.boot_ordinal && d.reset_reason === primeBoot.reset_reason && d.uptime_ms > primeBoot.uptime_ms,
          "reset_origin_prime_boot_transition",
        );
        firstBoot = d;
      }
      if (healthy(d) && !firstStartup) {
        check(d.uptime_ms > primeStartup.uptime_ms, "reset_origin_prime_startup_regression");
        firstStartup = d;
      }
      last.set(key(d), d);
      reducer.observe({ sequence: ++sequence, hostMonotonicMs: batch.hostMonotonicMs, diagnostic: d });
    }
  }
  check(firstBoot && firstStartup && end.hostMonotonicMs - lastTime <= policy.maximum_gap_ms, "reset_origin_no_fresh_progress");
  const summary = reducer.finish({ hostMonotonicMs: end.hostMonotonicMs });
  check(
    summary.coverageComplete &&
      summary.issues.length === 0 &&
      summary.observedTransitionCount === 0 &&
      summary.segments.length === 1 &&
      summary.unboundStartupRecords === 0 &&
      summary.identityConflicts === 0 &&
      summary.startupFailureRecords === 0 &&
      summary.storageUnavailableRecords === 0 &&
      summary.storageReadyRecords > 0 &&
      summary.unattributedPanicReceiptRecords === 0 &&
      summary.unattributedAllocationReceiptRecords === 0 &&
      summary.bootAdvances >= policy.minimum_boot_advances &&
      summary.healthyStartupAdvances >= policy.minimum_startup_advances &&
      summary.segments[0].boot.spanMs >= policy.minimum_span_ms &&
      summary.segments[0].healthyStartup.spanMs >= policy.minimum_span_ms &&
      summary.recordCount <= policy.maximum_records,
    "reset_origin_observation_unqualified",
  );
  return { summary, batches: hashes, first_boot_uptime_ms: firstBoot.uptime_ms, first_startup_uptime_ms: firstStartup.uptime_ms };
}
async function evidence(root, context, hostCleanup) {
  cleanup(hostCleanup);
  const serverClaim = await proof(resolve(root, "reset-origin-server-claim.json"));
  check(
    equal(serverClaim.value, { schema: "fixed-usb-reset-origin-server-claim-v1", context_sha256: digest(JSON.stringify(context)) }),
    "reset_origin_server_claim",
  );
  for (const name of [
    "issued.json",
    "consumed.json",
    "install-consumed.json",
    "iterative.fault.json",
    "no-mining-read-only-interruption.json",
    "restart-consumed.json",
    "reset-origin-failure.json",
    "failed-inventory.json",
  ])
    await missing(resolve(root, name));
  check(!(await readdir(root)).some((name) => /^(?:flash|install|cycle)-[0-9]/u.test(name)), "reset_origin_effect_evidence_forbidden");
  const inner = context.no_mining_context,
    states = await readNoMiningStates(root, inner);
  const before = (await proof(resolve(root, "no-mining-accounting-before.json"))).value,
    after = (await proof(resolve(root, "no-mining-accounting-after.json"))).value;
  await requireRecordedAccounting(root, inner, {
    ledger_before: before.ledger,
    ledger_after: after.ledger,
    original_budget_before: before.original_budget,
    original_budget_after: after.original_budget,
    recovery_before: before.state,
    recovery_after: after.state,
  });
  for (const value of [before, after]) {
    requireIdleLedger(value.ledger, 17, 1380000);
    requireExhaustedOriginal(value.original_budget);
    baseline(value.state);
    check(equal(states[value.observed_sequence - 1]?.state, value.state), "reset_origin_accounting_state");
  }
  const startFile = await proof(resolve(root, "reset-origin-start.json")),
    endFile = await proof(resolve(root, "reset-origin-end.json")),
    start = startFile.value,
    end = endFile.value;
  check(
    start.observed_sequence === before.observed_sequence &&
      integer(end.observed_sequence) &&
      end.observed_sequence >= start.observed_sequence &&
      end.observed_sequence < after.observed_sequence &&
      after.observed_sequence < states.at(-1).sequence,
    "reset_origin_journal_order",
  );
  baseline(states[start.observed_sequence - 1]?.state);
  baseline(states[end.observed_sequence - 1]?.state);
  baseline(states.at(-1).state, true);
  const baselineId = before.state.preservation.baseline_id;
  check(
    states.every(
      ({ state }) =>
        !state.failure &&
        !state.serialFailureCategory &&
        !state.admissionFailureStage &&
        (state.qualification?.attempt === undefined || state.qualification.attempt.ordinal < 17),
    ) &&
      states
        .slice(start.observed_sequence - 1, after.observed_sequence)
        .every(
          ({ state }) =>
            state.connected &&
            !state.serialOwnershipReleased &&
            !state.running &&
            state.deviceLeaseInactive &&
            state.deviceBaselineConfirmed === true &&
            state.preservation?.baseline_id === baselineId,
        ) &&
      after.state.preservation.baseline_id === baselineId &&
      states.at(-1).state.preservation.baseline_id === baselineId,
    "reset_origin_session_changed",
  );
  return {
    ...(await observation(root, context, start, end)),
    ledger: after.ledger,
    original_budget: after.original_budget,
    final_state: states.at(-1).state,
    before_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-before.json")),
    after_accounting_sha256: await fileDigest(resolve(root, "no-mining-accounting-after.json")),
    start_sha256: startFile.sha256,
    end_sha256: endFile.sha256,
    final_sequence: states.at(-1).sequence,
  };
}
function result(e) {
  return {
    result: "observed_stable",
    observation_qualified: true,
    reset_origin_resolved: false,
    prior_reset_attribution: "unknown",
    restart_performed: false,
    qualification_pass: false,
    mining_authorized: false,
    cadence_admission_authorized: false,
    host_span_ms: e.summary.hostSpanMs,
    boot_advances: e.summary.bootAdvances,
    startup_advances: e.summary.healthyStartupAdvances,
    next_ordinal: e.ledger.next_ordinal,
    total_charged_ms: e.ledger.total_charged_ms,
  };
}
export async function judgeResetOrigin(root, inputPath, operations = {}) {
  root = resolve(root);
  const context = await loadResetOriginContext(root, { operations });
  await protectedPath(inputPath);
  const hostCleanup = await readJson(inputPath);
  const observed = await evidence(root, context, hostCleanup);
  const receipt = {
    schema: "fixed-usb-reset-origin-result-v1",
    context,
    context_sha256: digest(JSON.stringify(context)),
    ...result(observed),
    ...observed,
    cleanup: hostCleanup,
    cleanup_input: { path: resolve(inputPath), sha256: await fileDigest(inputPath) },
    inventory: (await inventory(root)).filter((row) => row.path !== "result.json"),
  };
  await writeNew(resolve(root, "result.json"), { receipt, sha256: digest(JSON.stringify(receipt)) });
  return result(observed);
}
export async function readResetOrigin(path, operations = {}) {
  path = resolve(path);
  const root = dirname(path);
  check(path === resolve(root, "result.json"), "reset_origin_result_path");
  const saved = await proof(path);
  exactObject(saved.value, ["receipt", "sha256"]);
  const receipt = saved.value.receipt;
  check(
    saved.value.sha256 === digest(JSON.stringify(receipt)) && receipt.schema === "fixed-usb-reset-origin-result-v1",
    "reset_origin_result_integrity",
  );
  const context = await loadResetOriginContext(root, { historical: true, operations });
  check(receipt.context_sha256 === digest(JSON.stringify(context)) && equal(context, receipt.context), "reset_origin_result_context");
  await protectedPath(receipt.cleanup_input.path);
  check(
    (await fileDigest(receipt.cleanup_input.path)) === receipt.cleanup_input.sha256 &&
      equal(await readJson(receipt.cleanup_input.path), receipt.cleanup),
    "reset_origin_cleanup_changed",
  );
  const observed = await evidence(root, context, receipt.cleanup);
  const expected = {
    schema: "fixed-usb-reset-origin-result-v1",
    context,
    context_sha256: digest(JSON.stringify(context)),
    ...result(observed),
    ...observed,
    cleanup: receipt.cleanup,
    cleanup_input: receipt.cleanup_input,
    inventory: (await inventory(root)).filter((row) => row.path !== "result.json"),
  };
  check(equal(receipt, expected), "reset_origin_result_changed");
  return receipt;
}
