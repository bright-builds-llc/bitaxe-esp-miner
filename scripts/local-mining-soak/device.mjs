// The local mining soak's HTTP client for the station endpoint handed off in the same
// session. It only reads system info, saves the soak's pool and sends pause or resume.
import { isIP } from "node:net";
import { requireCondition } from "../fixed-usb-qualification/contract.mjs";
import { PRIVATE_FIRST_OCTETS } from "../fixed-usb-soak/observer.mjs";

const DEVICE_TIMEOUT_MS = 10_000;
/** The retained log is up to 512 KiB. */
const LOG_TIMEOUT_MS = 30_000;

/** A client bound to one private-LAN endpoint; anything else refuses the soak. */
export function deviceClient(endpoint, fetchImpl = fetch) {
  requireCondition(isIP(endpoint?.ipv4 ?? "") === 4 && PRIVATE_FIRST_OCTETS(endpoint.ipv4.split(".").map(Number)) &&
    Number.isInteger(endpoint.httpPort) && endpoint.httpPort > 0 && endpoint.httpPort < 65536, "soak_device_endpoint");
  const base = `http://${endpoint.ipv4}:${endpoint.httpPort}`;
  const call = (path, init = {}) => fetchImpl(`${base}${path}`, { ...init, signal: AbortSignal.timeout(DEVICE_TIMEOUT_MS) });
  return {
    async info() {
      const response = await call("/api/system/info");
      requireCondition(response.ok, "soak_device_info");
      return response.json();
    },
    async patch(settings) {
      const response = await call("/api/system", { method: "PATCH", headers: { "content-type": "application/json" },
        body: JSON.stringify(settings) });
      requireCondition(response.ok, "soak_device_settings_patch");
    },
    /** The retained log, read once per break; it can hold the station address, so callers keep it private. */
    async logs() {
      const response = await fetchImpl(`${base}/api/system/logs`, { signal: AbortSignal.timeout(LOG_TIMEOUT_MS) });
      requireCondition(response.ok, "soak_device_logs");
      return response.text();
    },
    async command(name) {
      requireCondition(name === "pause" || name === "resume", "soak_device_command");
      const response = await call(`/api/system/${name}`, { method: "POST" });
      requireCondition(response.ok, `soak_device_${name}`);
    },
  };
}
