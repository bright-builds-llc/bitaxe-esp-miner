import assert from "node:assert/strict";
import { mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  RestorationFailure,
  restorationDiagnostic,
  restoreSelfTestSettings,
  validateRestoredSettingsAndTheme,
} from "./self-test-campaign-restoration.js";

const wifi = { ssid: "private-ssid" };
const pool = { poolURL: "private.invalid", poolPort: 3333, poolUser: "private-user" };
const backup = { settings: { frequency: 525, display: "SSD1306 (128x32)" }, theme: { colorScheme: "dark" } };
const restored = {
  frequency: 525, display: "SSD1306 (128x32)", startMiningOnBoot: false, ssid: "private-ssid",
  stratumURL: "private.invalid", stratumPort: 3333, stratumUser: "private-user",
  useFallbackStratum: false, fallbackStratumURL: "",
};

function diagnosticOf(action: () => void) {
  try {
    action();
  } catch (error) {
    return restorationDiagnostic(error);
  }
  return null;
}

test("a restored device matching its backup validates", () => {
  // Act / Assert
  assert.doesNotThrow(() => validateRestoredSettingsAndTheme(restored, backup.theme, backup, wifi, pool));
});

test("a mismatch names its keys and none of their values", () => {
  // Act
  const diagnostic = diagnosticOf(() => validateRestoredSettingsAndTheme(
    { ...restored, display: "other" }, backup.theme, backup, wifi, pool,
  ));
  // Assert
  assert.deepEqual(diagnostic, { step: "mismatch", keys: ["display"] });
});

test("a final-state failure names its checks and none of their values", () => {
  // Act
  const diagnostic = diagnosticOf(() => validateRestoredSettingsAndTheme(
    { ...restored, startMiningOnBoot: true, fallbackStratumURL: "private.fallback" },
    backup.theme, backup, wifi, pool,
  ));
  // Assert
  assert.deepEqual(diagnostic, { step: "final_state", checks: ["startMiningOnBoot", "fallbackStratumURL"] });
});

test("a failure keeps its original message for the campaign's callers", () => {
  // Act
  const failure = new RestorationFailure("mismatch", { keys: ["frequency"] });
  // Assert
  assert.equal(failure.message, "restoration mismatch");
});

test("an unrecognized error reads as unclassified", () => {
  // Act / Assert
  assert.deepEqual(restorationDiagnostic(new Error("private.invalid")), { step: "unclassified" });
});

/** Credential files with private values, so a test can check none reaches a diagnostic. */
async function credentials(wifiDocument = JSON.stringify({ ssid: "private-ssid", wifiPass: "private-pass" })) {
  const root = await mkdtemp(path.join(os.tmpdir(), "bitaxe-restoration-"));
  const wifiPath = path.join(root, "wifi.json");
  const poolPath = path.join(root, "pool.json");
  await writeFile(wifiPath, wifiDocument, { mode: 0o600 });
  await writeFile(poolPath, JSON.stringify({ ...pool, poolPassword: "private-password" }), { mode: 0o600 });
  return { wifiPath, poolPath };
}

/** Runs a restore against `respond` and returns its diagnostic. */
async function restoreDiagnostic(respond: (route: string, method: string) => Response | Promise<Response>, wifiDocument?: string) {
  const { wifiPath, poolPath } = await credentials(wifiDocument);
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async (input, init) => {
    const url = new URL(typeof input === "string" ? input : input.toString());
    return respond(url.pathname, init?.method ?? "GET");
  };
  try {
    await restoreSelfTestSettings(new URL("http://device.invalid"), backup, wifiPath, poolPath);
    return null;
  } catch (error) {
    return restorationDiagnostic(error);
  } finally {
    globalThis.fetch = originalFetch;
  }
}

test("a refused write records both statuses", async () => {
  // Act
  const diagnostic = await restoreDiagnostic((_route, method) =>
    new Response("", { status: method === "PATCH" ? 409 : 200 }));
  // Assert
  assert.deepEqual(diagnostic, { step: "mutation_status", settings: 409, theme: 200 });
});

test("an unreachable device records the write's route", async () => {
  // Act
  const diagnostic = await restoreDiagnostic(() => { throw new TypeError("fetch failed"); });
  // Assert
  assert.deepEqual(diagnostic, { step: "mutation_transport", route: "/api/system" });
});

test("a refused read-back records its route and status", async () => {
  // Act
  const diagnostic = await restoreDiagnostic((_route, method) =>
    new Response("", { status: method === "GET" ? 503 : 200 }));
  // Assert
  assert.deepEqual(diagnostic, { step: "confirmation", route: "/api/system/info", status: 503 });
});

test("an unparsable read-back is a confirmation failure with an invalid body", async () => {
  // Act
  const diagnostic = await restoreDiagnostic((_route, method) =>
    new Response(method === "GET" ? "{truncated" : "", { status: 200 }));
  // Assert
  assert.deepEqual(diagnostic, { step: "confirmation", route: "/api/system/info", status: 200, body: "invalid" });
});

test("unreadable credentials are an input failure that names no value", async () => {
  // Act
  const diagnostic = await restoreDiagnostic(() => new Response("", { status: 200 }), "{\"ssid\": \"private-ssid\"");
  // Assert
  assert.deepEqual(diagnostic, { step: "input", input: "credentials" });
});

test("a restore's diagnostic never carries a private value", async () => {
  // Act
  const diagnostic = await restoreDiagnostic((route, method) => method === "GET"
    ? Response.json(route === "/api/theme" ? backup.theme : { ...restored, ssid: "other", stratumUser: "private-other" })
    : new Response("", { status: 200 }));
  // Assert
  assert.deepEqual(diagnostic, { step: "final_state", checks: ["ssid", "stratumUser"] });
  assert.equal(/private|other/u.test(JSON.stringify(diagnostic)), false);
});
