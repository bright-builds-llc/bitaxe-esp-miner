import test from "node:test";
import assert from "node:assert/strict";
import { chmod, link, mkdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { makeCorpusFixture } from "./preflight-corpus.test-fixture.mjs";
import { verifyPreflightCorpus } from "./preflight-corpus.mjs";

test("real batched published Git and mapped corpus prove interrupted classification without relaxing generic guard", async t => {
  // Arrange
  const f = await makeCorpusFixture(t);
  // Act
  const facts = await verifyPreflightCorpus(f.root, f.operations);
  // Assert
  assert.equal(facts.inspectedInputs.length, f.anchors.fileCount); assert.equal(facts.inventorySha256, f.anchors.inventorySha256);
  assert.equal(facts.diagnosedCause.code, "noise_forbidden_inventory_name"); assert.equal(facts.nonClaims.includes("no_fresh_device_accounting"), true);
});
test("changed, missing, extra or effect artifacts and unsafe permissions are rejected", async t => {
  for (const mutation of [
    async f => writeFile(resolve(f.root, "snapshot/gate/page"), "altered"),
    async f => rm(resolve(f.root, "snapshot/gate/page")),
    async f => writeFile(resolve(f.root, "issued.json"), "{}", { mode: 0o600 }),
    async f => chmod(resolve(f.root, "snapshot/gate/page"), 0o644),
    async f => mkdir(`${f.root}.operator`, { mode: 0o700 }),
    async f => writeFile(resolve(f.firmwareRoot, f.anchors.commandLog), "different log"),
  ]) {
    const f = await makeCorpusFixture(t); await mutation(f);
    await assert.rejects(verifyPreflightCorpus(f.root, f.operations));
  }
});
test("symlink and hardlink aliases are rejected", async t => {
  for (const makeAlias of [symlink, link]) {
    const f = await makeCorpusFixture(t), path = resolve(f.root, "snapshot/gate/page"), outside = resolve(f.firmwareRoot, "alias-data");
    await writeFile(outside, await readFile(path), { mode: 0o600 }); await rm(path); await makeAlias(outside, path);
    await assert.rejects(verifyPreflightCorpus(f.root, f.operations), /bootstrap_corpus_path/u);
  }
});
test("published source mismatch is rejected even when mapped corpus hashes remain correct", async t => {
  const f = await makeCorpusFixture(t);
  await assert.rejects(verifyPreflightCorpus(f.root, { ...f.operations, verifyPublishedSources: async () => { throw Error("published source mismatch"); } }), /published source mismatch/u);
});

test("real Git batch rejects a changed expected source digest", async t => {
  const { verifyPublishedSources } = await import("./preflight-corpus-git.mjs");
  const f = await makeCorpusFixture(t);
  await assert.rejects(verifyPublishedSources(f.firmwareRoot, f.anchors.commit,
    [{ ...f.context.sourceInventory[0], sha256: "0".repeat(64) }]), /bootstrap_corpus_git_bytes/u);
});
test("external command log changes during verification cannot be bound as the earlier input", async t => {
  const { verifyPublishedSources } = await import("./preflight-corpus-git.mjs");
  const f = await makeCorpusFixture(t);
  await assert.rejects(verifyPreflightCorpus(f.root, { ...f.operations, verifyPublishedSources: async (...args) => {
    await verifyPublishedSources(...args);
    await writeFile(resolve(f.firmwareRoot, f.anchors.commandLog), "changed during inspection");
  } }), /bootstrap_corpus_changed/u);
});
