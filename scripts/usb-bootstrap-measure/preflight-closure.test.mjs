import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile, link, unlink } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { makeCorpusFixture } from "./preflight-corpus.test-fixture.mjs";
import { closePreflight, reviewPreflight } from "./preflight-closure.mjs";
import { CHECKER_FILES } from "./checker-identity.mjs";
import { CONTRACT, PREFLIGHT_AMENDMENT } from "./values.mjs";
import { canonical, proof } from "../str005-noise-serial/files.mjs";
const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
async function fixture(t) {
  const f = await makeCorpusFixture(t);
  for (const path of [...CHECKER_FILES, CONTRACT.path, PREFLIGHT_AMENDMENT.path]) {
    await mkdir(dirname(resolve(f.firmwareRoot, path)), { recursive: true, mode: 0o700 });
    await writeFile(resolve(f.firmwareRoot, path), await readFile(resolve(REPO, path)), { mode: 0o600 });
  }
  const git = args => execFileSync("git", ["-C", f.firmwareRoot, ...args], { encoding: "utf8" }).trim();
  git(["add", "scripts", "firmware", "docs"]); git(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "Published corrected checker fixture"]);
  git(["update-ref", "refs/remotes/origin/main", git(["rev-parse", "HEAD"])]);
  return { ...f, operations: { ...f.operations, cleanPushed() {}, verifyClosureAncestry: async () => {} } };
}
test("exact interrupted preparation closes exclusively as unverified without changing its bytes", async t => {
  // Arrange.
  const f = await fixture(t), before = (await proof(f.root, "context.json")).sha256;
  // Act.
  const created = await closePreflight(f.root, f.operations), reviewed = await reviewPreflight(f.root, f.operations);
  // Assert.
  assert.equal(created.closureSha256, reviewed.closureSha256); assert.equal(created.classification, "interrupted_before_effects");
  assert.equal(created.effectsObserved, false); assert.equal((await proof(f.root, "context.json")).sha256, before);
  const receipt = (await proof(dirname(f.root), created.closurePath)).value;
  assert.equal(receipt.earliestCause.code, "bootstrap_operation_failed"); assert.equal(receipt.diagnosedCause.code, "noise_forbidden_inventory_name");
  assert.equal(canonical(receipt.inspectedInputs), canonical(f.inventory)); await assert.rejects(closePreflight(f.root, f.operations));
});
test("interrupted pending closure cannot be overwritten or treated as reviewed", async t => {
  const f = await fixture(t); f.operations.beforeClosurePublish = () => { throw Error("synthetic interruption"); };
  await assert.rejects(closePreflight(f.root, f.operations), /synthetic interruption/u);
  await assert.rejects(reviewPreflight(f.root, f.operations)); delete f.operations.beforeClosurePublish;
  await assert.rejects(closePreflight(f.root, f.operations));
});
test("closure rejects external command log drift and receipt field tampering", async t => {
  const f = await fixture(t), created = await closePreflight(f.root, f.operations);
  const path = created.closurePath, original = await readFile(path), value = JSON.parse(original); value.effectsObserved = true;
  await writeFile(path, JSON.stringify(value)); await assert.rejects(reviewPreflight(f.root, f.operations)); await writeFile(path, original);
  await writeFile(resolve(f.firmwareRoot, f.anchors.commandLog), "changed original command output\n");
  await assert.rejects(reviewPreflight(f.root, f.operations));
});
test("checker drift during publication keeps the closure pending and never publishes", async t => {
  const f = await fixture(t);
  f.operations.beforeClosurePublish = async () => writeFile(resolve(f.firmwareRoot, "scripts/usb-bootstrap-measure/main.mjs"), "changed unpublished checker");
  await assert.rejects(closePreflight(f.root, f.operations));
  await assert.rejects(reviewPreflight(f.root, f.operations));
});
test("closure CLI exposes only the compact non-authorizing result", async t => {
  const f = await fixture(t), { main } = await import("./main.mjs");
  const value = await main(["close-preflight", "--private-root", f.root], f.operations);
  assert.deepEqual(Object.keys(value).sort(), ["status", "classification", "contextSha256", "closureSha256", "nonClaims", "hardware_qualified", "device_effects"].sort());
  assert.equal(value.device_effects, false); assert.equal(JSON.stringify(value).includes(f.firmwareRoot), false);
});

test("a linked closure alias is not accepted as an independent receipt", async t => {
  const f = await fixture(t), created = await closePreflight(f.root, f.operations), alias = `${created.closurePath}.alias`;
  await link(created.closurePath, alias); await assert.rejects(reviewPreflight(f.root, f.operations), { code: "bootstrap_closure_private_path" }); await unlink(alias);
});
