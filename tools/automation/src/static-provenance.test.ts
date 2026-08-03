import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

function runfileRoot(): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  return maybeRunfiles === undefined
    ? (process.env["BUILD_WORKSPACE_DIRECTORY"] ?? process.cwd())
    : path.join(maybeRunfiles, "_main");
}

test("static provenance assets preserve unavailable fallback and safe commit links", async () => {
  // Arrange
  const root = runfileRoot();
  const assetRoot = path.join(root, "firmware/bitaxe/static");
  const [script, fallback, recovery, releaseDocument] = await Promise.all([
    readFile(path.join(assetRoot, "www/assets/provenance.js"), "utf8"),
    readFile(path.join(assetRoot, "www/index.html"), "utf8"),
    readFile(path.join(assetRoot, "recovery_page.html"), "utf8"),
    readFile(path.join(assetRoot, "www/assets/release.json"), "utf8"),
  ]);

  // Act
  const release = JSON.parse(releaseDocument) as Record<string, unknown>;

  // Assert
  for (const page of [fallback, recovery]) {
    assert.match(page, /Firmware provenance/u);
    assert.match(page, /id="provenance-version">Unavailable/u);
    assert.match(page, /target="_blank"/u);
    assert.match(page, /rel="noopener noreferrer"/u);
  }
  assert.match(script, /\/commit\//u);
  assert.match(script, /sourceDirty/u);
  assert.match(script, /Unavailable/u);
  assert.equal(release["schema"], "bitaxe-rust-static-release-v1");
});
