// BAP settings during a soak's lease (task-parity-bap-settings-effects). The soak drives the
// device's settings through the accessory port with the repo's `bap-client`, as an accessory
// would, and judges from the observer journal that each saved setting took effect live. The plan
// ends on the starting profile, so the settings fingerprint equals the page's baseline.
import { execFile } from "node:child_process";
import { promisify } from "node:util";

/** The profile a BAP soak starts and ends on: the upstream default with the manual fan stored. */
export const BAP_PROFILE = "upstream-default-bap";

/** `expected` is the acknowledgement value; `null` means the reference sends none (`fan_speed`). */
const set = (parameter, value, expected) => Object.freeze({ op: "set", parameter, value, expected });
const request = (parameter, token, value) => Object.freeze({ op: "request", parameter, token, value });

/**
 * Lower frequency before voltage and raise voltage before frequency, so the saved pair never
 * passes through 485 MHz at 1100 mV. Every pair already has hardware evidence.
 */
export const SOAK_BAP_CHANGES = Object.freeze([
  Object.freeze({ atActiveMs: 100000, ops: Object.freeze([
    set("show_new_block", "1", "1"), request("show_new_block", "show_new_block", "1"),
    set("show_new_block", "0", "0"), request("show_new_block", "show_new_block", "0"),
    set("found_block", "0", "0"), request("found_block", "block_found", "0"),
  ]) }),
  Object.freeze({ atActiveMs: 180000, ops: Object.freeze([set("frequency", "400", "400.00"), set("asic_voltage", "1100", "1100")]),
    settings: Object.freeze({ frequency: 400, coreVoltage: 1100, autofanspeed: 1, manualFanSpeed: 100 }) }),
  Object.freeze({ atActiveMs: 280000, ops: Object.freeze([set("fan_speed", "80", null)]),
    settings: Object.freeze({ frequency: 400, coreVoltage: 1100, autofanspeed: 0, manualFanSpeed: 80 }) }),
  Object.freeze({ atActiveMs: 420000, ops: Object.freeze([
    set("asic_voltage", "1200", "1200"), set("frequency", "485", "485.00"),
    set("fan_speed", "100", null), set("auto_fan", "1", "auto_fan_speed_set"),
  ]), settings: Object.freeze({ frequency: 485, coreVoltage: 1200, autofanspeed: 1, manualFanSpeed: 100 }) }),
]);

const ASIC_SETTINGS = new Set(["frequency", "asic_voltage"]);
/** Longer than the client's 45 s settings window plus its own start-up. */
export const CLIENT_TIMEOUT_MS = 60000;
/** The Gate records the device about once a second; an older record no longer proves the gate open. */
const RECORD_FRESH_MS = 5000;

/** Whether one operation's printed frames and outcome show exactly what the plan expects. */
export function bapOperationPassed(op, maybeSummary, frames) {
  if (op.op === "request") {
    return maybeSummary?.outcome === "answered" &&
      frames.some((frame) => frame.command === "RES" && frame.parameter === op.token && frame.value === op.value);
  }
  if (op.expected === null) return maybeSummary?.outcome === "sent_silently";
  return maybeSummary?.outcome === "confirmed" &&
    frames.some((frame) => frame.command === "ACK" && frame.parameter === op.parameter && frame.value === op.expected);
}

function parseLines(stdout) {
  return stdout.split("\n").filter(Boolean).map((line) => {
    try { return JSON.parse(line); } catch { return null; }
  });
}

/** Runs one operation through the frozen `bap-client`; its frames carry no secret for these settings. */
export async function runBapOperation(binary, port, op, run = promisify(execFile)) {
  const args = ["--port", port, op.op, op.parameter];
  if (op.op === "set") {
    args.push("--value", op.value, "--confirm", op.parameter, ...(ASIC_SETTINGS.has(op.parameter) ? ["--allow-asic-setting"] : []));
  }
  let stdout;
  try {
    ({ stdout } = await run(binary, args, { env: {}, timeout: CLIENT_TIMEOUT_MS, maxBuffer: 65536 }));
  } catch (error) {
    // The client exits 2 for a refused or unanswered operation and still prints its frames.
    stdout = String(error?.stdout ?? "");
  }
  const lines = parseLines(stdout);
  const maybeSummary = lines.at(-1) ?? undefined;
  const frames = lines.slice(0, -1).filter(Boolean);
  return { op: op.op, parameter: op.parameter, outcome: maybeSummary?.outcome ?? "unparseable",
    error_code: maybeSummary?.error_code ?? null, frames, passed: bapOperationPassed(op, maybeSummary, frames) };
}

/**
 * Whether the newest Gate record still lets a BAP frame go out: the soak mines unrevoked, the record
 * is fresh, and the work gate has room for a whole client run. Checked before every operation, since
 * a change with a silent `fan_speed` takes most of a minute.
 */
export function gateAllowsOperation(maybeRecord, nowUnixMs) {
  const q = maybeRecord?.state?.qualification;
  return Boolean(maybeRecord?.state?.running && q?.soak && q.revocation_reason === "none" &&
    nowUnixMs - maybeRecord.receivedAtUnixMs <= RECORD_FRESH_MS &&
    Number.isSafeInteger(q.work_gate_remaining_ms) && q.work_gate_remaining_ms >= CLIENT_TIMEOUT_MS);
}

/** Runs a change's operations in order and stops at the first that fails or the gate refuses. */
export async function runBapChange(binary, port, ops, { runOperation = runBapOperation, allowed = () => true } = {}) {
  const results = [];
  for (const op of ops) {
    if (!allowed()) return { results, maybeFailure: "bap_gate_closed" };
    const result = await runOperation(binary, port, op);
    results.push(result);
    if (!result.passed) return { results, maybeFailure: "bap_operation_failed" };
  }
  return { results, maybeFailure: undefined };
}
