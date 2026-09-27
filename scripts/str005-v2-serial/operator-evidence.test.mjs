import assert from "node:assert/strict";
import { readFile, readdir, unlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { completedFixture } from "./completed-fixture.mjs";
import { finalize, review } from "./finalize.mjs";
import { proof } from "../str005-noise-serial/files.mjs";

test("completed operator evidence seals and historical review uses its immutable snapshot", async t => {
  // Arrange: explicit software fixture; actual daemon lifetime has separate process tests.
  const f = await completedFixture(t);
  // Act
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  const saved = (await proof(f.root, "final-result.json")).value;
  // Assert: later external operator-file changes cannot rewrite historical judgment.
  assert.equal(result.status, "passed");
  assert(saved.inputs.operator.some(item => item.path === "disposition.json"));
  await writeFile(resolve(`${f.root}.operator`, "disposition.json"), "changed after sealing\n");
  assert.deepEqual(await review(f.root, { ...f.operations,
    processSnapshot: () => assert.fail("historical kernel query") }), result);
});

test("stopped operator without a required installation operation cannot qualify", async t => {
  // Arrange: all device/cleanup evidence remains complete; one operator obligation is absent.
  const f = await completedFixture(t), directory = `${f.root}.operator`;
  const names = await readdir(directory);
  let removed = false;
  for (const name of names.filter(name => /^request-[a-f0-9]{32}\.json$/u.test(name))) {
    const request = JSON.parse(await readFile(resolve(directory, name)));
    if (request.action !== "install") continue;
    await unlink(resolve(directory, name));
    await unlink(resolve(directory, `result-${request.requestId}.json`));
    removed = true; break;
  }
  assert(removed);
  // Act
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  // Assert
  assert.equal(result.status, "unverified");
  assert.equal((await proof(f.root, "judgment-failure.json")).value.code, "v2_operator_incomplete");
  assert.deepEqual(await review(f.root, f.operations), result);
});

test("a live operator cannot be sealed even when its files claim stopped", async t => {
  // Arrange
  const f = await completedFixture(t);
  const { owner } = (await proof(`${f.root}.operator`, "locator.json")).value;
  // Act / Assert: kernel absence is independent of a persisted success assertion.
  await assert.rejects(finalize(f.root, `${f.root}.cleanup/receipt.json`, {
    ...f.operations, processSnapshot: async () => [owner],
  }), { code: "noise_owner_remains" });
  await assert.rejects(readFile(resolve(f.root, "final-result.json")), { code: "ENOENT" });
});

test("an unrelated absent process cannot substitute for the supervisor's operator parent", async t => {
  // Arrange: keep both forged operator records mutually consistent.
  const f = await completedFixture(t), directory = `${f.root}.operator`;
  for (const name of ["locator.json", "disposition.json"]) {
    const path = resolve(directory, name), value = JSON.parse(await readFile(path));
    value.owner.pid += 1; value.owner.pgid += 1;
    await writeFile(path, JSON.stringify(value));
  }
  // Act
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  // Assert: the recorded supervisor parent must independently match.
  assert.equal(result.status, "unverified");
  assert.equal((await proof(f.root, "judgment-failure.json")).value.code, "v2_operator_owner_join");
});
