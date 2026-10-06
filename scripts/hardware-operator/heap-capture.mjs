// Passive internal-heap series: per window a fresh detector and one receive-only `just monitor`
// capture into private files, then one judgement over every window (lesson-diagnose-heap-loss-from-a-passive-series).
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { judgeSeries, parseSamples } from "../internal-heap-series/series.mjs";
import { refuse } from "./errors.mjs";
import { absent, breadcrumb, privateDirectory } from "./host.mjs";
import { detect, runJust } from "./just.mjs";

// AGENTS.md "Flash And Monitor Timeouts": hardware monitor captures use at least 360 seconds.
export const MINIMUM_WINDOW_SECONDS = 360;
export const MAXIMUM_WINDOWS = 48;

export function windowNames(name, windows) {
  refuse(/^[a-z0-9][a-z0-9-]{0,63}$/u.test(name ?? ""), "heap_capture_name");
  refuse(Number.isSafeInteger(windows) && windows >= 1 && windows <= MAXIMUM_WINDOWS, "heap_capture_windows");
  return Array.from({ length: windows }, (_, index) => `${name}-${String(index + 1).padStart(3, "0")}`);
}

const windowFiles = (parent, window) => ({
  detector: resolve(parent, `${window}.detector.stdout.log`), detectorStderr: resolve(parent, `${window}.detector.stderr.log`),
  capture: resolve(parent, `${window}.capture.log`), monitorStderr: resolve(parent, `${window}.monitor.stderr.log`),
});

/** Capture `windows` consecutive windows; every window must see the same physical device. */
export async function heapCapture({ parent, name, windows, seconds, maybeThresholds }, operations = {}) {
  process.umask(0o077);
  refuse(Number.isSafeInteger(seconds) && seconds >= MINIMUM_WINDOW_SECONDS, "heap_capture_seconds");
  parent = await privateDirectory(parent, "heap_capture_parent_missing");
  const names = windowNames(name, windows);
  for (const window of names) for (const path of Object.values(windowFiles(parent, window))) await absent(path, "heap_capture_output_exists");
  let maybePhysical = null;
  const captured = [];
  for (const window of names) {
    const files = windowFiles(parent, window);
    const device = await detect(files.detector, files.detectorStderr, operations);
    refuse(maybePhysical === null || device.physical === maybePhysical, "physical_identity_changed");
    maybePhysical = device.physical;
    const exit = await runJust(["monitor", "--board", "205", "--port", device.port, "--expected-physical-sha256", device.physical,
      "--capture-timeout-seconds", String(seconds)], { stdoutPath: files.capture, stderrPath: files.monitorStderr }, operations);
    const samples = parseSamples(await readFile(files.capture, "latin1"));
    captured.push({ window, monitor_exit: exit, samples: samples.length });
    await breadcrumb(parent, "heap-capture", { window, monitor_exit: exit, samples: samples.length });
    // A failed or empty window ends the chain: later windows could not repair the gap in the series.
    if (exit !== 0 || samples.length === 0) return { event: "heap_capture_finished", windows: captured, failed_window: window };
  }
  const summary = { event: "heap_capture_finished", windows: captured };
  if (!maybeThresholds) return summary;
  const texts = await Promise.all(names.map((window) => readFile(windowFiles(parent, window).capture, "latin1")));
  return { ...summary, judgement: judgeSeries(texts.flatMap(parseSamples), maybeThresholds) };
}
