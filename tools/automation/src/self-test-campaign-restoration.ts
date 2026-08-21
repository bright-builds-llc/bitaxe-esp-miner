import { readFile } from "node:fs/promises";

type JsonObject = Record<string, unknown>;

const restorableKeys = [
  "hostname", "stratumProtocol", "stratumURL", "stratumPort", "stratumUser",
  "stratumSuggestedDifficulty", "stratumExtranonceSubscribe", "stratumTLS", "stratumCert",
  "stratumV2ChannelType", "stratumV2AuthorityPubkey", "stratumDecodeCoinbase",
  "fallbackStratumProtocol", "fallbackStratumURL", "fallbackStratumPort", "fallbackStratumUser",
  "fallbackStratumSuggestedDifficulty", "fallbackStratumExtranonceSubscribe", "fallbackStratumTLS",
  "fallbackStratumCert", "fallbackStratumV2ChannelType", "fallbackStratumV2AuthorityPubkey",
  "fallbackStratumDecodeCoinbase", "useFallbackStratum", "frequency", "coreVoltage",
  "overclockEnabled", "display", "rotation", "invertscreen", "displayOffset", "displayTimeout",
  "autofanspeed", "manualFanSpeed", "minFanSpeed", "temptarget", "overheat_mode", "statsFrequency",
] as const;

function object(value: unknown): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("restoration input is invalid");
  }
  return value as JsonObject;
}

async function fetchObject(origin: URL, route: string): Promise<JsonObject> {
  const response = await fetch(new URL(route, origin));
  if (!response.ok) throw new Error("restoration confirmation failed");
  return object(await response.json());
}

export async function restoreSelfTestSettings(
  origin: URL,
  backup: JsonObject,
  wifiPath: string,
  poolPath: string,
): Promise<void> {
  const settings = object(backup["settings"]);
  const theme = object(backup["theme"]);
  const wifi = object(JSON.parse(await readFile(wifiPath, "utf8")));
  const pool = object(JSON.parse(await readFile(poolPath, "utf8")));
  const body: JsonObject = { startMiningOnBoot: false };
  for (const key of restorableKeys) {
    if (Object.hasOwn(settings, key)) body[key] = settings[key];
  }
  body["ssid"] = wifi["ssid"];
  body["wifiPass"] = wifi["wifiPass"];
  body["stratumURL"] = pool["poolURL"];
  body["stratumPort"] = pool["poolPort"];
  body["stratumUser"] = pool["poolUser"];
  body["stratumPassword"] = pool["poolPassword"];
  const response = await fetch(new URL("/api/system", origin), {
    method: "PATCH",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  const themeResponse = await fetch(new URL("/api/theme", origin), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(theme),
  });
  if (!response.ok || !themeResponse.ok) throw new Error("restoration mutation failed");
  const confirmed = await fetchObject(origin, "/api/system/info");
  const confirmedTheme = await fetchObject(origin, "/api/theme");
  for (const key of restorableKeys) {
    if (Object.hasOwn(settings, key)
      && JSON.stringify(confirmed[key]) !== JSON.stringify(settings[key])) {
      throw new Error("restoration mismatch");
    }
  }
  if (confirmed["startMiningOnBoot"] !== false
    || JSON.stringify(confirmedTheme) !== JSON.stringify(theme)) {
    throw new Error("restoration final state mismatch");
  }
}
