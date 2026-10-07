// Static contract for the SolidJS variant: authored-source rules, the built
// bundle's shape, and agreement with the firmware's server-side route list.
import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import { PAGES, ROUTES } from "../src/core/ui-core.js";
import { SOLID_PACKAGE, readWorkspaceFile, runfileRoot } from "./support.js";

/** SPIFFS object names are 64 bytes including the terminating NUL. */
const SPIFFS_NAME_LIMIT = 63;

async function filesUnder(root: string, prefix = ""): Promise<string[]> {
  const entries = await readdir(path.join(root, prefix), { withFileTypes: true });
  const nested = await Promise.all(entries.map(async (entry) => {
    const relative = prefix === "" ? entry.name : `${prefix}/${entry.name}`;
    return entry.isDirectory() ? filesUnder(root, relative) : [relative];
  }));
  return nested.flat().sort();
}

async function authoredSources(): Promise<string> {
  const root = path.join(runfileRoot(), SOLID_PACKAGE, "src");
  const files = await filesUnder(root);
  const contents = await Promise.all(files.map((file) => readFile(path.join(root, file), "utf8")));
  return contents.join("\n");
}

test("authored solid sources keep the current UI's safety and accessibility contract", async () => {
  // Arrange
  const sources = await authoredSources();

  // Act
  const endpoints = ["/api/system/info", "/api/system/logs", "/api/system/scoreboard", "/api/system/OTA", "/api/system/OTAWWW", "/api/theme", "/api/ws"];

  // Assert
  // Solid's compiled templates use innerHTML internally, so this rule covers authored code only.
  assert.doesNotMatch(sources, /localStorage|sessionStorage|innerHTML|eval\(/u);
  for (const endpoint of endpoints) assert.ok(sources.includes(endpoint), endpoint);
  assert.match(sources, /autocomplete="new-password"/u);
  assert.match(sources, /\.confirm\(/u);
  assert.match(sources, /\.inert = /u);
  assert.match(sources, /Source on GitHub/u);
  assert.match(sources, /https:\/\/openlinks\.us\//u);
  assert.match(sources, /"www-update-form"/u);
});

test("solid routes and pages match the current variant and the server route list", async () => {
  // Arrange
  const serverSource = await readWorkspaceFile("crates/bitaxe-api/src/static_plan.rs");
  const block = /const OPERATOR_UI_ROUTES: &\[&str\] = &\[([^\]]*)\];/u.exec(serverSource)?.[1] ?? "";

  // Act
  const serverRoutes = [...block.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);

  // Assert
  assert.deepEqual(serverRoutes.sort(), Object.keys(ROUTES).sort());
  assert.deepEqual([...PAGES], ["dashboard", "network", "pool", "settings", "scoreboard", "logs", "update", "theme"]);
});

test("the built bundle keeps index.html unhashed and every other script or style content-hashed", async () => {
  // Arrange
  const dist = path.join(runfileRoot(), SOLID_PACKAGE, "dist");

  // Act
  const files = await filesUnder(dist);
  const index = await readFile(path.join(dist, "index.html"), "utf8");
  const referenced = [...index.matchAll(/(?:src|href)="(\/assets\/[^"]+)"/gu)].map((match) => match[1]);

  // Assert
  assert.ok(files.includes("index.html"));
  assert.ok(files.includes("assets/release.json"));
  for (const file of files.filter((name) => /\.(?:js|css)$/u.test(name))) {
    assert.match(file, /^assets\/[A-Za-z0-9_-]+-[A-Za-z0-9_-]{8}\.(?:js|css)$/u, file);
  }
  for (const file of files) assert.ok(`/${file}.gz`.length <= SPIFFS_NAME_LIMIT, file);
  assert.equal(files.some((file) => file.endsWith(".map") || file.endsWith(".gz")), false);
  assert.ok(referenced.length >= 2);
  for (const asset of referenced) assert.ok(files.includes(asset?.slice(1) ?? ""), asset);
});

test("the built bundle keeps the dark default and the responsive layout", async () => {
  // Arrange
  const dist = path.join(runfileRoot(), SOLID_PACKAGE, "dist");
  const files = await filesUnder(dist);

  // Act
  const css = await Promise.all(files.filter((file) => file.endsWith(".css")).map((file) => readFile(path.join(dist, file), "utf8")));
  const index = await readFile(path.join(dist, "index.html"), "utf8");
  const release = JSON.parse(await readFile(path.join(dist, "assets/release.json"), "utf8")) as Record<string, unknown>;

  // Assert
  assert.match(css.join("\n"), /color-scheme:\s*dark/u);
  assert.match(css.join("\n"), /@media\s*\(max-width:\s*620px\)/u);
  assert.match(index, /data-theme="dark"/u);
  assert.match(index, /Source on GitHub/u);
  assert.equal(release["schema"], "bitaxe-rust-static-release-v1");
});
