// Terminal soak judgement from Gate records, device ledgers, the released final state and the Rust window
// judge. Every failure is a closed category; nothing here reads pool, endpoint or credential values.
import { requireReleasedState } from "../fixed-usb-qualification/iterative-preflight.mjs";
import { SOAK_MAXIMUM_ACTIVE_MS, SOAK_RENEWALS, SOAK_WORK_GATE_MS } from "./contract.mjs";

/** The soak's live stop limits on every fresh running sample (the same bounds the firmware revokes on). */
export function unsafeSample(q) {
  return (q.voltage_fresh && !(q.voltage_volts >= 4.5 && q.voltage_volts <= 5.5)) ||
    (q.power_fresh && !(q.power_watts >= 0 && q.power_watts <= 15)) ||
    (q.temperature_fresh && !(q.chip_temp_celsius < 75)) ||
    (q.fan_fresh && !(q.fan_rpm > 0)) || !q.watchdog_alive;
}

/** Gate status observations that map device active time onto host time for the window judge. */
export function clockObservations(records) {
  return records
    .filter((record) => record.state.running && record.state.qualification?.soak && record.state.qualification.active_ms > 0)
    .map((record) => ({ observedUnixMs: record.receivedAtUnixMs, activeMs: record.state.qualification.active_ms }));
}

function terminalFailures(records, ordinal) {
  const soakSamples = records.map((record) => record.state.qualification).filter((q) => q?.soak?.ordinal === ordinal);
  const last = soakSamples.at(-1);
  if (!last) return ["soak_never_observed"];
  const failures = [];
  const check = (failed, category) => { if (failed) failures.push(category); };
  check(last.revocation_reason !== "lease_or_budget_expired", "soak_not_ended_by_budget");
  check(!last.safe_stop_complete || last.safe_stop_stage !== "fan_paused", "safe_stop_unconfirmed");
  check(last.active_limit_ms !== SOAK_MAXIMUM_ACTIVE_MS || last.budget_reserved_ms !== SOAK_MAXIMUM_ACTIVE_MS, "soak_budget_mismatch");
  check(!(last.active_ms >= SOAK_WORK_GATE_MS && last.active_ms <= SOAK_MAXIMUM_ACTIVE_MS), "soak_active_time");
  check(last.accepted < 1, "no_accepted_share");
  check(last.work_dispatched < 1 || last.nonce_work_correlations < 1, "no_correlated_work");
  check(last.mine_on_boot, "mine_on_boot_enabled");
  check(soakSamples.some((q) => q.revocation_reason !== "none" && q.revocation_reason !== "lease_or_budget_expired"), "soak_revoked");
  check(records.some((record) => record.state.running && record.state.qualification && unsafeSample(record.state.qualification)), "unsafe_sample");
  check(records.some((record) => record.state.renewalsConfirmed > SOAK_RENEWALS), "renewal_bound");
  check(records.some((record) => record.state.failure !== undefined), "browser_failure");
  return failures;
}

/**
 * Judge one completed soak. `windowJudgement` is the parsed `soak-judge` output, or null when it could not
 * run. The result is `passed` only when every terminal, ledger, cleanup and window criterion holds.
 */
export function judgeSoak({ context, records, issuance, ledgerAfter, finalState, windowJudgement, observerClean }) {
  const failures = [];
  const check = (failed, category) => { if (failed) failures.push(category); };
  let cleanupConfirmed = true;
  try { requireReleasedState(finalState, context); } catch { cleanupConfirmed = false; }
  check(!cleanupConfirmed, "cleanup_unconfirmed");
  const before = issuance.ledger_before;
  check(!(ledgerAfter?.schema === "worker-soak-ledger-v1" && ledgerAfter.pending === false && ledgerAfter.next_ordinal === issuance.ordinal + 1 &&
    ledgerAfter.total_charged_ms === before.total_charged_ms + SOAK_MAXIMUM_ACTIVE_MS), "soak_ledger_not_charged_once");
  failures.push(...terminalFailures(records, issuance.ordinal));
  check(windowJudgement === null, "window_judge_unavailable");
  check(!observerClean, "observer_unclean");
  if (windowJudgement && !windowJudgement.passed) failures.push(...windowJudgement.failures.map((failure) => `window_${failure}`));
  return { schema: "fixed-usb-soak-result-v1", result: failures.length === 0 ? "passed" : "unverified", ordinal: issuance.ordinal,
    cumulative_charged_ms: ledgerAfter?.total_charged_ms ?? null, cleanup_confirmed: cleanupConfirmed, failures,
    window_judgement: windowJudgement, parity_promotion: false };
}
