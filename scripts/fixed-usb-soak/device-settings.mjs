// The device configuration a soak runs (firmware ADR-0037). The lease carries no hardware
// profile, so the soak writes its contract's settings while the device is idle, reads them
// back, and records both. The read-back must match, or the soak does not start.
import { isIP } from "node:net";
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { PRIVATE_FIRST_OCTETS } from "./observer.mjs";

/** Upstream defaults with the automatic fan (owner decision 2026-10-16). */
export const SOAK_DEVICE_SETTINGS = Object.freeze({ frequency: 485, coreVoltage: 1200, autofanspeed: 1 });
/**
 * Profiles a soak may write, chosen at preflight. `catalog-maximum-frequency` runs the catalog's 575 MHz
 * at 1200 mV: 1300 mV would exceed the 15 W live stop limit (owner decision 2026-10-16).
 */
export const SOAK_PROFILES = Object.freeze({
  "upstream-default": SOAK_DEVICE_SETTINGS,
  "catalog-maximum-frequency": Object.freeze({ frequency: 575, coreVoltage: 1200, autofanspeed: 1 }),
  // A BAP soak changes the manual fan speed; storing it first keeps the settings fingerprint stable.
  "upstream-default-bap": Object.freeze({ frequency: 485, coreVoltage: 1200, autofanspeed: 1, manualFanSpeed: 100 }),
});
export const DEFAULT_SOAK_PROFILE = "upstream-default";

/** The profile a preflight freezes: the named one, or the one a live plan needs, which must agree. */
export function preflightProfile(maybeProfile, maybeRequiredProfile) {
  const profile = maybeProfile ?? maybeRequiredProfile ?? DEFAULT_SOAK_PROFILE;
  profileSettings(profile);
  requireCondition(maybeRequiredProfile === undefined || profile === maybeRequiredProfile, "soak_live_changes_profile");
  return profile;
}

/** The settings a context's profile names; anything else refuses the soak. */
export function profileSettings(name) {
  requireCondition(Object.hasOwn(SOAK_PROFILES, name), "soak_profile_unknown");
  return { ...SOAK_PROFILES[name] };
}
const DEVICE_TIMEOUT_MS = 10000;
/** Read back so evidence records what the automatic fan ran toward. */
const FAN_SETTINGS = Object.freeze(["temptarget", "minFanSpeed"]);

export async function applyDeviceSettings(endpoint, settings = SOAK_DEVICE_SETTINGS, fetchImpl = fetch) {
  requireCondition(isIP(endpoint?.ipv4 ?? "") === 4 && PRIVATE_FIRST_OCTETS(endpoint.ipv4.split(".").map(Number)) &&
    Number.isInteger(endpoint.httpPort) &&
    endpoint.httpPort > 0 && endpoint.httpPort < 65536, "soak_device_endpoint");
  const base = `http://${endpoint.ipv4}:${endpoint.httpPort}`;
  const patch = await fetchImpl(`${base}/api/system`, { method: "PATCH", headers: { "content-type": "application/json" },
    body: JSON.stringify(settings), signal: AbortSignal.timeout(DEVICE_TIMEOUT_MS) });
  requireCondition(patch.ok, "soak_device_settings_patch");
  const response = await fetchImpl(`${base}/api/system/info`, { signal: AbortSignal.timeout(DEVICE_TIMEOUT_MS) });
  requireCondition(response.ok, "soak_device_settings_readback");
  const info = await response.json();
  const observed = Object.fromEntries([...Object.keys(settings), ...FAN_SETTINGS].map((key) => [key, Number(info?.[key])]));
  requireCondition(Object.entries(settings).every(([key, value]) => observed[key] === value), "soak_device_settings_readback");
  return { schema: "fixed-usb-soak-device-settings-v1", requested: { ...settings }, observed };
}
