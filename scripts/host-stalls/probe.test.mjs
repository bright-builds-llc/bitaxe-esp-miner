import test from "node:test";
import assert from "node:assert/strict";
import { compareSeries, launchProbe } from "./probe.mjs";
import { chmod, mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";

function series(label) {
  return { schema: "host-stall-probe-series-v1", label, binary: "/private/probe/native-probe", cwd: "/private/probe",
    binarySha256: "a".repeat(64), timeoutMs: 15000, quietMs: 2000, requestedRuns: 1,
    results: [{ outcome: "success", probeOutcome: "success", markerObserved: true, entryObservedMs: 3,
      events: [{ type: "stderr_marker_observed", offsetMs: 3 }], ancestors: [], cleanup: { complete: true } }] };
}

test("identical probe bytes and settings produce observations without a causal claim", () => {
  // Arrange
  const left = series("agent"), right = series("terminal");
  right.results[0].outcome = "timeout";
  right.results[0].probeOutcome = "timeout";
  // Act
  const compared = compareSeries(left, right);
  // Assert
  assert.deepEqual(compared.series.map((entry) => entry.outcomes), [["success"], ["timeout"]]);
  assert.equal(compared.conclusion, "observations_only_execution_labels_are_not_causal_proof");
});
test("comparison refuses changed bytes even when both executions pass", () => {
  const right = series("terminal");
  right.binarySha256 = "b".repeat(64);
  assert.throws(() => compareSeries(series("agent"), right), /probe_comparison_mismatch:binarySha256/u);
});
test("comparison refuses unequal execution deadlines", () => {
  const right = series("terminal");
  right.timeoutMs = 30000;
  assert.throws(() => compareSeries(series("agent"), right), /probe_comparison_mismatch:timeoutMs/u);
});
test("incomplete series cannot appear as a complete comparison", () => {
  const right = series("terminal");
  right.requestedRuns = 3;
  assert.throws(() => compareSeries(series("agent"), right), /incomplete_probe_series/u);
});
test("successful comparison inputs require an observed entry marker", () => {
  const right = series("terminal");
  right.results[0].markerObserved = false;
  right.results[0].entryObservedMs = null;
  assert.throws(() => compareSeries(series("agent"), right), /invalid_probe_observation/u);
});
test("successful comparison inputs require proven cleanup", () => {
  const right = series("terminal");
  right.results[0].cleanup.complete = false;
  assert.throws(() => compareSeries(series("agent"), right), /invalid_probe_observation/u);
});
test("both sides missing execution conditions cannot pass comparison", () => {
  const left = series("agent"), right = series("terminal");
  for (const value of [left, right]) for (const key of ["binary", "cwd", "timeoutMs", "quietMs"]) delete value[key];
  assert.throws(() => compareSeries(left, right), /invalid_probe_conditions/u);
});

async function inertBundle(t) {
  const parent = await realpath(await mkdtemp(join(tmpdir(), "probe-series-test-")));
  await chmod(parent, 0o700);
  t.after(() => rm(parent, { recursive: true, force: true }));
  const bundle = join(parent, "bundle");
  await mkdir(bundle, { mode: 0o700 });
  const binary = Buffer.from("inert fixture; the injected recorder never executes these bytes");
  await writeFile(join(bundle, "native-probe"), binary, { flag: "wx", mode: 0o700 });
  const digest = (value) => createHash("sha256").update(value).digest("hex");
  const source = await readFile(new URL("./fixtures/native-probe.rs", import.meta.url));
  await writeFile(join(bundle, "probe.json"), JSON.stringify({ schema: "host-stall-probe-v1", binary: "native-probe",
    binarySha256: digest(binary), sourceSha256: digest(source), marker: "host_stall_probe_entered_main" }), { flag: "wx", mode: 0o600 });
  return { bundle, root: join(parent, "series"), label: "fixture", runs: 3 };
}

test("a cancelled probe ends the entire series after successful cleanup", async (t) => {
  // Arrange
  const options = await inertBundle(t);
  let calls = 0;
  const listeners = process.listenerCount("SIGTERM");
  // Act
  const result = await launchProbe(options, async () => {
    calls += 1;
    return { outcome: "cancelled", events: [], cleanup: { complete: true } };
  });
  // Assert
  assert.equal(calls, 1);
  assert.equal(result.complete, false);
  assert.equal(result.cancelled, true);
  assert.equal(process.listenerCount("SIGTERM"), listeners);
  assert.throws(() => compareSeries(result, result), /incomplete_probe_series/u);
});

test("cancellation between runs preserves success without starting another child", async (t) => {
  // Arrange
  const options = await inertBundle(t), controller = new AbortController();
  let calls = 0;
  // Act
  const result = await launchProbe({ ...options, signal: controller.signal }, async () => {
    calls += 1;
    controller.abort();
    return { outcome: "success", events: [{ type: "spawn_requested", offsetMs: 1 }, { type: "stderr_marker_observed", offsetMs: 2 }], cleanup: { complete: true } };
  });
  // Assert
  assert.equal(calls, 1);
  assert.equal(result.results[0].probeOutcome, "success");
  assert.equal(result.cancelled, true);
  assert.equal(result.complete, false);
});
