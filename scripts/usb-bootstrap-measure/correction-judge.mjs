/** Inputs are independently parsed/verified evidence, never caller approval flags. */
export function correctionJudgment(result, host, device, finalState) {
  const events = new Map((host?.events ?? []).map(row => [row.stage, row.elapsedUs]));
  const opened = events.get("reader_opened"), reset = events.get("reset_command_call_start"), bootstrap = device?.firstBootstrap;
  const checks = {
    captureQualified: result.capture.qualified === true,
    hostTimingComplete: host?.schema === "bootstrap-host-timing-v2" && host.earliestFailure === null && host.captureComplete === true && host.cleanupComplete === true && host.captureDurationMs === 30000 &&
      host.readerJoined === true && host.quarantineReleased === true && host.readerOpenCount === 1 && host.readerReopenCount === 0 && !host.captureOverflow && !host.overflow && !host.clockDiscontinuity,
    readerHeadroom: Number.isSafeInteger(opened) && Number.isSafeInteger(reset) && opened >= reset && opened - reset <= 1500000,
    nativeBootstrapComplete: device?.complete === true && bootstrap?.outcome === "completed" && bootstrap.stage === "completed" && bootstrap.flags === 0 &&
      Number.isSafeInteger(bootstrap.start_ms) && Number.isSafeInteger(bootstrap.end_ms) && bootstrap.end_ms >= bootstrap.start_ms && bootstrap.end_ms - bootstrap.start_ms <= 2000,
    zeroTxFailures: device?.complete === true && device.counters.actualFailures === 0 && device.firstFailure === null && !device.legacyFailureObserved && device.issues.length === 0 && bootstrap?.flags === 0,
    preservationAndAccounting: result.restoration.confirmed === true && finalState?.deviceRestorationConfirmed === true && finalState.deviceLeaseInactive === true &&
      finalState.preservation?.device_identity_match === true && finalState.preservation.settings_match === true && finalState.preservation.authorization_high_water_match === true && finalState.preservation.mine_on_boot === false,
    cleanupComplete: result.cleanup.complete === true,
  };
  return { accepted: result.status === "measurement_complete" && Object.values(checks).every(Boolean), checks };
}
