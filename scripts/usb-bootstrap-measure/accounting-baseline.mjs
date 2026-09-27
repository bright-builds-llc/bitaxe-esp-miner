/** Version-4 accounting safety predicate; raw page status is never rewritten. */
export function accountingBaselineAllowed(state, stage, maybeBaselineId) {
  if (!["before", "after"].includes(stage) || !state) return false;
  const preservation = state.preservation;
  if (state.connected !== true || state.serialOwnershipReleased !== false || state.running !== false || state.failure !== undefined ||
    state.deviceBaselineConfirmed !== true || state.deviceLeaseInactive !== true || state.heartbeatSuppressed !== false || state.renewalsConfirmed !== 0 ||
    preservation?.device_identity_match !== true || preservation.settings_match !== true || preservation.authorization_high_water_match !== true || preservation.mine_on_boot !== false ||
    typeof preservation.baseline_id !== "string" || (maybeBaselineId !== undefined && preservation.baseline_id !== maybeBaselineId)) return false;
  if (stage === "before") return state.status === "ready";
  return state.deviceRestorationConfirmed === true && ["ready", "baseline_confirmed"].includes(state.status);
}
