// Upstream-default soak contract (firmware ADR-0033): one 600-active-second Stratum V1 real-pool soak
// through the Gate, gated by an exact task line in the active soak task.
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { nonce, requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { SOAK_CONTEXT_SCHEMA, SOAK_MAXIMUM_ACTIVE_MS } from "../fixed-usb-qualification/judge.mjs";

export { SOAK_CONTEXT_SCHEMA, SOAK_MAXIMUM_ACTIVE_MS };
export const SOAK_TASK = "task-ultra205-default-profile-soak-reverification";
export const SOAK_TASK_LINE = "Ultra 205 upstream-default soak hardware: enabled.";
export const SOAK_WORK_GATE_MS = 600000;
/** 34 renewals at 20 s cover up to 80 s of preparation plus the 600 s gate. */
export const SOAK_RENEWALS = 34;
export const SOAK_SUGGESTED_DIFFICULTY = 1000;
/** Fixed origin that holds the Ultra 205 Web Serial grant (AGENTS.md, Persistent Gate Browser Tab). */
export const SOAK_PORT = 48765;
/** Idle WebSocket pre-phase proving the Phase 4 hardening before any authorization is signed. */
export const IDLE_PROOF_MS = 60000;
export const IDLE_MINIMUM_HTTP_SAMPLES = 20;

/** The soak task must be active exactly once and carry the exact enable line in its own block. */
export async function requireSoakTask(firmwareRoot) {
  const lines = (await readFile(resolve(firmwareRoot, "TASKS.md"), "utf8")).split(/\r?\n/u);
  let section = "", inBlock = false, count = 0, enabled = false;
  for (const line of lines) {
    if (line.startsWith("## ")) { section = line; inBlock = false; }
    if (line.startsWith("### ")) {
      inBlock = section === "## Active" && line.slice(4).split(/\s/u)[0] === SOAK_TASK;
      if (inBlock) count += 1;
    }
    if (inBlock && line.trim() === SOAK_TASK_LINE) enabled = true;
  }
  requireCondition(count === 1, "soak_task_ambiguous");
  requireCondition(enabled, "soak_task_disabled");
}

export function soakAllowance(ordinal) {
  requireCondition(Number.isSafeInteger(ordinal) && ordinal > 0 && ordinal <= 0xffffffff, "soak_ordinal");
  return { schema: "worker-soak-allowance-v1", id: nonce(), ordinal, maximumActiveMilliseconds: SOAK_MAXIMUM_ACTIVE_MS };
}

/** A device soak ledger that is idle and internally consistent; its next ordinal is the soak to sign. */
export function requireIdleSoakLedger(value) {
  requireCondition(value && value.schema === "worker-soak-ledger-v1" && value.pending === false &&
    Number.isSafeInteger(value.next_ordinal) && value.next_ordinal >= 1 &&
    value.last_completed_ordinal === value.next_ordinal - 1 &&
    value.total_charged_ms === (value.next_ordinal - 1) * SOAK_MAXIMUM_ACTIVE_MS, "soak_ledger_not_idle");
  return value.next_ordinal;
}

/** Idle qualification ledger: a soak must not overlap a pending qualification reservation. */
export function requireQualificationIdle(value) {
  requireCondition(value && value.schema === "worker-qualification-ledger-v1" && value.pending === false, "qualification_ledger_pending");
}
