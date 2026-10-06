import test from "node:test";
import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { processSnapshot } from "../host-stalls/capture.mjs";
import { resolve } from "node:path";
import { detachedOwner, kill, workspaceFixture, writeCollection, writeServerOwner } from "./fixtures.test-helper.mjs";
import { isLive, liveIdentity } from "./host.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { recoveryCoreDump, restartSequence } from "./sequences.mjs";

const rejectsWith = (promise, code) => assert.rejects(promise, (error) => error.code === code);
const recipes = async (fixture) => (await fixture.calls()).map((call) => call.slice(0, 2).join(" "));

async function restartFixture(t, scenario = {}, collection = {}) {
  const fixture = await workspaceFixture(scenario);
  const owner = await detachedOwner();
  const recovery = resolve(fixture.root, "recovery");
  await writeServerOwner(recovery, owner);
  await writeCollection(recovery, collection);
  t.after(async () => {
    await kill(owner);
    const served = await proof(fixture.root, "restart/server-owner.json").catch(() => null);
    if (served) await kill(await liveIdentity(served.value.owner.pid));
  });
  return { fixture, owner };
}

test("restart-sequence finishes the recovery stage, detects, then starts the restart serve", async (t) => {
  // Arrange
  const { fixture, owner } = await restartFixture(t);
  // Act
  const result = await restartSequence({ name: "str005-step5-restart", root: fixture.root }, fixture.operations);
  // Assert
  assert.equal(result.event, "restart_serving");
  assert.equal(await isLive(owner), false);
  assert.deepEqual(await recipes(fixture), ["detect-ultra205", "str005-step5-restart finish", "detect-ultra205", "str005-step5-restart serve"]);
});

test("restart-sequence refuses an incomplete recovery without starting the restart serve", async (t) => {
  // Arrange
  const { fixture } = await restartFixture(t, { recoveryComplete: false });
  // Act / Assert
  await rejectsWith(restartSequence({ name: "str005-step5-restart", root: fixture.root }, fixture.operations), "recovery_incomplete");
  assert.deepEqual(await recipes(fixture), ["detect-ultra205", "str005-step5-restart finish"]);
});

test("restart-sequence refuses a recovery whose window closed, before stopping its owner", async (t) => {
  // Arrange
  const { fixture, owner } = await restartFixture(t, {}, { startedAtUnixMs: Date.now() - 125_000 });
  // Act / Assert
  await rejectsWith(restartSequence({ name: "str005-step5-restart", root: fixture.root }, fixture.operations), "recovery_window_expired");
  assert.equal(await isLive(owner), true);
});

test("restart-sequence refuses an existing restart stage before any effect", async (t) => {
  // Arrange
  const { fixture, owner } = await restartFixture(t);
  await mkdir(resolve(fixture.root, "restart"), { mode: 0o700 });
  // Act / Assert
  await rejectsWith(restartSequence({ name: "str005-step5-restart", root: fixture.root }, fixture.operations), "restart_stage_exists");
  assert.equal(await isLive(owner), true);
});

test("a serve that exits before recording its owner is reported", async (t) => {
  // Arrange
  const { fixture } = await restartFixture(t, { serveFails: true });
  // Act / Assert
  await rejectsWith(restartSequence({ name: "str005-step5-restart", root: fixture.root }, fixture.operations), "restart_serve_exited");
});

async function recoveryFixture(t, scenario) {
  const fixture = await workspaceFixture({ proof: true, ...scenario });
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  await writeServerOwner(fixture.root, owner);
  return { fixture, coreRoot: resolve(fixture.parent, "core-001") };
}

test("recovery-core-dump reads the core dump with every expectation taken from the fresh proof", async (t) => {
  // Arrange
  const { fixture, coreRoot } = await recoveryFixture(t, {});
  // Act
  const result = await recoveryCoreDump({ name: "str005-control-diagnostic-recovery", root: fixture.root, coreRoot }, fixture.operations);
  // Assert
  assert.equal(result.read_exit, 0);
  assert.deepEqual((await fixture.calls()).at(-1), ["core-dump-read", "--board", "205", "--port", "/dev/cu.usbmodemFAKE1",
    "--expected-physical-sha256", "a".repeat(64), "--expected-installed-source", "b".repeat(40), "--expected-installed-elf", "c".repeat(64),
    "--recovery-proof", resolve(fixture.root, "current-recovery.json"), "--private-root", coreRoot]);
});

test("a stale recovery proof stops before the core-dump read", async (t) => {
  // Arrange
  const { fixture, coreRoot } = await recoveryFixture(t, { proofAgeMs: 121_000 });
  // Act / Assert
  await rejectsWith(recoveryCoreDump({ name: "str005-share-recovery", root: fixture.root, coreRoot }, fixture.operations), "recovery_window_expired");
  assert.equal((await recipes(fixture)).includes("core-dump-read"), false);
});

test("a detector that sees another device stops before the core-dump read", async (t) => {
  // Arrange
  const { fixture, coreRoot } = await recoveryFixture(t, { physicals: ["a".repeat(64), "d".repeat(64)] });
  // Act / Assert
  await rejectsWith(recoveryCoreDump({ name: "str005-panic-recovery", root: fixture.root, coreRoot }, fixture.operations), "detector_physical_mismatch");
  assert.equal((await recipes(fixture)).includes("core-dump-read"), false);
});

test("an existing core root and a non-recovery owner are refused before any effect", async (t) => {
  // Arrange
  const { fixture, coreRoot } = await recoveryFixture(t, {});
  await mkdir(coreRoot, { mode: 0o700 });
  // Act / Assert
  await rejectsWith(recoveryCoreDump({ name: "str005-share-recovery", root: fixture.root, coreRoot }, fixture.operations), "core_root_exists");
  await rejectsWith(recoveryCoreDump({ name: "str005-accepted-share", root: fixture.root, coreRoot }, fixture.operations), "owner_not_recovery");
  assert.deepEqual(await fixture.calls(), []);
});

test("a restart serve that never records its owner is stopped, leaving no untracked serve", async (t) => {
  // Arrange
  const { fixture } = await restartFixture(t, { serveHangs: true });
  // Act
  await rejectsWith(restartSequence({ name: "str005-step5-restart", root: fixture.root }, { ...fixture.operations, serveReadyMs: 500 }),
    "restart_serve_not_ready");
  // Assert
  const pgid = Number(await readFile(resolve(fixture.env.FAKE_JUST_DIR, "serve.pid"), "utf8"));
  const rows = await processSnapshot();
  assert.equal(rows.some((row) => row.pgid === pgid && !row.state.startsWith("Z")), false);
});

test("a collection-begin record still being written is waited for rather than refused", async (t) => {
  // Arrange
  const fixture = await workspaceFixture();
  const owner = await detachedOwner();
  t.after(() => kill(owner));
  const recovery = resolve(fixture.root, "recovery");
  await writeServerOwner(recovery, owner);
  await writeFile(resolve(recovery, "collection-begin.json"), "", { mode: 0o600 });
  setTimeout(() => writeCollection(recovery).catch(() => {}), 100);
  // Act
  const result = await restartSequence({ name: "str005-step5-restart", root: fixture.root }, { ...fixture.operations, collectionBeginGraceMs: 2000 });
  // Assert
  assert.equal(result.event, "restart_serving");
  const served = await proof(fixture.root, "restart/server-owner.json");
  await kill(await liveIdentity(served.value.owner.pid));
});
