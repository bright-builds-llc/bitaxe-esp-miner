import { readFile } from "node:fs/promises";

import type { JsonObject } from "./stratum-v2-campaign-preflight.js";

export async function validateRestorableInputs(
  settings: JsonObject,
  wifiPath: string,
  poolPath: string,
  fail: (category: string, message: string) => never,
): Promise<void> {
  const parse = async (candidate: string): Promise<JsonObject> => {
    const value: unknown = JSON.parse(await readFile(candidate, "utf8"));
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
      fail("hardware_blocked", "local restoration input is malformed");
    }
    return value as JsonObject;
  };
  const wifi = await parse(wifiPath);
  const pool = await parse(poolPath);
  if (settings["startMiningOnBoot"] !== false
    || settings["ssid"] !== wifi["ssid"]
    || settings["stratumURL"] !== pool["poolURL"]
    || settings["stratumPort"] !== pool["poolPort"]
    || settings["stratumUser"] !== pool["poolUser"]
    || settings["useFallbackStratum"] === true
    || (typeof settings["fallbackStratumURL"] === "string"
      && settings["fallbackStratumURL"].length > 0)) {
    fail("hardware_blocked", "local inputs cannot construct exact restoration");
  }
}
