import { chmod, lstat, mkdir, readFile, realpath, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { basename, dirname, isAbsolute, resolve } from "node:path";
import { runRecordedCommand } from "./runner.mjs";

const SOURCE = await readFile(new URL("./fixtures/native-probe.rs", import.meta.url));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const labelPattern = /^[a-z][a-z0-9_-]{0,39}$/u;
const outcomes = new Set(["success", "nonzero", "spawn_error", "timeout", "cancelled", "evidence_error", "cleanup_failed", "marker_missing"]);

async function privateDirectory(path) {
  const info = await lstat(path);
  if (!info.isDirectory() || info.isSymbolicLink() || (info.mode & 0o077) !== 0 || info.uid !== process.getuid()) throw new Error("private_directory_required");
  return realpath(path);
}
async function createRoot(path) {
  const parent = await privateDirectory(dirname(resolve(path)));
  const root = resolve(parent, basename(resolve(path)));
  await mkdir(root, { mode: 0o700 });
  return root;
}
async function save(path, value) {
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx", mode: 0o600 });
}

/** Compile a single immutable entry-marker executable; subsequent comparisons reuse it. */
export async function prepareProbe({ root, timeoutMs = 120000 }) {
  root = await createRoot(root);
  const source = resolve(root, "probe.rs"), binary = resolve(root, "native-probe");
  await writeFile(source, SOURCE, { flag: "wx", mode: 0o600 });
  const compiled = await runRecordedCommand({ command: ["rustc", "+stable", source, "--edition", "2021", "-C", "debuginfo=0", "-o", binary],
    cwd: root, evidenceRoot: resolve(root, "compile"), label: "probe-compile", quietMs: 5000, timeoutMs });
  if (compiled.outcome !== "success") throw new Error("probe_compile_failed_see_private_evidence");
  await chmod(binary, 0o700);
  const manifest = { schema: "host-stall-probe-v1", binary: "native-probe", binarySha256: hash(await readFile(binary)),
    sourceSha256: hash(SOURCE), marker: "host_stall_probe_entered_main", compiler: "rustc +stable", debugInfo: 0 };
  await save(resolve(root, "probe.json"), manifest);
  return { root, ...manifest };
}

async function loadProbe(bundle) {
  bundle = await privateDirectory(resolve(bundle));
  const manifestPath = resolve(bundle, "probe.json"), binary = resolve(bundle, "native-probe");
  for (const path of [manifestPath, binary]) {
    const info = await lstat(path);
    if (!info.isFile() || info.isSymbolicLink() || (info.mode & 0o077) !== 0) throw new Error("private_probe_file_required");
  }
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  if (manifest.schema !== "host-stall-probe-v1" || manifest.binary !== "native-probe" ||
    manifest.binarySha256 !== hash(await readFile(binary)) || manifest.sourceSha256 !== hash(SOURCE) ||
    manifest.marker !== "host_stall_probe_entered_main") throw new Error("probe_identity_mismatch");
  return { bundle, binary, manifest };
}

/** Record bounded repeated launches with the same bytes, path and working directory. */
export async function launchProbe({ bundle, root, label, runs = 3, timeoutMs = 15000, quietMs = 2000, signal }, recordCommand = runRecordedCommand) {
  if (!labelPattern.test(label) || !Number.isInteger(runs) || runs < 1 || runs > 10) throw new Error("probe_options_invalid");
  const probe = await loadProbe(bundle);
  root = await createRoot(root);
  const results = [];
  const controller = new AbortController();
  const cancel = () => controller.abort();
  const combinedSignal = signal ? AbortSignal.any([signal, controller.signal]) : controller.signal;
  process.on("SIGINT", cancel);
  process.on("SIGTERM", cancel);
  try {
    for (let index = 0; index < runs && !combinedSignal.aborted; index += 1) {
      await loadProbe(bundle);
      const result = await recordCommand({ command: [probe.binary], cwd: probe.bundle,
        evidenceRoot: resolve(root, `run-${String(index + 1).padStart(2, "0")}`), label, timeoutMs, quietMs,
        expectedStderrMarker: probe.manifest.marker, signal: combinedSignal });
      await loadProbe(bundle);
      const maybeMarker = result.events.find((event) => event.type === "stderr_marker_observed");
      const maybeSpawn = result.events.find((event) => event.type === "spawn_requested");
      result.markerObserved = Boolean(maybeMarker);
      result.entryObservedMs = maybeMarker && maybeSpawn ? maybeMarker.offsetMs - maybeSpawn.offsetMs : null;
      result.probeOutcome = result.outcome === "success" && !maybeMarker ? "marker_missing" : result.outcome;
      results.push(result);
      await save(resolve(root, `result-${index + 1}.json`), result);
      // Cancellation ends the whole requested series, even after successful cleanup.
      if (result.outcome === "cancelled" || result.cleanup?.complete !== true) break;
    }
    const cancelled = combinedSignal.aborted || results.some((result) => result.outcome === "cancelled");
    const record = { schema: "host-stall-probe-series-v1", label, binary: probe.binary, cwd: probe.bundle,
      binarySha256: probe.manifest.binarySha256, timeoutMs, quietMs, requestedRuns: runs,
      complete: results.length === runs && !cancelled, cancelled, results };
    await save(resolve(root, "series.json"), record);
    return record;
  } finally {
    process.off("SIGINT", cancel);
    process.off("SIGTERM", cancel);
  }
}

/** Compare observations without promoting a label or a successful run into a causal claim. */
export function compareSeries(left, right) {
  for (const series of [left, right]) {
    if (series?.schema !== "host-stall-probe-series-v1" || !Array.isArray(series.results) || !series.results.length ||
      series.results.length !== series.requestedRuns || series.complete === false || series.cancelled === true) throw new Error("incomplete_probe_series");
    if (typeof series.binary !== "string" || !isAbsolute(series.binary) || typeof series.cwd !== "string" || !isAbsolute(series.cwd) ||
      ![series.timeoutMs, series.quietMs, series.requestedRuns].every((value) => Number.isSafeInteger(value) && value > 0) ||
      series.requestedRuns > 10 || !labelPattern.test(series.label)) throw new Error("invalid_probe_conditions");
    if (!/^[a-f0-9]{64}$/u.test(series.binarySha256)) throw new Error("invalid_probe_digest");
    for (const result of series.results) {
      if (!outcomes.has(result.probeOutcome) || typeof result.markerObserved !== "boolean" ||
        (result.markerObserved && (!Number.isFinite(result.entryObservedMs) || result.entryObservedMs < 0)) ||
        (!result.markerObserved && result.entryObservedMs !== null) ||
        (result.probeOutcome === "success" && (result.outcome !== "success" || !result.markerObserved || result.cleanup?.complete !== true)))
        throw new Error("invalid_probe_observation");
    }
  }
  for (const field of ["binary", "cwd", "binarySha256", "timeoutMs", "quietMs"]) {
    if (left[field] !== right[field]) throw new Error(`probe_comparison_mismatch:${field}`);
  }
  return { schema: "host-stall-comparison-v1", binarySha256: left.binarySha256,
    series: [left, right].map((series) => ({ label: series.label, runs: series.results.length,
      outcomes: series.results.map((result) => result.probeOutcome), entryObservedMs: series.results.map((result) => result.entryObservedMs),
      events: series.results.map((result) => result.events),
      ancestors: series.results.map((result) => result.ancestors) })),
    conclusion: "observations_only_execution_labels_are_not_causal_proof" };
}

export async function compareProbeRoots(left, right) {
  return compareSeries(...await Promise.all([left, right].map(async (root) => {
    root = await privateDirectory(resolve(root));
    const path = resolve(root, "series.json"), info = await lstat(path);
    if (!info.isFile() || info.isSymbolicLink() || (info.mode & 0o077)) throw new Error("private_series_required");
    return JSON.parse(await readFile(path, "utf8"));
  })));
}
