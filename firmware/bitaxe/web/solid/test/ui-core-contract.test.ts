// One contract, two implementations: every case runs against the handwritten
// `current` core (`ui-core.js`) and the SolidJS core (`src/core/ui-core.ts`).
import assert from "node:assert/strict";
import test from "node:test";

import * as solidCore from "../src/core/ui-core.js";
import { evaluateCurrentScript, plain } from "./support.js";

type UiCore = Pick<
  typeof solidCore,
  | "buildSettingsPatch"
  | "formatMetric"
  | "isKnownRoute"
  | "normalizePath"
  | "patchFieldNames"
  | "patchSummary"
  | "publicError"
  | "routeFor"
  | "scoreboardRows"
  | "themeFromPayload"
  | "themePayload"
>;

async function implementations(): Promise<ReadonlyArray<readonly [string, UiCore]>> {
  const current = (await evaluateCurrentScript("ui-core.js", "BitaxeUiCore")) as UiCore;
  return [["current", current], ["solid", solidCore]];
}

const scoreboardEntry = { difficulty: 42.5, job_id: "job-a", extranonce2: "0001", ntime: 1, nonce: "1234ABCD", version_bits: "20000000" };

test("both cores map the same history routes and fall back to the dashboard", async () => {
  for (const [name, core] of await implementations()) {
    // Arrange
    const paths = ["/", "/network/", "/ap", "/system", "/design?x=1", "/logs#tail", "/not-admitted", 7];

    // Act
    const routes = paths.map((value) => core.routeFor(value as string));

    // Assert
    assert.deepEqual(routes, ["dashboard", "network", "network", "dashboard", "theme", "logs", "dashboard", "dashboard"], name);
    assert.equal(core.isKnownRoute("/scoreboard/"), true, name);
    assert.equal(core.isKnownRoute("/not-admitted"), false, name);
    assert.equal(core.normalizePath("/pool/"), "/pool", name);
  }
});

test("both cores build write-only patches that drop blanks, unknown fields and unsafe numbers", async () => {
  for (const [name, core] of await implementations()) {
    // Arrange
    const poolForm = {
      stratumProtocol: "SV1",
      stratumURL: " synthetic.pool.invalid ",
      stratumPort: "3333",
      stratumUser: "synthetic.worker",
      stratumPassword: "synthetic-password",
      ignored: "not-admitted",
    };

    // Act
    const network = core.buildSettingsPatch("network", { hostname: "  miner  ", ssid: "", wifiPass: "" });
    const pool = core.buildSettingsPatch("pool", poolForm);
    const unsafe = core.buildSettingsPatch("settings", { statsFrequency: "1e400" });
    const unknown = core.buildSettingsPatch("frequency", { frequency: "485" });

    // Assert
    assert.deepEqual(plain(network), { hostname: "miner" }, name);
    assert.deepEqual(plain(pool), {
      stratumProtocol: "SV1",
      stratumURL: "synthetic.pool.invalid",
      stratumPort: 3333,
      stratumUser: "synthetic.worker",
      stratumPassword: "synthetic-password",
    }, name);
    assert.deepEqual(plain(unsafe), {}, name);
    assert.deepEqual(plain(unknown), {}, name);
  }
});

test("both cores summarize patches without revealing secrets", async () => {
  for (const [name, core] of await implementations()) {
    // Arrange
    const patch = core.buildSettingsPatch("network", { hostname: "miner", wifiPass: "synthetic-secret" });

    // Act
    const summary = plain(core.patchSummary("network", patch));

    // Assert
    assert.deepEqual(summary, ["hostname", "wifiPass:updated"], name);
    assert.doesNotMatch(JSON.stringify(summary), /synthetic-secret/u, name);
    assert.deepEqual(plain(core.patchFieldNames("settings")), ["statsFrequency"], name);
  }
});

test("both cores give the same public error messages", async () => {
  for (const [name, core] of await implementations()) {
    // Arrange
    const categories = ["timeout", "http", "invalid-response", "unavailable", undefined];

    // Act
    const messages = categories.map((category) => core.publicError({ category }));

    // Assert
    assert.deepEqual(messages, [
      "The device did not respond before the request timed out.",
      "The device rejected the request.",
      "The device returned an invalid response.",
      "The device is unavailable.",
      "The device is unavailable.",
    ], name);
  }
});

test("both cores format telemetry identically", async () => {
  for (const [name, core] of await implementations()) {
    // Arrange
    const cases: ReadonlyArray<readonly [string, unknown]> = [
      ["hashRate", 512.345], ["temp", 51.26], ["power", 14.04], ["fanrpm", 4011.6], ["wifiRSSI", -61.4],
      ["uptimeSeconds", 3725], ["uptimeSeconds", 90061], ["hashRate", "fast"], ["hostname", "miner"],
      ["hostname", " "], ["sharesAccepted", 12], ["miningActivity", undefined],
    ];

    // Act
    const rendered = cases.map(([field, value]) => core.formatMetric(field, value));

    // Assert
    assert.deepEqual(rendered, [
      "512.3 GH/s", "51.3 °C", "14.0 W", "4012 RPM", "-61 dBm", "1h 2m", "1d 1h", "—", "miner",
      "Unavailable", "12", "Unavailable",
    ], name);
  }
});

test("both cores admit only the bounded exact descending scoreboard shape", async () => {
  for (const [name, core] of await implementations()) {
    // Arrange
    const ascending = [{ ...scoreboardEntry, difficulty: 1 }, { ...scoreboardEntry, difficulty: 2 }];
    const lowercase = [{ ...scoreboardEntry, nonce: "1234abcd" }];
    const oversized = Array.from({ length: 21 }, () => scoreboardEntry);

    // Act
    const accepted = plain(core.scoreboardRows([scoreboardEntry, { ...scoreboardEntry, difficulty: 7 }]));

    // Assert
    assert.equal(accepted.length, 2, name);
    assert.deepEqual(accepted[0], { difficulty: 42.5, jobId: "job-a", extranonce2: "0001", ntime: 1, nonce: "1234ABCD", versionBits: "20000000" }, name);
    for (const rejected of [ascending, lowercase, oversized, null, { entries: [] }]) {
      assert.equal(core.scoreboardRows(rejected).length, 0, name);
    }
  }
});

test("both cores keep the theme bounded and dark by default", async () => {
  for (const [name, core] of await implementations()) {
    // Arrange
    const legacy = { colorScheme: "light", accentColor: "#12ABEF" };

    // Act
    const fallback = plain(core.themeFromPayload(null));
    const fromLegacy = plain(core.themeFromPayload(legacy));
    const payload = plain(core.themePayload({ colorScheme: "unknown", accentColor: "red" }));

    // Assert
    assert.deepEqual(fallback, { scheme: "dark", accent: "#f7931a" }, name);
    assert.deepEqual(fromLegacy, { scheme: "light", accent: "#12ABEF" }, name);
    assert.deepEqual(payload, { colorScheme: "dark", accentColors: { primary: "#f7931a" } }, name);
  }
});
