import assert from "node:assert/strict";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { resetOriginFixture } from "./reset-origin-fixtures.mjs";
import { loadResetOriginContext, resetOriginPreflight, RESET_ORIGIN_POLICY } from "./reset-origin-context.mjs";
import { fileDigest, readJson } from "./contract.mjs";

test("observe-only preflight separates published host driver from unchanged retained runtime", async (t) => {
  // Arrange
  const f = await resetOriginFixture(t);
  // Act
  const context = await loadResetOriginContext(f.root, { operations: f.operations });
  // Assert
  assert.notEqual(context.qualification_driver.source_commit, context.firmware_commit);
  assert.equal(context.firmware_commit, f.context.firmware_commit);
  assert.deepEqual(context.observation_policy, RESET_ORIGIN_POLICY);
  assert.equal(context.mining_authorized, false);
  assert.equal(context.restart_authorized, false);
  assert.equal(context.installation_authorized, false);
  assert.equal(context.manifest, resolve(f.root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"));
  assert.equal((await readJson(resolve(f.root, "artifact-snapshot.json"))).files.length, 13);
  assert.equal(context.runtime_source.failed_inventory_sha256, await fileDigest(resolve(f.sourceRoot, "failed-inventory.json")));
});

test("driver permits only declared host changes and never firmware, trust, deletion or nonregular files", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["firmware", "trust", "delete", "symlink"]) {
    const f = await resetOriginFixture(t, { preflight: false });
    const base = f.operations.git;
    f.operations.git = (root, args) =>
      args[0] === "diff"
        ? mode === "firmware"
          ? "M\tfirmware/bitaxe/src/main.rs"
          : mode === "trust"
            ? "M\tfirmware/bitaxe/bwg/deployment-trust.json"
            : mode === "delete"
              ? "D\tTASKS.md"
              : "M\tTASKS.md"
        : args[0] === "ls-tree" && mode === "symlink"
          ? "120000 blob fixture"
          : base(root, args);
    await assert.rejects(resetOriginPreflight(f.options, f.operations), { code: "reset_origin_driver_forbidden_change" });
    assert(!(await readdir(dirname(f.root))).includes("observation"));
  }
});

test("transitive contract drift is detected even when the five new files are unchanged", async (t) => {
  // Arrange
  const f = await resetOriginFixture(t),
    path = resolve(f.options.firmwareRoot, "scripts/fixed-usb-qualification/contract.mjs");
  await writeFile(path, (await readFile(path, "utf8")) + "\n// fixture drift\n");
  // Act / Assert
  await assert.rejects(loadResetOriginContext(f.root, { operations: f.operations }), { code: "reset_origin_driver_drift" });
});

test("observation assignment cannot be reused after interruption before root creation", async (t) => {
  // Arrange
  const f = await resetOriginFixture(t, { preflight: false });
  // Act / Assert
  await assert.rejects(
    resetOriginPreflight(f.options, {
      ...f.operations,
      mkdir: async () => {
        throw Error("fixture interruption");
      },
    }),
    /fixture interruption/u,
  );
  await assert.rejects(resetOriginPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), {
    code: "EEXIST",
  });
  await assert.rejects(readFile(resolve(f.root, "context.json")), { code: "ENOENT" });
});

test("source anchor, campaign, driver ancestry and active task remain mandatory", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["anchor", "campaign", "ancestry", "archived"]) {
    const f = await resetOriginFixture(t, { preflight: false });
    if (mode === "anchor") delete f.operations.expectedResetOriginSourceSeal;
    if (mode === "campaign")
      await writeFile(
        f.options.originalCampaignRecord,
        JSON.stringify({ schema: "fixed-usb-campaign-v1", campaign_id: Buffer.alloc(16, 8).toString("base64url") }),
      );
    if (mode === "ancestry") {
      const base = f.operations.git;
      f.operations.git = (root, args) => (args[0] === "merge-base" ? "0".repeat(40) : base(root, args));
    }
    if (mode === "archived") await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Active\n");
    await assert.rejects(resetOriginPreflight(f.options, f.operations));
  }
});

test("authority and pool inputs fail before source or credential paths can be inspected", async () => {
  // Arrange / Act / Assert
  for (const field of ["authorityDirectory", "poolCredentials"])
    await assert.rejects(resetOriginPreflight({ privateRoot: "/missing", [field]: "/never-read" }), {
      code: "reset_origin_credentials_forbidden",
    });
});
