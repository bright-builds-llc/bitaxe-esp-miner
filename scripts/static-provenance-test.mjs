#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";

const [scriptPath, fallbackPath, recoveryPath, releasePath] =
  process.argv.slice(2);
assert.ok(scriptPath && fallbackPath && recoveryPath && releasePath);

const context = vm.createContext({});
vm.runInContext(fs.readFileSync(scriptPath, "utf8"), context, {
  filename: scriptPath,
});
const { hydrate } = context.BitaxeProvenance;

function documentFixture() {
  const elements = new Map(
    ["version", "commit", "built"].map((name) => [
      `provenance-${name}`,
      { textContent: "Unavailable" },
    ]),
  );
  return {
    elements,
    document: {
      getElementById(id) {
        return elements.get(id);
      },
    },
  };
}

async function hydrateWith(fetch) {
  const fixture = documentFixture();
  const result = await hydrate({ document: fixture.document, fetch });
  return { result, elements: fixture.elements };
}

function successfulResponse(payload) {
  return async () => ({
    ok: true,
    async json() {
      return payload;
    },
  });
}

const clean = await hydrateWith(
  successfulResponse({
    semanticVersion: "1.2.3",
    sourceCommit: "0123456789abcdef0123456789abcdef01234567",
    buildTimestampUtc: "2026-07-26T21:24:00Z",
    sourceDirty: false,
  }),
);
assert.equal(clean.result, true);
assert.equal(clean.elements.get("provenance-version").textContent, "1.2.3");
assert.equal(clean.elements.get("provenance-commit").textContent, "0123456789ab");
assert.equal(
  clean.elements.get("provenance-built").textContent,
  "2026-07-26T21:24:00Z",
);

const dirty = await hydrateWith(
  successfulResponse({
    semanticVersion: "1.2.3-dev",
    sourceCommit: "fedcba9876543210fedcba9876543210fedcba98",
    buildTimestampUtc: "2026-07-26T21:24:00.123Z",
    sourceDirty: true,
  }),
);
assert.equal(dirty.result, true);
assert.equal(
  dirty.elements.get("provenance-commit").textContent,
  "fedcba987654 (dirty)",
);

for (const fetch of [
  successfulResponse({
    semanticVersion: "1.2.3",
    sourceCommit: "../invalid",
    buildTimestampUtc: "not-a-timestamp",
    sourceDirty: false,
  }),
  async () => ({ ok: false }),
  async () => {
    throw new Error("network unavailable");
  },
]) {
  const failed = await hydrateWith(fetch);
  assert.equal(failed.result, false);
  for (const element of failed.elements.values()) {
    assert.equal(element.textContent, "Unavailable");
  }
}

const fallback = fs.readFileSync(fallbackPath, "utf8");
const recovery = fs.readFileSync(recoveryPath, "utf8");
for (const page of [fallback, recovery]) {
  assert.match(page, /Firmware provenance/);
  assert.match(page, /id="provenance-version">Unavailable/);
  assert.match(page, /id="provenance-commit">Unavailable/);
  assert.match(page, /id="provenance-built">Unavailable/);
  assert.match(
    page,
    /href="https:\/\/github\.com\/bright-builds-llc\/bitaxe-esp-miner"/,
  );
  assert.match(page, /target="_blank"/);
  assert.match(page, /rel="noopener noreferrer"/);
}
assert.match(fallback, /src="\/assets\/provenance\.js"/);
assert.match(recovery, /fetch\("\/api\/system\/info"/);

const release = JSON.parse(fs.readFileSync(releasePath, "utf8"));
assert.equal(release.schema, "bitaxe-rust-static-release-v1");
assert.equal(release.name, "bitaxe-rust-fallback-ui");
assert.equal(
  release.source_url,
  "https://github.com/bright-builds-llc/bitaxe-esp-miner",
);
