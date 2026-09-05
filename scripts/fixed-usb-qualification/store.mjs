import { appendFile, lstat, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { exactObject, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { judgeWindow, validateCycle, validateState } from "./judge.mjs";

async function exists(path) {
  try { await lstat(path); return true; }
  catch (error) { if (error.code === "ENOENT") return false; throw error; }
}
export async function selectedWindow(root) {
  for (let index = 0; index < 3; index += 1) {
    if (!await exists(resolve(root, `window-${index}.result.json`))) return index;
  }
  return 3;
}
export async function windowRecords(root, index) {
  const path = resolve(root, `window-${index}.samples.jsonl`);
  if (!await exists(path)) return [];
  await protectedPath(path);
  return (await readFile(path, "utf8")).trim().split("\n").filter(Boolean).map((line) => JSON.parse(line));
}
export async function recordState(root, context, state) {
  validateState(state, context);
  const index = await selectedWindow(root);
  if (index === 3 || !await exists(resolve(root, `window-${index}.issued.json`))) return { recorded: false };
  const records = await windowRecords(root, index);
  requireCondition(records.length < 512, "sample_bound");
  const path = resolve(root, `window-${index}.samples.jsonl`);
  if (await exists(path)) await protectedPath(path);
  await appendFile(path, `${JSON.stringify({ sequence: records.length + 1, state })}\n`, { mode: 0o600 });
  return { recorded: true, window: index, sequence: records.length + 1 };
}
export async function recordFault(root, body) {
  exactObject(body, ["kind", "running", "visibility", "heartbeatSuppressed", "generation"]);
  const index = await selectedWindow(root);
  requireCondition(body.running === true && (((index === 0 || index === 1) && body.kind === "visibility_hidden" && body.visibility === "hidden") ||
    (index === 2 && body.kind === "heartbeats_suppressed" && body.heartbeatSuppressed === true)), "fault_witness");
  requireCondition(await exists(resolve(root, `window-${index}.consumed.json`)), "window_not_delivered");
  const records = await windowRecords(root, index);
  const preceding = records.at(-1);
  requireCondition(preceding?.state.running === true && preceding.state.qualification?.generation === body.generation &&
    preceding.state.qualification.gate_closed_ms === null && preceding.state.qualification.work_gate_remaining_ms > 3000, "fault_generation_binding");
  const path = resolve(root, `window-${index}.fault.json`);
  if (await exists(path)) return { recorded: false };
  await writeNew(path, { window: index, kind: body.kind, generation: body.generation, after_sequence: preceding.sequence, browser_observation: true });
  return { recorded: true };
}
export async function finishWindow(root, context, index) {
  requireCondition(index === await selectedWindow(root), "window_order");
  const records = await windowRecords(root, index);
  for (const record of records) validateState(record.state, context);
  const faultPath = resolve(root, `window-${index}.fault.json`);
  const fault = await exists(faultPath) ? await readJson(faultPath) : undefined;
  const result = judgeWindow(index, records, fault);
  await writeNew(resolve(root, `window-${index}.result.json`), result);
  return result;
}
export async function recordCycle(root, context, input) {
  requireCondition(Number.isInteger(input?.cycle) && input.cycle >= 1 && input.cycle <= 20, "cycle_number");
  const previous = input.cycle > 1 ? await readJson(resolve(root, `cycle-${input.cycle - 1}.json`)) : undefined;
  const result = validateCycle(input, context, previous);
  await writeNew(resolve(root, `cycle-${input.cycle}.json`), result);
  return { cycle_report_accepted: input.cycle, hardware_execution_claimed_by_supervisor: false };
}

export async function requireCompleteCycles(root, context, browserState, window) {
  let previous;
  for (let cycle = 1; cycle <= 20; cycle += 1) {
    const path = resolve(root, `cycle-${cycle}.json`);
    await protectedPath(path);
    previous = validateCycle(await readJson(path), context, previous);
  }
  requireCondition(browserState?.connected === true && browserState.deviceLeaseInactive === true && browserState.preservation?.mine_on_boot === false &&
    browserState.preservation.baseline_id === previous.baseline_id && browserState.preservation.device_identity_match === true &&
    browserState.preservation.settings_match === true && (window > 0 || browserState.preservation.authorization_high_water_match === true), "live_baseline_missing");
}
