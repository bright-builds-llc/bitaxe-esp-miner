// `just hardware-operator <action> ...`: repo-owned sequencing for task-gated hardware runs.
// These commands add no authority; every effect they start still needs an active TASKS.md contract.
// Usage: scripts/hardware-operator/README.md
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { HardwareOperatorError, refuse } from "./errors.mjs";
import { heapCapture } from "./heap-capture.mjs";
import { userPath, workspace } from "./host.mjs";
import { DEFAULT_REPLY_TIMEOUT_MS, awaitPending, launch, send, stopHolder } from "./noise-operator.mjs";
import { stopAndFinish } from "./owner-finish.mjs";
import { recoveryCoreDump, restartSequence } from "./sequences.mjs";

const string = { type: "string" };
const ACTIONS = {
  "noise-launch": { options: { "private-root": string }, required: ["private-root"] },
  "noise-send": { options: { "private-root": string, command: string, index: string, "timeout-seconds": string }, required: ["private-root", "command"] },
  "noise-await": { options: { "private-root": string, "timeout-seconds": string }, required: ["private-root"] },
  "noise-stop-holder": { options: { "private-root": string }, required: ["private-root"] },
  "owner-finish": { options: { owner: string, "private-root": string, stage: string, "wait-collection": { type: "boolean" } }, required: ["owner", "private-root"] },
  "restart-sequence": { options: { owner: string, "private-root": string }, required: ["owner", "private-root"] },
  "recovery-core-dump": { options: { owner: string, "private-root": string, "core-root": string, "wait-collection": { type: "boolean" } },
    required: ["owner", "private-root", "core-root"] },
  "heap-capture": { options: { parent: string, name: string, seconds: string, windows: string, "min-free-bytes": string,
    "min-largest-block-bytes": string, "min-samples": string }, required: ["parent", "name", "seconds"] },
};
const THRESHOLDS = { "min-free-bytes": "minFreeBytes", "min-largest-block-bytes": "minLargestBlockBytes", "min-samples": "minSamples" };

function count(value, code) {
  refuse(/^\d{1,9}$/u.test(value ?? ""), code);
  return Number(value);
}

/** Parse argv into one closed action and its options; unknown actions, flags and positionals are refused. */
export function argumentsFor(argv) {
  const [action, ...rest] = argv;
  const spec = ACTIONS[action];
  refuse(spec !== undefined, "hardware_operator_action");
  let values;
  try { ({ values } = parseArgs({ args: rest, options: spec.options, strict: true, allowPositionals: false })); } catch {
    throw new HardwareOperatorError("hardware_operator_arguments");
  }
  for (const name of spec.required) refuse(values[name] !== undefined, "hardware_operator_arguments");
  return { action, values };
}

function thresholdsFrom(values) {
  const given = Object.keys(THRESHOLDS).filter((name) => values[name] !== undefined);
  refuse(given.length === 0 || given.length === 3, "heap_capture_thresholds");
  if (given.length === 0) return null;
  return Object.fromEntries(given.map((name) => [THRESHOLDS[name], count(values[name], "heap_capture_thresholds")]));
}

/** `--command install --index 2` becomes the parent's JSON line; the parent's closed rules still validate it. */
export function commandLine(values) {
  const index = values.index === undefined ? undefined : count(values.index, "noise_command_install_index");
  return JSON.stringify({ action: values.command, ...(index === undefined ? {} : { index }) });
}

function replyTimeoutMs(values) {
  if (values["timeout-seconds"] === undefined) return DEFAULT_REPLY_TIMEOUT_MS;
  const seconds = count(values["timeout-seconds"], "noise_send_timeout");
  refuse(seconds >= 1, "noise_send_timeout");
  return seconds * 1000;
}

const noiseParent = () => resolve(workspace(), "scripts/hardware-operator/noise-parent.mjs");

export async function run(action, values) {
  const root = values["private-root"] === undefined ? undefined : userPath(values["private-root"]);
  if (action === "noise-launch") return launch(root, { parentProgram: noiseParent() });
  if (action === "noise-send") return send(root, commandLine(values), { replyTimeoutMs: replyTimeoutMs(values) });
  if (action === "noise-await") return awaitPending(root, { replyTimeoutMs: replyTimeoutMs(values) });
  if (action === "noise-stop-holder") return stopHolder(root);
  if (action === "owner-finish") {
    const { summary } = await stopAndFinish({ name: values.owner, root, maybeStage: values.stage, waitCollection: values["wait-collection"] === true });
    return summary;
  }
  if (action === "restart-sequence") return restartSequence({ name: values.owner, root });
  if (action === "recovery-core-dump")
    return recoveryCoreDump({ name: values.owner, root, coreRoot: userPath(values["core-root"]), waitCollection: values["wait-collection"] === true });
  return heapCapture({ parent: userPath(values.parent), name: values.name, seconds: count(values.seconds, "heap_capture_seconds"),
    windows: values.windows === undefined ? 1 : count(values.windows, "heap_capture_windows"), maybeThresholds: thresholdsFrom(values) });
}

/** Exit 0 only when the action and any command it ran succeeded; typed codes are safe to print. */
export function exitCodeFor(result) {
  if (result.event === "operator_error") return 1;
  if (result.finish_exit !== undefined && result.finish_exit !== 0) return 1;
  if (result.read_exit !== undefined && result.read_exit !== 0) return 1;
  if (result.judgement && result.judgement.passed !== true) return 1;
  if (result.failed_window !== undefined) return 1;
  return 0;
}

/** Repo errors carry snake_case codes (HardwareOperatorError, QualificationError); Node system codes are uppercase. */
export function typedCode(error) {
  return typeof error?.code === "string" && /^[a-z][a-z0-9_]*$/u.test(error.code) ? error.code : null;
}

async function main(argv) {
  try {
    const { action, values } = argumentsFor(argv);
    const result = await run(action, values);
    process.stdout.write(`${JSON.stringify(result)}\n`);
    process.exitCode = exitCodeFor(result);
  } catch (error) {
    const typed = typedCode(error);
    process.stdout.write(`${JSON.stringify({ event: "hardware_operator_error", code: typed ?? "hardware_operator_failed",
      ...(error?.reply ? { reply: error.reply } : {}) })}\n`);
    if (typed === null) process.stderr.write(`${error?.stack ?? error}\n`);
    process.exitCode = 2;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main(process.argv.slice(2));
