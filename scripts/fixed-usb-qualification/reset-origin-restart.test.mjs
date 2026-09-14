import assert from "node:assert/strict";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { inspectRestartStartup } from "./reset-origin-restart-startup.mjs";
import { validateRestartEvidence, validateFailedRestartEvidence } from "./reset-origin-restart-evidence.mjs";
import { validateRestartState } from "./reset-origin-restart-state.mjs";
import { restartPreflight, loadRestartContext, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { requireRestartInstallation, consumeRestartInstall } from "./reset-origin-restart-install.mjs";
import { restartPacket, startupCapture, restartFixture, installedRestartFixture, recordRestartFixture } from "./reset-origin-restart-fixtures.mjs";
import { recoveryState } from "./cadence-startup-fixtures.mjs";
import { main } from "./main.mjs";
import { readJson, writeNew } from "./contract.mjs";
import { saveRestartAccounting } from "./reset-origin-restart-state.mjs";
import { inspectStartupSources } from "./cadence-startup-context.mjs";
import { saveRestartFailure } from "./reset-origin-restart-failure.mjs";
const context = { firmware_commit: "a".repeat(40), app_elf_sha256: "b".repeat(64), gate_commit: "c".repeat(40), request_nonce: Buffer.alloc(16, 1).toString("base64url") };

test("startup review separates an initial panic category from an observed second boot", () => {
  // Arrange / Act / Assert
  const bytes = startupCapture(context, "panic");
  assert.equal(inspectRestartStartup(bytes, context).observed_transitions, 0);
  assert.throws(() => inspectRestartStartup(Buffer.from(bytes.toString().replaceAll("boot_ordinal=7 reset_reason=panic uptime_ms=3000", "boot_ordinal=8 reset_reason=panic uptime_ms=3000")), context), { code: "restart_install_boot_transition" });
});
test("startup review rejects actual panic, failed startup, wrong identity and truncated-only proof", () => {
  // Arrange / Act / Assert
  for (const bytes of [Buffer.concat([startupCapture(context), Buffer.from("rust_panic_receipt schema=v1 file_hash=12345678 line=1 redacted=true\n")]),
    Buffer.from(startupCapture(context).toString().replace("first_failure=none", "first_failure=hardware")),
    startupCapture({ ...context, firmware_commit: "d".repeat(40) }), Buffer.from("usb_runtime_identity schema=v1")])
    assert.throws(() => inspectRestartStartup(bytes, context));
});
test("restart evidence proves coalesced ACK order and fresh post-reopen readiness", () => {
  // Arrange / Act / Assert
  for (const reopen of [false, true]) assert.equal(validateRestartEvidence(restartPacket(context, { reopen }), context, 7).summary.portReopens, Number(reopen));
  const coalesced = restartPacket(context);
  for (const row of [...coalesced.observations, ...coalesced.lifecycle.slice(0, -1)]) row.atMs = 0;
  assert.equal(validateRestartEvidence(coalesced, context, 7).summary.ackMatched, true);
  const stale = restartPacket(context, { reopen: true }); stale.observations = stale.observations.slice(0, 4);
  assert.throws(() => validateRestartEvidence(stale, context, 7), { code: "restart_fresh_boot_missing" });
});
test("restart evidence rejects wrong ACK, extra boot, false continuity and missing startup advance", () => {
  // Arrange / Act / Assert
  for (const mode of ["ack", "boot", "continuity", "startup", "bounds"]) {
    const value = restartPacket(context);
    if (mode === "ack") value.ack.requestNonceSha256 = "0".repeat(64);
    if (mode === "boot") value.observations[0].diagnostic.boot_ordinal = 9;
    if (mode === "continuity") value.summary.continuity = "same_port_reopened";
    if (mode === "startup") value.observations.at(-1).diagnostic.uptime_ms = 1000;
    if (mode === "bounds") value.summary.bytes = 262145;
    assert.throws(() => validateRestartEvidence(value, context, 7));
  }
});
test("failed observer counters remain intact without becoming passing evidence", () => {
  // Arrange
  const value = restartPacket(context); value.summary.stage = "failed"; value.summary.bytes = 300000;
  value.lifecycle.at(-1).event = "failed";
  // Act / Assert
  assert.equal(validateFailedRestartEvidence(value, context, 7).summary.bytes, 300000);
  assert.throws(() => validateRestartEvidence(value, context, 7));
});
test("new state schema preserves the prearm restarting state before a summary exists", () => {
  // Arrange
  const state = { ...recoveryState(context), status: "restarting", deviceBaselineConfirmed: false };
  // Act / Assert
  assert.equal(validateRestartState(state, context).status, "restarting");
});
test("restart preflight freezes both firmware identities and rejects reused assignment", async t => {
  // Arrange
  const f = await restartFixture(t);
  // Act
  const loaded = await loadRestartContext(f.root, { operations: f.operations });
  // Assert
  assert.equal(restartInnerContext(f.root, loaded, "before-install").firmware_commit, f.context.before_source.firmware_commit);
  assert.notEqual(loaded.firmware_commit, loaded.before_source.firmware_commit);
  assert.equal((await readJson(resolve(f.root, "artifact-snapshot.json"))).files.length, 13);
  await assert.rejects(restartPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), { code: "EEXIST" });
});
test("restart assignment remains consumed after partial preparation", async t => {
  // Arrange
  const f = await restartFixture(t, { preflight: false });
  // Act / Assert
  await assert.rejects(restartPreflight(f.options, { ...f.operations, mkdir: async () => { throw Error("fixture interrupted"); } }), /fixture interrupted/u);
  await assert.rejects(restartPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), { code: "EEXIST" });
});
test("factual installation preserves the original unqualified grade and cannot flash twice", async t => {
  // Arrange
  const f = await installedRestartFixture(t, { panic: true });
  // Act
  const evidence = await requireRestartInstallation(f.root, f.context);
  // Assert
  assert.equal(evidence.legacy_qualified, false);
  assert.equal((await readJson(resolve(f.root, "install-001/flash-command-evidence.json"))).commit_ready, false);
  await assert.rejects(consumeRestartInstall(f.root, f.operations));
});
test("every restart CLI rejects Work Lease signing and pool inputs before reading paths", async () => {
  // Arrange / Act / Assert
  for (const command of ["preflight", "serve", "consume-install", "install-review", "judge", "review"])
    for (const flag of ["--authority-directory", "--pool-credentials"])
      await assert.rejects(main([`reset-origin-restart-${command}`, "--private-root", "/missing", flag, "/never-read"]), { code: "command_arguments" });
});

test("startup profiles cannot name a different boot before the first discriminator", () => {
  // Arrange
  const capture = startupCapture(context).toString(), profile = capture.split("\n").find(line => line.startsWith("usb_boot_profile="));
  const wrong = profile.replace('"boot_ordinal":7', '"boot_ordinal":6');
  // Act / Assert
  assert.throws(() => inspectRestartStartup(Buffer.from(`${wrong}\n${capture}`), context), { code: "restart_install_profile_boot" });
});
test("restart rejects decreasing uptime even when the ordinal and reset category are repeated", () => {
  // Arrange
  const value = restartPacket(context, { reopen: true });
  value.observations[4].diagnostic.uptime_ms = 1;
  // Act / Assert
  assert.throws(() => validateRestartEvidence(value, context, 7), { code: "restart_boot_uptime_regression" });
});

test("initial panic cannot excuse failed, pending, dry-run or interactive capture", async t => {
  // Arrange
  const f = await installedRestartFixture(t, { panic: true }), path = resolve(f.root, "install-001/flash-command-evidence.json"), original = await readJson(path);
  // Act / Assert
  for (const change of [{ capture_status: "failed" }, { capture_status: "timed_out_pending_private_classification" }, { capture_status: "dry_run" }, { capture_mode: "interactive" }]) {
    await writeFile(path, JSON.stringify({ ...original, ...change }));
    await assert.rejects(requireRestartInstallation(f.root, f.context));
  }
});


test("installation claim rejects failure persisted during deferred source verification", async t => {
  // Arrange
  const f = await restartFixture(t), before = restartInnerContext(f.root, f.context, "before-install"), scope = resolve(f.root, "before-install");
  await recordRestartFixture(scope, before, recoveryState(before));
  await saveRestartAccounting(scope, before, { stage: "before", ledger: f.ledger, original_budget: f.original, state: recoveryState(before) });
  await recordRestartFixture(scope, before, recoveryState(before, true));
  const owner = { pid: 101, pgid: 101, startedAt: "fixture-process" };
  await writeNew(resolve(f.root, "install-001.host-root.json"), owner);
  await writeNew(resolve(f.root, "install-001.observer-armed.json"), owner);
  const entered = Promise.withResolvers(), resumed = Promise.withResolvers();
  const pending = consumeRestartInstall(f.root, { ...f.operations, inspectSources: async options => {
    const source = await inspectStartupSources(options, f.operations);
    entered.resolve(); await resumed.promise;
    return source;
  } });
  const rejected = assert.rejects(pending, { code: "private_path_exists" });
  // Act
  await entered.promise;
  try { await saveRestartFailure(f.root, f.context, "fixture_terminal_failure"); }
  finally { resumed.resolve(); }
  await rejected;
  // Assert
  await assert.rejects(readFile(resolve(f.root, "install-consumed.json")), { code: "ENOENT" });
  assert.equal((await readJson(resolve(f.root, "restart-failure.json"))).code, "fixture_terminal_failure");
});
