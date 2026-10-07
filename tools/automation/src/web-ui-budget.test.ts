// Bazel test //tools/automation:web_ui_budget_test: every staged variant must
// stay within its checked-in gzip budget, keep exact gzip siblings and fit the
// `www` partition. all.test.ts does not import it, so automation_test does not
// depend on the web UI builds.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import { readStage } from "./web-ui-files.js";
import { budgetViolations, spiffsObjectPages, variantSize, type WebUiBudget } from "./web-ui-sizes.js";
import { verifyGzipSiblings, webUiVariants } from "./web-ui-stage.js";

/** 3 MiB `www` partition: 768 blocks of 15 usable pages, less two blocks SPIFFS keeps free for garbage collection. */
const usableWwwPages = (768 - 2) * 15;

function runfile(relative: string): string {
  const maybeRunfiles = process.env["RUNFILES_DIR"] ?? process.env["JS_BINARY__RUNFILES"];
  return maybeRunfiles === undefined ? path.resolve(relative) : path.join(maybeRunfiles, "_main", relative);
}

async function stagedVariants() {
  return Promise.all(webUiVariants.map(async (variant) => {
    const stage = await readStage(runfile(`firmware/bitaxe/web-ui-${variant}`));
    assert.equal(stage.metadata.variant, variant);
    return stage;
  }));
}

test("every staged web UI variant stays within its gzip budget", async () => {
  // Arrange
  const budget = JSON.parse(await readFile(runfile("firmware/bitaxe/web-ui-budget.json"), "utf8")) as WebUiBudget;
  const stages = await stagedVariants();

  // Act
  const sizes = stages.map((stage) => variantSize(stage.metadata.variant, stage.files));
  const violations = budgetViolations(sizes, budget);

  // Assert
  assert.equal(budget.schema_version, "bitaxe-web-ui-budget-v1");
  assert.deepEqual(violations, []);
});

test("every staged gzip file decompresses to exactly its source", async () => {
  // Arrange
  const stages = await stagedVariants();

  // Act
  const gzipCounts = stages.map((stage) => stage.files.filter((file) => file.path.endsWith(".gz")).length);

  // Assert
  for (const stage of stages) assert.doesNotThrow(() => verifyGzipSiblings(stage.files), stage.metadata.variant);
  assert.ok(gzipCounts.every((count) => count > 0));
});

test("every staged variant fits the www partition with a version.txt", async () => {
  // Arrange
  const stages = await stagedVariants();

  // Act
  const pages = stages.map((stage) => variantSize(stage.metadata.variant, stage.files).estimatedSpiffsPages + spiffsObjectPages(23));

  // Assert
  for (const used of pages) assert.ok(used <= usableWwwPages, `${String(used)} pages exceed ${String(usableWwwPages)}`);
});
