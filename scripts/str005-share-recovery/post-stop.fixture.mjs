/** Recovery002 post-Stop state shape. Identity/baseline identifiers are synthetic.
 * This fixture records the baseline_confirmed boundary, not hardware qualification.
 */
export function postStopState(context, closed = false) {
  return { schema: 'worker-serial-acceptance-v1', gateCommit: context.gate_commit,
    status: closed ? 'closed' : 'baseline_confirmed', connected: !closed, running: false,
    heartbeatSuppressed: false, renewalsConfirmed: 0, serialOwnershipReleased: closed,
    helloRecovery: { discardedRecords: 0, discardedBytes: 0, discardedReplies: 0 },
    deviceRestorationConfirmed: true, deviceBaselineConfirmed: true, deviceLeaseInactive: true,
    expectedFirmwareSourceCommit: context.firmware_commit, expectedAppElfSha256: context.app_elf_sha256,
    qualification: { accepted: 0, active_limit_ms: null, active_ms: 0, budget_complete: true, budget_reserved_ms: 240000,
      chip_temp_celsius: 29, fan_fresh: true, fan_rpm: 7725, gate_closed_ms: null, generation: 0,
      generation_elapsed_ms: 0, last_valid_heartbeat_ms: 3902188, mine_on_boot: false, nonce_work_correlations: 0,
      power_fresh: true, power_watts: 0.78, rejected: 0, revocation_reason: 'none', safe_stop_complete: false,
      safe_stop_stage: 'not_started', schema: 'worker-qualification-v1', shutdown_budget_ms: 0,
      shutdown_started_ms: null, submitted: 0, temperature_fresh: true, voltage_fresh: true, voltage_volts: 5.46,
      watchdog_alive: true, work_dispatched: 0, work_gate_remaining_ms: null },
    preservation: { schema: 'worker-preservation-continuity-v1', baseline_id: Buffer.alloc(16, 4).toString('base64url'),
      settings_match: true, authorization_high_water_match: true, device_identity_match: true, mine_on_boot: false } };
}
