import { requireActiveStatistics } from "./reset-origin-restart-statistics.mjs";
import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { digest, exactObject, requireCondition as check } from "./contract.mjs";
import { proof } from "./cadence-premining-evidence.mjs";
import { createResetOriginObservation, parseResetOriginDiagnostic } from "./reset-origin-observation.mjs";

const key = (d) => `${d.category}:${d.stage ?? d.origin ?? ""}`;
const healthy = (d) => d.category === "startup" && d.stage === "runtime_ready" && d.state === "complete" && d.first_failure === "none";
const integer = (value) => Number.isSafeInteger(value) && value >= 0;
function diagnostics(values) {
  check(Array.isArray(values) && values.length <= 40, "reset_origin_batch_observations");
  const parsed = values.map(parseResetOriginDiagnostic);
  check(new Set(parsed.map(key)).size === parsed.length, "reset_origin_duplicate_snapshot_keys");
  return parsed;
}
/** Validate capture facts; callers separately enforce session, accounting and effect authority. */
export async function inspectResetOriginObservation(root, context, start, end) {
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
    check(
      values.every((value) => value.category !== "statistics_startup" || ["prepared", "active"].includes(value.state)),
      "statistics_startup_not_active",
    );
    if (context.statistics_startup_required) requireActiveStatistics(values);
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
      check(
        values.every((d) => d.category !== "startup" || (d.state !== "failed" && d.first_failure === "none")),
        "reset_origin_prime_failure",
      );
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
