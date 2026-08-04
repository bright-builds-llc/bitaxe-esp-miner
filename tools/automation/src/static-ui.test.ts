import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import vm from "node:vm";

interface UiCore {
  buildSettingsPatch(kind: string, source: Record<string, string>): Record<string, unknown>;
  isKnownRoute(pathname: string): boolean;
  patchSummary(kind: string, patch: Record<string, unknown>): string[];
  publicError(error: { category: string }): string;
  routeFor(pathname: string): string;
  themeFromPayload(payload: unknown): { scheme: string; accent: string };
  themePayload(values: Record<string, string>): Record<string, unknown>;
}

function runfileRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? (process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd())
    : path.join(maybeRunfiles, "_main");
}

async function staticAssets(): Promise<Record<string, string>> {
  const assetRoot = path.join(runfileRoot(), "firmware/bitaxe/static/www");
  const names = [
    "index.html",
    "assets/app.css",
    "assets/ui-core.js",
    "assets/api-client.js",
    "assets/app.js",
  ];
  const contents = await Promise.all(
    names.map(async (name) => [name, await readFile(path.join(assetRoot, name), "utf8")] as const),
  );
  return Object.fromEntries(contents);
}

function evaluateCore(source: string): UiCore {
  const context: Record<string, unknown> = {};
  vm.runInNewContext(source, context, { filename: "ui-core.js" });
  return context["BitaxeUiCore"] as UiCore;
}

test("operator UI pure core admits only known routes and builds write-only patches", async () => {
  // Arrange
  const assets = await staticAssets();
  const core = evaluateCore(assets["assets/ui-core.js"] ?? "");

  // Act
  const network = core.buildSettingsPatch("network", {
    hostname: "  synthetic-miner  ",
    ssid: "synthetic-network",
    wifiPass: "",
    ignored: "not-admitted",
  });
  const pool = core.buildSettingsPatch("pool", {
    stratumProtocol: "SV1",
    stratumURL: "synthetic.pool.invalid",
    stratumPort: "3333",
    stratumUser: "synthetic.worker",
    stratumPassword: "synthetic-password",
  });
  const summary = core.patchSummary("pool", pool);

  // Assert
  assert.equal(core.routeFor("/network/"), "network");
  assert.equal(core.routeFor("/ap"), "network");
  assert.equal(core.routeFor("/system"), "dashboard");
  assert.equal(core.routeFor("/not-admitted"), "dashboard");
  assert.equal(core.isKnownRoute("/not-admitted"), false);
  assert.deepEqual({ ...network }, {
    hostname: "synthetic-miner",
    ssid: "synthetic-network",
  });
  assert.equal(pool["stratumPort"], 3333);
  assert.equal(pool["ignored"], undefined);
  assert.deepEqual([...summary], [
    "stratumProtocol",
    "stratumURL",
    "stratumPort",
    "stratumUser",
    "stratumPassword:updated",
  ]);
  assert.doesNotMatch(JSON.stringify(summary), /synthetic-password/u);
  assert.equal(core.publicError({ category: "http" }), "The device rejected the request.");
});

test("operator UI theme contract remains bounded and dark by default", async () => {
  // Arrange
  const assets = await staticAssets();
  const core = evaluateCore(assets["assets/ui-core.js"] ?? "");

  // Act
  const fallback = core.themeFromPayload(null);
  const light = core.themePayload({ colorScheme: "light", accentColor: "#12abef" });
  const invalid = core.themePayload({ colorScheme: "unknown", accentColor: "red" });

  // Assert
  assert.deepEqual({ ...fallback }, { scheme: "dark", accent: "#f7931a" });
  assert.equal(light["colorScheme"], "light");
  assert.deepEqual({ ...(light["accentColors"] as Record<string, string>) }, { primary: "#12abef" });
  assert.equal(invalid["colorScheme"], "dark");
});

test("production static UI exposes scoped workflows without browser persistence or OTAWWW", async () => {
  // Arrange
  const assets = await staticAssets();
  const combinedScripts = [
    assets["assets/ui-core.js"],
    assets["assets/api-client.js"],
    assets["assets/app.js"],
  ].join("\n");
  const index = assets["index.html"] ?? "";
  const css = assets["assets/app.css"] ?? "";

  // Act
  const pages = [...index.matchAll(/data-page="([^"]+)"/gu)].map((match) => match[1]);

  // Assert
  assert.deepEqual(pages, ["dashboard", "network", "pool", "settings", "logs", "update", "theme"]);
  assert.match(index, /autocomplete="new-password"/u);
  assert.match(index, /Source on GitHub/u);
  assert.match(index, /https:\/\/openlinks\.us\//u);
  assert.match(index, /AxeOS image update unavailable/u);
  assert.match(combinedScripts, /\/api\/system\/info/u);
  assert.match(combinedScripts, /\/api\/system\/logs/u);
  assert.match(combinedScripts, /\/api\/system\/OTA/u);
  assert.doesNotMatch(combinedScripts, /OTAWWW/u);
  assert.doesNotMatch(combinedScripts, /localStorage|sessionStorage|innerHTML|eval\(/u);
  assert.match(combinedScripts, /global\.confirm/u);
  assert.match(combinedScripts, /navigation\.inert/u);
  assert.match(combinedScripts, /textContent/u);
  assert.match(css, /color-scheme: dark/u);
  assert.match(css, /@media \(max-width: 620px\)/u);
});
