import test from "node:test";
import assert from "node:assert/strict";
import { createServer } from "node:net";
import { once } from "node:events";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { detachedOwner, kill, workspaceFixture, writeCollection, writeServerOwner } from "./fixtures.test-helper.mjs";
import { isLive } from "./host.mjs";
import { positionalRecipes } from "./just.mjs";
import { stopAndFinish } from "./owner-finish.mjs";
import { ownerLayout } from "./owners.mjs";

const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);

test("owner-finish stops the recorded owner, writes the final detector and runs finish in order", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(fixture.root, owner);
  // Act
  const { summary } = await stopAndFinish({ name: "str005-heartbeat-shutdown", root: fixture.root }, fixture.operations);
  // Assert
  assert.deepEqual([summary.stop, summary.finish_exit, await isLive(owner)], ["stopped", 0, false]);
  assert.deepEqual(await fixture.calls(), [["detect-ultra205"], ["str005-heartbeat-shutdown", "finish", "--private-root", fixture.root]]);
  assert.match(await readFile(resolve(fixture.parent, "final-detector.stdout.log"), "utf8"), /^port: /mu);
});

test("a staged owner uses its stage's owner record, detector name and finish flags", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(resolve(fixture.root, "restart"), owner);
  // Act
  await stopAndFinish({ name: "str005-step5-restart", root: fixture.root, maybeStage: "restart" }, fixture.operations);
  // Assert
  assert.deepEqual((await fixture.calls()).at(-1), ["str005-step5-restart", "finish", "--private-root", fixture.root, "--stage", "restart"]);
  assert.match(await readFile(resolve(fixture.parent, "restart-final-detector.stdout.log"), "utf8"), /^port: /mu);
});

test("a missing owner record is refused before any command runs", async () => {
  // Arrange
  const fixture = await workspaceFixture();
  // Act / Assert
  await rejectsWith(stopAndFinish({ name: "str005-accepted-share", root: fixture.root }, fixture.operations), "owner_record_missing");
  assert.deepEqual(await fixture.calls(), []);
});

test("an existing final detector is refused while the owner keeps running", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(fixture.root, owner);
  await writeFile(resolve(fixture.parent, "final-detector.stdout.log"), "", { mode: 0o600 });
  // Act / Assert
  await rejectsWith(stopAndFinish({ name: "str005-share-probe", root: fixture.root }, fixture.operations), "owner_output_exists");
  assert.equal(await isLive(owner), true);
});

test("a port still held after the owner exits stops the run before the detector", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  const owner = await detachedOwner();
  const listener = createServer().listen(0, "127.0.0.1");
  await once(listener, "listening");
  t.after(() => { listener.close(); return kill(owner); });
  await writeServerOwner(fixture.root, owner, listener.address().port);
  // Act / Assert
  await rejectsWith(stopAndFinish({ name: "str005-heartbeat-probe", root: fixture.root }, fixture.operations), "port_held");
  assert.deepEqual(await fixture.calls(), []);
});

test("a recorded pid now owned by another process is not signalled", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(fixture.root, { ...owner, startedAt: "Thu Jan  1 00:00:00 1970" });
  // Act / Assert: the live process shares the recorded group, so release can never be proved.
  await rejectsWith(stopAndFinish({ name: "str005-heartbeat-probe", root: fixture.root }, { ...fixture.operations, ownerStopMs: 1000 }), "owner_exit_unproved");
  assert.equal(await isLive(owner), true);
});

test("waiting for a collection refuses one whose freshness window already closed, before stopping", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(fixture.root, owner);
  await writeCollection(fixture.root, { startedAtUnixMs: Date.now() - 121_000 });
  // Act / Assert
  await rejectsWith(stopAndFinish({ name: "str005-control-diagnostic-recovery", root: fixture.root, waitCollection: true }, fixture.operations),
    "recovery_window_expired");
  assert.equal(await isLive(owner), true);
});

test("owners outside the closed table and wrong stage shapes are refused", () => {
  // Arrange
  const cases = [["str005-noise-serial", undefined], ["str005-step5-restart", undefined], ["str005-accepted-share", "restart"]];
  // Act
  const codes = cases.map(([name, stage]) => { try { ownerLayout(name, "/p/r", stage); return "accepted"; } catch (error) { return error.code; } });
  // Assert
  assert.deepEqual(codes, ["owner_unknown", "owner_stage_required", "owner_stage_unexpected"]);
});

test("a positional-argument recipe receives its finish arguments without shell quotes", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  await writeFile(resolve(fixture.base, "Justfile"), "[positional-arguments]\nultra205-soak action *args:\n    true\n");
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(fixture.root, owner);
  // Act
  await stopAndFinish({ name: "ultra205-soak", root: fixture.root }, fixture.operations);
  // Assert
  const raw = (await readFile(resolve(fixture.env.FAKE_JUST_DIR, "raw-calls.jsonl"), "utf8")).trim().split("\n").map((line) => JSON.parse(line));
  assert.deepEqual(raw.at(-1), ["ultra205-soak", "finish", "--private-root", fixture.root]);
});

test("positional recipes are read from the Justfile", () => {
  // Arrange
  const text = "# comment\n[positional-arguments]\n@diagnose *args:\n    x\n\nother *args:\n    y\n[positional-arguments]\nhardware-operator action *args:\n";
  // Act
  const names = positionalRecipes(text);
  // Assert
  assert.deepEqual([...names].sort(), ["diagnose", "hardware-operator"]);
});

test("the restoration owner is finished through its positional recipe without a stage", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  await writeFile(resolve(fixture.base, "Justfile"), "[positional-arguments]\nbwg-restoration action *args:\n    true\n");
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(fixture.root, owner);
  // Act
  await stopAndFinish({ name: "bwg-restoration", root: fixture.root }, fixture.operations);
  // Assert
  const raw = (await readFile(resolve(fixture.env.FAKE_JUST_DIR, "raw-calls.jsonl"), "utf8")).trim().split("\n").map((line) => JSON.parse(line));
  assert.deepEqual(raw.at(-1), ["bwg-restoration", "finish", "--private-root", fixture.root]);
  assert.equal(ownerLayout("bwg-restoration", fixture.root).owner.staged, false);
});
