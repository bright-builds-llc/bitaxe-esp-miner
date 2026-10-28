import { readFile } from "node:fs/promises";

type JsonObject = Record<string, unknown>;

export const restorableKeys = [
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

/** The restoration step that failed. */
export type RestorationStep = "input" | "mutation_transport" | "mutation_status" | "confirmation" | "mismatch" | "final_state";

const finalChecks = [
  "startMiningOnBoot", "ssid", "stratumURL", "stratumPort", "stratumUser", "useFallbackStratum",
  "fallbackStratumURL", "theme",
] as const;

/**
 * A failure's detail, closed by type so it never carries a value: key and check names, the routes
 * the restore uses, HTTP statuses, which input was invalid, and whether a device body was invalid.
 */
export type RestorationDetail = {
  readonly keys?: readonly (typeof restorableKeys)[number][];
  readonly checks?: readonly (typeof finalChecks)[number][];
  readonly route?: "/api/system" | "/api/theme" | "/api/system/info";
  readonly status?: number | null;
  readonly settings?: number;
  readonly theme?: number;
  readonly input?: "backup" | "credentials";
  readonly body?: "invalid";
};

export class RestorationFailure extends Error {
  constructor(readonly step: RestorationStep, readonly detail: RestorationDetail = {}) {
    super(step === "mismatch" ? "restoration mismatch"
      : step === "final_state" ? "restoration final state mismatch"
        : step === "input" ? "restoration input is invalid"
          : step === "confirmation" ? "restoration confirmation failed"
            : "restoration mutation failed");
    this.name = "RestorationFailure";
  }
}

/** The value-free record of a failed restoration; any other error reads as `unclassified`. */
export function restorationDiagnostic(error: unknown): JsonObject {
  if (error instanceof RestorationFailure) return { step: error.step, ...error.detail };
  return { step: "unclassified" };
}

export function validateRestoredSettingsAndTheme(
  confirmed: JsonObject,
  confirmedTheme: JsonObject,
  backup: JsonObject,
  wifi: JsonObject,
  pool: JsonObject,
): void {
  const settings = object(backup["settings"], "backup");
  const theme = object(backup["theme"], "backup");
  const mismatched = restorableKeys.filter(key => Object.hasOwn(settings, key)
    && JSON.stringify(confirmed[key]) !== JSON.stringify(settings[key]));
  if (mismatched.length > 0) throw new RestorationFailure("mismatch", { keys: mismatched });
  const checks: Readonly<Record<(typeof finalChecks)[number], boolean>> = {
    startMiningOnBoot: confirmed["startMiningOnBoot"] === false,
    ssid: confirmed["ssid"] === wifi["ssid"],
    stratumURL: confirmed["stratumURL"] === pool["poolURL"],
    stratumPort: confirmed["stratumPort"] === pool["poolPort"],
    stratumUser: confirmed["stratumUser"] === pool["poolUser"],
    useFallbackStratum: confirmed["useFallbackStratum"] !== true,
    fallbackStratumURL: !(typeof confirmed["fallbackStratumURL"] === "string"
      && confirmed["fallbackStratumURL"].length > 0),
    theme: JSON.stringify(confirmedTheme) === JSON.stringify(theme),
  };
  const failed = finalChecks.filter(name => !checks[name]);
  if (failed.length > 0) throw new RestorationFailure("final_state", { checks: failed });
}

function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function object(value: unknown, input: "backup" | "credentials"): JsonObject {
  if (!isObject(value)) throw new RestorationFailure("input", { input });
  return value;
}

async function readCredentials(file: string): Promise<JsonObject> {
  let value: unknown;
  try {
    value = JSON.parse(await readFile(file, "utf8"));
  } catch {
    throw new RestorationFailure("input", { input: "credentials" });
  }
  return object(value, "credentials");
}

async function fetchObject(origin: URL, route: "/api/system/info" | "/api/theme"): Promise<JsonObject> {
  let response: Response;
  try {
    response = await fetch(new URL(route, origin));
  } catch {
    throw new RestorationFailure("confirmation", { route, status: null });
  }
  if (!response.ok) throw new RestorationFailure("confirmation", { route, status: response.status });
  let value: unknown;
  try {
    value = await response.json();
  } catch {
    throw new RestorationFailure("confirmation", { route, status: response.status, body: "invalid" });
  }
  if (!isObject(value)) throw new RestorationFailure("confirmation", { route, status: response.status, body: "invalid" });
  return value;
}

async function send(origin: URL, route: "/api/system" | "/api/theme", method: string, body: JsonObject): Promise<Response> {
  try {
    return await fetch(new URL(route, origin), {
      method,
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
  } catch {
    throw new RestorationFailure("mutation_transport", { route });
  }
}

export async function restoreSelfTestSettings(
  origin: URL,
  backup: JsonObject,
  wifiPath: string,
  poolPath: string,
): Promise<void> {
  const settings = object(backup["settings"], "backup");
  const theme = object(backup["theme"], "backup");
  const wifi = await readCredentials(wifiPath);
  const pool = await readCredentials(poolPath);
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
  const response = await send(origin, "/api/system", "PATCH", body);
  const themeResponse = await send(origin, "/api/theme", "POST", theme);
  if (!response.ok || !themeResponse.ok) {
    throw new RestorationFailure("mutation_status", { settings: response.status, theme: themeResponse.status });
  }
  const confirmed = await fetchObject(origin, "/api/system/info");
  const confirmedTheme = await fetchObject(origin, "/api/theme");
  validateRestoredSettingsAndTheme(confirmed, confirmedTheme, backup, wifi, pool);
}
