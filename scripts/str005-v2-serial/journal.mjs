import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import { validateState } from "../fixed-usb-qualification/judge.mjs";
import { requireExhaustedOriginal, requireIdleLedger, validateLedger } from "../fixed-usb-qualification/iterative-contract.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { collectedRecord, validateRecordProgress } from "./device.mjs";
import { parseDeviceRecord } from "./device-record.mjs";
import { check, object, sha256, uint } from "./values.mjs";

export function stateContext(context, phase) {
  check(["before", "candidate"].includes(phase), "v2_phase");
  return { ...context, ...(phase === "before" ? context.before_source : {}) };
}
export function checkedState(state, context, phase) {
  if (state.status === "restoration_pending") {
    check(phase === "candidate" && !state.running, "v2_pending_restoration");
    validateState({ ...state, status: "restoration_unconfirmed" }, stateContext(context, phase));
  } else validateState(state, stateContext(context, phase));
  if (phase === "before" || context.scope === "channel") check(!state.running && !state.heartbeatSuppressed &&
    state.renewalsConfirmed === 0 && !["running", "window_loaded"].includes(state.status), "v2_mining_forbidden");
  return state;
}
export function baseline(state, closed = false) {
  check(state && state.deviceBaselineConfirmed && state.deviceLeaseInactive && !state.running && !state.failure &&
    state.preservation?.device_identity_match && state.preservation.settings_match &&
    state.preservation.authorization_high_water_match && !state.preservation.mine_on_boot, "v2_baseline");
  check(closed ? state.status === "closed" && !state.connected && state.serialOwnershipReleased :
    state.status === "ready" && state.connected && !state.serialOwnershipReleased, "v2_baseline_connection");
}
/** Expected signed-work advancement is proved separately; never rewrite the original comparison. */
export function restoredBaseline(state, context, closed = false) {
  if (context.scope !== "share" || state?.preservation?.authorization_high_water_match === true) return baseline(state, closed);
  const checkpoint = state?.authorizationRecovery;
  check(checkpoint?.matched === true && checkpoint.generation > 0 && checkpoint.generation === state.qualification?.generation,
    "v2_authorization_restoration");
  check(state.deviceBaselineConfirmed && state.deviceLeaseInactive && !state.running && !state.failure &&
    state.preservation.device_identity_match && state.preservation.settings_match && !state.preservation.mine_on_boot, "v2_baseline");
  check(closed ? state.status === "closed" && !state.connected && state.serialOwnershipReleased :
    state.status === "ready" && state.connected && !state.serialOwnershipReleased, "v2_baseline_connection");
}
export function healthy(state) {
  baseline(state);
  const q = state.qualification;
  check(q && q.watchdog_alive && !q.mine_on_boot && q.voltage_fresh && q.power_fresh && q.temperature_fresh && q.fan_fresh,
    "v2_fresh_health");
}
async function rows(root, prefix, maximum) {
  const names = (await readdir(root)).filter((name) => new RegExp(`^${prefix}-[0-9]{4}\\.json$`, "u").test(name)).sort();
  check(names.length <= maximum, "v2_journal_bound");
  const values = [];
  for (const [index, name] of names.entries()) {
    check(name === `${prefix}-${String(index + 1).padStart(4, "0")}.json`, "v2_journal_gap");
    values.push((await proof(root, name)).value);
  }
  return values;
}
export async function readJournal(root, context) {
  const values = await rows(root, "state", 1024);
  for (const [index, row] of values.entries()) {
    object(row, ["schema", "contextSha256", "sequence", "phase", "atHostMs", "state"]);
    check(row.schema === "str005-v2-state-v1" && row.contextSha256 === sha256(JSON.stringify(context)) && row.sequence === index + 1,
      "v2_state_journal_identity");
    uint(row.atHostMs); checkedState(row.state, context, row.phase);
    if (index > 0) check(row.atHostMs >= values[index - 1].atHostMs &&
      !(values[index - 1].phase === "candidate" && row.phase === "before"), "v2_journal_order");
  }
  return values;
}
export async function readDeviceJournal(root, context) {
  const values = await rows(root, "device", 1024);
  for (const [index, row] of values.entries()) {
    object(row, ["schema", "contextSha256", "sequence", "atHostMs", "record"]);
    check(row.schema === "str005-v2-device-record-v1" && row.contextSha256 === sha256(JSON.stringify(context)) && row.sequence === index + 1,
      "v2_device_journal_identity");
    uint(row.atHostMs); parseDeviceRecord(row.record);
    check(row.record.scope === context.scope && row.record.attemptId === context.attemptId, "v2_device_attempt");
    if (index > 0) {
      check(row.atHostMs >= values[index - 1].atHostMs, "v2_journal_order");
      validateRecordProgress(values[index - 1].record, row.record);
    }
  }
  return values;
}

/** One server-owned writer. Hot-path records do not rescan the growing evidence directory. */
export async function createJournal(root, context) {
  const states = await readJournal(root, context), devices = await readDeviceJournal(root, context);
  let stateSequence = states.length, deviceSequence = devices.length;
  let maybeLastState = states.at(-1), maybeLastDevice = devices.at(-1);
  const contextSha256 = sha256(JSON.stringify(context));
  return {
    lastState: () => maybeLastState === undefined ? undefined : structuredClone(maybeLastState),
    lastDevice: () => maybeLastDevice === undefined ? undefined : structuredClone(maybeLastDevice),
    async state(phase, state, now) {
      checkedState(state, context, phase); uint(now);
      check(stateSequence < 1024 && now >= (maybeLastState?.atHostMs ?? 0) &&
        !(maybeLastState?.phase === "candidate" && phase === "before"), "v2_journal_order");
      const row = { schema: "str005-v2-state-v1", contextSha256, sequence: stateSequence + 1, phase, atHostMs: now,
        state: structuredClone(state) };
      await writeNew(resolve(root, `state-${String(row.sequence).padStart(4, "0")}.json`), row);
      maybeLastState = row; stateSequence = row.sequence;
      return { recorded: true, sequence: row.sequence };
    },
    async device(status, now) {
      const record = collectedRecord(status); uint(now);
      check(record.scope === context.scope && record.attemptId === context.attemptId && deviceSequence < 1024 &&
        now >= (maybeLastDevice?.atHostMs ?? 0), "v2_device_attempt");
      if (maybeLastDevice) validateRecordProgress(maybeLastDevice.record, record);
      const row = { schema: "str005-v2-device-record-v1", contextSha256, sequence: deviceSequence + 1, atHostMs: now, record };
      await writeNew(resolve(root, `device-${String(row.sequence).padStart(4, "0")}.json`), row);
      maybeLastDevice = row; deviceSequence = row.sequence;
      return { recorded: true, sequence: row.sequence };
    },
  };
}

export async function saveAccounting(root, context, input, last) {
  object(input, ["stage", "state", "ledger", "original_budget"]);
  check(["before-install", "before", "after"].includes(input.stage), "v2_accounting_stage");
  validateLedger(input.ledger); requireExhaustedOriginal(input.original_budget);
  check(last && equal(last.state, input.state), "v2_accounting_unpublished");
  if (input.stage === "after") restoredBaseline(input.state, context); else baseline(input.state);
  check(last.phase === (input.stage === "before-install" ? "before" : "candidate"), "v2_accounting_phase");
  if (input.stage !== "after") requireIdleLedger(input.ledger, 18, 1560000);
  if (input.stage !== "before-install") {
    const prior = (await proof(root, "accounting-before-install.json")).value;
    check(equal(prior.original_budget, input.original_budget) && prior.state.preservation.baseline_id === input.state.preservation.baseline_id &&
      last.sequence > prior.observedSequence, "v2_accounting_continuity");
    if (input.stage === "before") check(equal(prior.ledger, input.ledger), "v2_accounting_before_changed");
  }
  if (input.stage === "after") {
    const before = (await proof(root, "accounting-before.json")).value;
    const restoration = (await proof(root, "restoration.json")).value;
    check(last.sequence > restoration.observedSequence && restoration.observedSequence > before.observedSequence, "v2_accounting_order");
  }
  // Failed attempts still retain real after-ledger values; only the judge decides whether they pass.
  await writeNew(resolve(root, `accounting-${input.stage}.json`), { schema: "str005-v2-accounting-v1",
    contextSha256: sha256(JSON.stringify(context)), observedSequence: last.sequence, ...input });
  return { accounting_saved: true, stage: input.stage };
}
