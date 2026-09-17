import assert from "node:assert/strict";
import test from "node:test";
import { resolve } from "node:path";
import { readFile, stat } from "node:fs/promises";
import { contextFixture } from "./context-fixtures.mjs";
import { createJournal, saveAccounting } from "./journal.mjs";
import { installed } from "./completed-fixture.mjs";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { admitExecution } from "./operator-execution.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { sha256 } from "./values.mjs";

async function fixture(t, scope = "channel", closed = true) {
  const f = await contextFixture(t, { scope }), journal = await createJournal(f.root, f.context);
  await journal.state("before", state(f.context, "before", closed), 1);
  return { ...f, permit: { kind: "execute", contextSha256: sha256(JSON.stringify(f.context)), claimSha256: null } };
}

test("fresh closed ownership admits only the fixed detector command without acquiring hardware", async t => {
  const f = await fixture(t);
  const result = await admitExecution(f.root, "detect", 0, f.permit, f.operations);
  assert.deepEqual(result.argv, ["detect-ultra205"]);
});

test("connected browser ownership prevents detector admission", async t => {
  const f = await fixture(t, "channel", false);
  await assert.rejects(admitExecution(f.root, "detect", 0, f.permit, f.operations), { code: "v2_baseline_connection" });
});

test("parent context pin and separate Share installation inventory cannot be overridden", async t => {
  const f = await fixture(t, "share");
  await assert.rejects(admitExecution(f.root, "detect", 0, f.permit, f.operations), { code: "v2_execute_context" });
  await assert.rejects(admitExecution(f.root, "detect", 1, { ...f.permit, contextSha256: "0".repeat(64) }, f.operations), { code: "v2_execute_context" });
  await assert.rejects(admitExecution(f.root, "detect", 1, { ...f.permit, claimSha256: "0".repeat(64) }, f.operations), { code: "v2_detect_claim" });
});

test("archived authority is rejected again by the effect child", async t => {
  const f = await fixture(t);
  await f.put(resolve(f.options.firmwareRoot, "TASKS.md"), "## Future\n### task-str005-v2-serial-qualification | inactive\n");
  await assert.rejects(admitExecution(f.root, "detect", 0, f.permit, f.operations), { code: "v2_live_task_inactive" });
});

/** Real private files, claim producer and late admission; all OS effects are injected absence checks. */
async function flashFixture(t) {
  const f = await contextFixture(t), journal = await createJournal(f.root, f.context);
  await writeNew(resolve(f.previous.root, "install-0.claim.json"), { detector: { physical: "c".repeat(64) } });
  await writeNew(resolve(f.previous.root, "sealed-inventory.json"), { files: [{ path: "install-0.claim.json",
    sha256: (await proof(f.previous.root, "install-0.claim.json")).sha256 }] });
  const initial = state(f.context, "before");
  await journal.state("before", initial, 1);
  await saveAccounting(f.root, f.context, { stage: "before-install", state: initial,
    ledger, original_budget: original }, journal.lastState());
  await journal.state("before", state(f.context, "before", true), 2);
  const { claim, person } = await installed(f, 0, true);
  f.operations.pid = person.pid;
  const observed = (await proof(f.root, "install-0.detect.observation.json")).value;
  let now = f.operations.unixNow();
  f.operations.unixNow = () => now;
  const absence = f.operations.execFileSync;
  let holderChecks = 0, maybeBeforeHolder;
  f.operations.execFileSync = (program, args, options) => {
    assert.equal(program, "/usr/sbin/lsof");
    assert.deepEqual(args, ["-t", holderChecks % 2 === 0 ? "/dev/cu.synthetic" : "/dev/tty.synthetic"]);
    holderChecks++; maybeBeforeHolder?.(holderChecks);
    return absence(program, args, options);
  };
  return { ...f, finishedAt: observed.finished_at_unix_ms,
    permit: { kind: "execute", contextSha256: claim.context_sha256, claimSha256: claim.claim_sha256 },
    setNow(value) { now = value; }, beforeHolder(callback) { maybeBeforeHolder = callback; },
    holderChecks: () => holderChecks };
}

test("fresh same-physical flash permit returns only the fixed admitted argv", async t => {
  // Arrange
  const f = await flashFixture(t); f.setNow(f.finishedAt + 500);
  // Act
  const result = await admitExecution(f.root, "flash", 0, f.permit, f.operations);
  // Assert: admission returns data only; no child or image directory is created.
  assert.deepEqual(result.argv, ["flash-monitor", "--board", "205", "--port", "/dev/cu.synthetic",
    "--manifest", f.context.manifest, "--evidence-dir", resolve(f.root, "install-0"),
    "--capture-timeout-seconds", "30", "--redact-evidence"]);
  assert.equal(f.holderChecks(), 4);
  assert.equal((await stat(resolve(f.root, "install-0.claim.json"))).mode & 0o777, 0o600);
  await assert.rejects(stat(resolve(f.root, "install-0")), { code: "ENOENT" });
});

test("a flash permit becomes unusable when its detector ages after the consumed claim", async t => {
  // Arrange
  const f = await flashFixture(t); f.setNow(f.finishedAt + 60001);
  // Act / Assert
  await assert.rejects(admitExecution(f.root, "flash", 0, f.permit, f.operations), { code: "noise_detector_stale" });
  assert.equal(f.holderChecks(), 0);
  assert.equal((await proof(f.root, "install-0.claim.json")).sha256, f.permit.claimSha256);
  await assert.rejects(stat(resolve(f.root, "install-0")), { code: "ENOENT" });
});

test("time spent in the final holder check cannot carry a fresh detector past its deadline", async t => {
  // Arrange: the shared fresh-detector reader passes, then the later holder check stalls.
  const f = await flashFixture(t); f.setNow(f.finishedAt + 59000);
  f.beforeHolder(count => { if (count === 3) f.setNow(f.finishedAt + 60001); });
  // Act / Assert
  await assert.rejects(admitExecution(f.root, "flash", 0, f.permit, f.operations), { code: "noise_detector_stale" });
  assert.equal(f.holderChecks(), 4);
  assert.equal((await proof(f.root, "install-0.claim.json")).sha256, f.permit.claimSha256);
  await assert.rejects(stat(resolve(f.root, "install-0")), { code: "ENOENT" });
});

test("fresh changed detector identity cannot replace the parent-pinned physical device", async t => {
  // Arrange
  const f = await flashFixture(t); f.setNow(f.finishedAt + 1);
  const path = resolve(f.root, "install-0.detect.stdout.log");
  const originalLog = await readFile(path, "utf8");
  await f.put(path, originalLog.replace("c".repeat(64), "d".repeat(64)));
  // Act / Assert
  await assert.rejects(admitExecution(f.root, "flash", 0, f.permit, f.operations), { code: "v2_execute_detector" });
  assert.equal(f.holderChecks(), 0);
  assert.equal((await proof(f.root, "install-0.claim.json")).sha256, f.permit.claimSha256);
  await assert.rejects(stat(resolve(f.root, "install-0")), { code: "ENOENT" });
});
