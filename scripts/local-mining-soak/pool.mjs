// Saves the owner's pool as the device's primary pool for local mining (owner decision
// 2026-10-17: save it and keep it). The values come from the ignored pool-credentials file
// at run time and are never printed, recorded or committed.

/** The settings PATCH for a pool read by `readPoolForSigning`: a bare host, as upstream stores it. */
export function poolSettings(pool) {
  const endpoint = new URL(pool.endpoint);
  return { stratumURL: endpoint.hostname, stratumPort: Number(endpoint.port), stratumUser: pool.username,
    stratumPassword: pool.password };
}

/** Whether the device reports the saved pool; the password is not readable, so it is not compared. */
export function poolSaved(info, settings) {
  return info?.stratumURL === settings.stratumURL && Number(info?.stratumPort) === settings.stratumPort &&
    info?.stratumUser === settings.stratumUser;
}
