import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { runRecordedCommand } from "./runner.mjs";
import { compareProbeRoots, launchProbe, prepareProbe } from "./probe.mjs";
import { createEvidenceRoot, writeEvidence } from "./evidence.mjs";

const HELP = `Host-only stall diagnostics (no automatic retry or quiet-output termination).
  prepare --root FRESH_PRIVATE_CHILD [--timeout-ms 120000]
  probe --bundle PREPARED_ROOT --root FRESH_PRIVATE_CHILD --label LABEL [--runs 3]
  run --root FRESH_PRIVATE_CHILD --label LABEL [--cwd PATH] [--quiet-ms 15000] [--timeout-ms N] -- COMMAND ARGS...
  compare --left SERIES_ROOT --right SERIES_ROOT --root FRESH_PRIVATE_CHILD
Parents must exist and be private (0700). Output and command arguments stay in private evidence.
An explicit timeout stops only the owned process group. Quiet intervals collect diagnostics.
`;

function parse(args) {
  const [mode, ...rest] = args, options = {}, separator = rest.indexOf("--");
  const flags = separator < 0 ? rest : rest.slice(0, separator);
  const command = separator < 0 ? [] : rest.slice(separator + 1);
  const allowed = { prepare: ["root", "timeout-ms"], probe: ["bundle", "root", "label", "runs", "quiet-ms", "timeout-ms"],
    run: ["root", "label", "cwd", "quiet-ms", "timeout-ms"], compare: ["left", "right", "root"] }[mode];
  if (!allowed) throw new Error("unknown_diagnostic_command");
  for (let index = 0; index < flags.length; index += 2) {
    const key = flags[index]?.slice(2), value = flags[index + 1];
    if (!flags[index]?.startsWith("--") || !allowed.includes(key) || key in options || value === undefined || value.startsWith("--"))
      throw new Error("invalid_diagnostic_option");
    options[key] = value;
  }
  if (mode !== "run" && command.length) throw new Error("unexpected_child_command");
  const required = { prepare: ["root"], probe: ["bundle", "root", "label"], run: ["root", "label"], compare: ["left", "right", "root"] }[mode];
  if (required.some((key) => !options[key]) || (mode === "run" && !command.length)) throw new Error("missing_diagnostic_option");
  for (const key of ["runs", "quiet-ms", "timeout-ms"]) {
    if (!(key in options)) continue;
    if (!/^[1-9][0-9]*$/u.test(options[key]) || !Number.isSafeInteger(Number(options[key]))) throw new Error("invalid_diagnostic_number");
    options[key] = Number(options[key]);
  }
  return { mode, options, command };
}

export async function main(args) {
  if (!args.length || args[0] === "--help") { process.stdout.write(HELP); return 0; }
  const { mode, options, command } = parse(args);
  let result;
  if (mode === "prepare") result = await prepareProbe({ root: options.root, timeoutMs: options["timeout-ms"] });
  if (mode === "probe") result = await launchProbe({ bundle: options.bundle, root: options.root, label: options.label,
    runs: options.runs, quietMs: options["quiet-ms"], timeoutMs: options["timeout-ms"] });
  if (mode === "compare") {
    const compared = await compareProbeRoots(options.left, options.right);
    const root = createEvidenceRoot(resolve(options.root));
    writeEvidence(root, "comparison.json", compared);
    result = { schema: compared.schema, binarySha256: compared.binarySha256, conclusion: compared.conclusion, evidenceRoot: root,
      series: compared.series.map(({ label, outcomes, entryObservedMs }) => ({ label, outcomes, entryObservedMs })) };
  }
  if (mode === "run") {
    const recorded = await runRecordedCommand({ command, cwd: resolve(options.cwd ?? process.cwd()), evidenceRoot: resolve(options.root),
      label: options.label, quietMs: options["quiet-ms"], timeoutMs: options["timeout-ms"] });
    // Never print raw command arguments, child output or captured process data.
    const events = recorded.events.map(({ type, utc, offsetMs, pid, code, signal, ordinal }) => ({ type, utc, offsetMs, pid, code, signal, ordinal }));
    result = { schema: recorded.schema, outcome: recorded.outcome, evidenceRoot: recorded.evidenceRoot, events,
      cleanupComplete: recorded.cleanup.complete, recorderFailures: recorded.failures.length,
      captures: recorded.captures.map(({ ordinal, status, diagnosticFailures }) => ({ ordinal, status, diagnosticFailures })) };
  }
  if (mode === "probe") result = { schema: result.schema, label: result.label, binarySha256: result.binarySha256,
    complete: result.complete, cancelled: result.cancelled,
    outcomes: result.results.map((run) => run.probeOutcome), entryObservedMs: result.results.map((run) => run.entryObservedMs),
    evidenceRoot: resolve(options.root) };
  process.stdout.write(`${JSON.stringify(result)}\n`);
  return result.complete === false || result.outcome && result.outcome !== "success" || result.outcomes?.some((outcome) => outcome !== "success") ? 1 : 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  main(process.argv.slice(2)).then((code) => { process.exitCode = code; }, (error) => {
    const category = /^[A-Z][A-Z0-9_]{0,63}$/u.test(error.code ?? "") ? error.code
      : /^[a-z][a-z0-9_]{0,80}$/u.test(error.message ?? "") ? error.message : "diagnostic_failed";
    process.stderr.write(`host-stall diagnostics failed: ${category}\n`);
    process.exitCode = 1;
  });
}
