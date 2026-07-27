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
      {
        textContent: "Unavailable",
        attributes: new Map(
          name === "commit"
            ? [
                ["target", "_blank"],
                ["rel", "noopener noreferrer"],
              ]
            : [],
        ),
        getAttribute(attribute) {
          return this.attributes.get(attribute) ?? null;
        },
        hasAttribute(attribute) {
          return this.attributes.has(attribute);
        },
        removeAttribute(attribute) {
          this.attributes.delete(attribute);
        },
        setAttribute(attribute, value) {
          this.attributes.set(attribute, value);
        },
      },
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

async function recoveryHydrationWith(page, fetch) {
  const fixture = documentFixture();
  const inertElement = { addEventListener() {} };
  const document = {
    getElementById(id) {
      return fixture.elements.get(id) ?? inertElement;
    },
  };
  const script = [...page.matchAll(/<script>([\s\S]*?)<\/script>/g)].at(-1)?.[1];
  assert.ok(script);
  const instrumentedScript = script.replace(
    "void hydrateProvenance();",
    [
      "globalThis.hydrateRecoveryProvenance = hydrateProvenance;",
      "globalThis.recoveryHydration = hydrateProvenance();",
    ].join("\n"),
  );
  const context = vm.createContext({ document, fetch });
  vm.runInContext(instrumentedScript, context, {
    filename: recoveryPath,
  });
  await context.recoveryHydration;
  return { context, elements: fixture.elements };
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
  clean.elements.get("provenance-commit").getAttribute("href"),
  "https://github.com/bright-builds-llc/bitaxe-esp-miner/commit/0123456789abcdef0123456789abcdef01234567",
);
assert.equal(
  clean.elements.get("provenance-commit").getAttribute("target"),
  "_blank",
);
assert.equal(
  clean.elements.get("provenance-commit").getAttribute("rel"),
  "noopener noreferrer",
);
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
assert.equal(
  dirty.elements.get("provenance-commit").getAttribute("href"),
  "https://github.com/bright-builds-llc/bitaxe-esp-miner/commit/fedcba9876543210fedcba9876543210fedcba98",
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
  assert.equal(
    failed.elements.get("provenance-commit").hasAttribute("href"),
    false,
  );
}

const fallback = fs.readFileSync(fallbackPath, "utf8");
const recovery = fs.readFileSync(recoveryPath, "utf8");
for (const page of [fallback, recovery]) {
  assert.match(page, /Firmware provenance/);
  assert.match(page, /id="provenance-version">Unavailable/);
  const commitAnchor = page.match(
    /<a\s+[^>]*id="provenance-commit"[^>]*>Unavailable<\/a>/,
  );
  assert.ok(commitAnchor);
  assert.doesNotMatch(commitAnchor[0], /\shref=/);
  assert.match(commitAnchor[0], /target="_blank"/);
  assert.match(commitAnchor[0], /rel="noopener noreferrer"/);
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
assert.match(
  recovery,
  /commit\.setAttribute\("href", `\$\{commitUrl}\$\{payload\.sourceCommit}`\)/,
);

const recoveryClean = await recoveryHydrationWith(
  recovery,
  successfulResponse({
    semanticVersion: "1.2.3",
    sourceCommit: "0123456789abcdef0123456789abcdef01234567",
    buildTimestampUtc: "2026-07-26T21:24:00Z",
    sourceDirty: false,
  }),
);
assert.equal(
  recoveryClean.elements.get("provenance-commit").textContent,
  "0123456789ab",
);
assert.equal(
  recoveryClean.elements.get("provenance-commit").getAttribute("href"),
  "https://github.com/bright-builds-llc/bitaxe-esp-miner/commit/0123456789abcdef0123456789abcdef01234567",
);

const recoveryDirty = await recoveryHydrationWith(
  recovery,
  successfulResponse({
    semanticVersion: "1.2.3-dev",
    sourceCommit: "fedcba9876543210fedcba9876543210fedcba98",
    buildTimestampUtc: "2026-07-26T21:24:00.123Z",
    sourceDirty: true,
  }),
);
assert.equal(
  recoveryDirty.elements.get("provenance-commit").textContent,
  "fedcba987654 (dirty)",
);
assert.equal(
  recoveryDirty.elements.get("provenance-commit").getAttribute("href"),
  "https://github.com/bright-builds-llc/bitaxe-esp-miner/commit/fedcba9876543210fedcba9876543210fedcba98",
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
  const failed = await recoveryHydrationWith(recovery, fetch);
  for (const element of failed.elements.values()) {
    assert.equal(element.textContent, "Unavailable");
  }
  assert.equal(
    failed.elements.get("provenance-commit").hasAttribute("href"),
    false,
  );
}

const release = JSON.parse(fs.readFileSync(releasePath, "utf8"));
assert.equal(release.schema, "bitaxe-rust-static-release-v1");
assert.equal(release.name, "bitaxe-rust-fallback-ui");
assert.equal(
  release.source_url,
  "https://github.com/bright-builds-llc/bitaxe-esp-miner",
);
