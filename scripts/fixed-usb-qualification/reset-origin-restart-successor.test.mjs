import { completeResetOriginFixture } from "./reset-origin-fixtures.mjs";
import { inspectResetOriginObservation } from "./reset-origin-observation-review.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import assert from "node:assert/strict";
import test from "node:test";
import { readFile, writeFile, readdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import {
  failedRestartFixture,
  successorRestartFixture,
  knownFailureDiagnostics,
  statisticsActive,
  statisticsLine,
} from "./reset-origin-restart-successor-fixtures.mjs";
import { inspectRestartInstallFailure, readRestartInstallFailure } from "./reset-origin-restart-install-failure.mjs";
import { restartPreflight, loadRestartContext, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { savePreinstallFailure, requirePreinstallFailure, inspectPreinstallFailure } from "./reset-origin-restart-preinstall.mjs";
import { saveRestartAccounting } from "./reset-origin-restart-state.mjs";
import { recordRestartFixture, startupCapture, restartPacket } from "./reset-origin-restart-fixtures.mjs";
import { recoveryState } from "./cadence-startup-fixtures.mjs";
import { consumeRestartInstall } from "./reset-origin-restart-install.mjs";
import { parseStatisticsStartup, requireActiveStatistics } from "./reset-origin-restart-statistics.mjs";
import { inspectRestartStartup } from "./reset-origin-restart-startup.mjs";
import { validateRestartEvidence } from "./reset-origin-restart-evidence.mjs";
import { main } from "./main.mjs";
import { BUNDLE, digest, readJson, writeNew } from "./contract.mjs";

async function beforeAccounting(f) {
  const inner = restartInnerContext(f.root, f.context, "before-install"),
    scope = resolve(f.root, "before-install"),
    state = recoveryState(inner);
  await recordRestartFixture(scope, inner, state);
  await saveRestartAccounting(scope, inner, { stage: "before", ledger: f.ledger, original_budget: f.original, state });
  return { inner, scope, state };
}
test("exact failed installation reader preserves the failure and rejects synthetic or coherently resealed anchors", async (t) => {
  // Arrange
  const f = await failedRestartFixture(t);
  // Act / Assert
  const read = await inspectRestartInstallFailure(f.root, f.context);
  assert.equal(read.known_failure.first_failure, "statistics");
  await assert.rejects(readRestartInstallFailure(f.root, f.operations), { code: "restart_install_failure_anchor" });
  const input = resolve(f.root, "install-review-input.json");
  await writeFile(input, (await readFile(input, "utf8")) + "\n");
  await assert.rejects(inspectRestartInstallFailure(f.root, f.context), { code: "restart_install_failure_inventory" });
  const sealPath = resolve(f.root, "failed-inventory.json"),
    seal = await readJson(sealPath);
  seal.files = await inventory(f.root);
  await writeFile(sealPath, JSON.stringify(seal));
  await inspectRestartInstallFailure(f.root, f.context);
  await assert.rejects(readRestartInstallFailure(f.root, f.operations), { code: "restart_install_failure_anchor" });
});
test("successor reserves a distinct installation context on the currently installed firmware, preserving the original assignment", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t),
    marker = `${f.stageARoot}.restart-assignment.json`,
    original = await readFile(marker);
  // Act
  const context = await loadRestartContext(f.root, { operations: f.operations });
  // Assert
  assert.equal(context.restart_attempt, 2);
  assert.equal(context.statistics_startup_required, true);
  assert.equal(context.before_source.firmware_commit, f.old.context.firmware_commit);
  assert.notEqual(context.before_source.firmware_commit, f.old.context.before_source.firmware_commit);
  assert.notEqual(context.request_nonce, f.old.context.request_nonce);
  assert.deepEqual(await readFile(marker), original);
  await assert.rejects(restartPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), {
    code: "EEXIST",
  });
});
test("successor reservation survives interrupted child creation and partial contexts cannot serve", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t, { preflight: false });
  // Act / Assert
  await assert.rejects(
    restartPreflight(f.options, {
      ...f.operations,
      mkdir: async () => {
        throw Error("interrupted fixture");
      },
    }),
    /interrupted fixture/u,
  );
  await assert.rejects(restartPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), {
    code: "EEXIST",
  });
  await assert.rejects(loadRestartContext(f.root, { operations: f.operations }));
});
test("successor rejects retained firmware, missing correction anchor, archived task and predecessor mutation before reserving", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t, { preflight: false });
  const source = (await import("./cadence-startup-context.mjs")).inspectStartupSources;
  // Act / Assert
  await assert.rejects(
    restartPreflight(
      { ...f.options },
      {
        ...f.operations,
        inspectSources: async (...args) => ({ ...(await source(...args)), firmware_commit: f.old.context.firmware_commit }),
      },
    ),
    { code: "restart_new_pair_required" },
  );
  const progress = await readJson(f.options.input);
  progress.evidence_sha256 = ["0".repeat(64)];
  await writeFile(f.options.input, JSON.stringify(progress));
  await assert.rejects(restartPreflight({ ...f.options }, f.operations), { code: "restart_install_failure_relation" });
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "# No active task\n");
  await assert.rejects(restartPreflight({ ...f.options }, f.operations));
  assert.equal(
    (await readdir(dirname(f.root))).some((n) => n.endsWith("restart-assignment-2.json")),
    false,
  );
});
test("fresh failure review requires same captured boot and current identity without treating statistics failure as healthy", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t),
    before = await beforeAccounting(f),
    diagnostics = knownFailureDiagnostics(f.context);
  // Act
  const receipt = await savePreinstallFailure(f.root, f.context, diagnostics);
  await recordRestartFixture(before.scope, before.inner, recoveryState(before.inner, true));
  await requirePreinstallFailure(f.root, f.context, 2);
  // Assert
  assert.equal(receipt.startup_healthy, false);
  assert.equal(receipt.recovery_verified, false);
  for (const change of [
    (d) => d.observations[0].boot_ordinal++,
    (d) => (d.observations[0].uptime_ms = 0),
    (d) => (d.observations[1].firmware_commit = "0".repeat(40)),
    (d) => (d.observations[2].first_failure = "hardware"),
    (d) => (d.observations[2].first_failure = "none"),
    (d) => (d.observations[3].http_ready = "false"),
  ]) {
    const bad = structuredClone(diagnostics);
    change(bad);
    assert.throws(() => inspectPreinstallFailure(bad, f.context));
  }
  const contradictory = structuredClone(diagnostics);
  contradictory.observations.push({ ...contradictory.observations[1], firmware_commit: "0".repeat(40) });
  assert.throws(() => inspectPreinstallFailure(contradictory, f.context), { code: "restart_preinstall_duplicate_diagnostic" });
  await assert.rejects(savePreinstallFailure(f.root, f.context, diagnostics));
  await assert.rejects(requirePreinstallFailure(f.root, f.context, 1), { code: "restart_preinstall_review_order" });
});
test("install cannot consume before the fresh known-failure review even with authenticated accounting and released USB", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t),
    before = await beforeAccounting(f);
  await recordRestartFixture(before.scope, before.inner, recoveryState(before.inner, true));
  // Act / Assert
  await assert.rejects(consumeRestartInstall(f.root, f.operations));
  assert.equal((await readdir(f.root)).includes("install-consumed.json"), false);
});
test("successor shape and ancestor hashes are revalidated on each invocation", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t),
    path = resolve(f.root, "context.json"),
    saved = await readJson(path);
  // Act / Assert
  saved.context.restart_attempt = "2";
  saved.sha256 = digest(JSON.stringify(saved.context));
  await writeFile(path, JSON.stringify(saved));
  await assert.rejects(loadRestartContext(f.root, { operations: f.operations }), { code: "restart_successor_shape" });
  saved.context.restart_attempt = 2;
  saved.sha256 = digest(JSON.stringify(saved.context));
  await writeFile(path, JSON.stringify(saved));
  await loadRestartContext(f.root, { operations: f.operations });
  await writeNew(resolve(f.old.root, "issued.json"), {});
  await assert.rejects(loadRestartContext(f.root, { operations: f.operations }));
});
test("statistics startup uses coherent heap pairs without assuming a before-after allocation delta", () => {
  // Arrange
  const active = statisticsActive();
  // Act / Assert
  assert.deepEqual(parseStatisticsStartup(active), active);
  assert.equal(requireActiveStatistics([active]).stack_caps, 2052);
  for (const mutate of [
    (v) => (v.before_largest_block_bytes = v.before_free_bytes + 1),
    (v) => (v.errno = 12),
    (v) => (v.stack_bytes = 4096),
    (v) => (v.stack_caps = "unavailable"),
  ]) {
    const bad = { ...active };
    mutate(bad);
    assert.throws(() => parseStatisticsStartup(bad));
  }
  for (const value of [
    { ...active, state: "prepared" },
    { ...active, state: "cancelled" },
    { ...active, state: "spawn_failed", errno: 12 },
    { ...active, stack_caps: 2056 },
  ])
    assert.throws(() => requireActiveStatistics([value]));
});
test("corrected install requires an active statistics receipt while historical no-receipt captures remain valid", () => {
  // Arrange
  const context = { firmware_commit: "a".repeat(40), app_elf_sha256: "b".repeat(64) },
    bytes = startupCapture(context);
  // Act / Assert
  assert.equal(inspectRestartStartup(bytes, context).observed_transitions, 0);
  const next = { ...context, statistics_startup_required: true };
  assert.throws(() => inspectRestartStartup(bytes, next), { code: "statistics_startup_missing" });
  assert.equal(inspectRestartStartup(Buffer.concat([bytes, Buffer.from(statisticsLine())]), next).statistics_startup.state, "active");
  assert.throws(() => inspectRestartStartup(Buffer.concat([bytes, Buffer.from(statisticsLine("prepared"))]), next));
});
test("restart requires fresh active statistics after the expected boot and any reopen boundary", () => {
  // Arrange
  const context = {
    firmware_commit: "a".repeat(40),
    app_elf_sha256: "b".repeat(64),
    request_nonce: Buffer.alloc(16, 1).toString("base64url"),
    statistics_startup_required: true,
  };
  // Act / Assert
  for (const reopen of [false, true]) {
    const packet = restartPacket(context, { reopen });
    assert.throws(() => validateRestartEvidence(packet, context, 7), { code: "statistics_startup_missing" });
    // Replace the identity slot with active then move exact identity to a new record before Hello.
    const end = packet.lifecycle.find((v) => v.event === "hello_started"),
      record = end.record;
    packet.observations.push({ record: record + 1, atMs: end.atMs - 1, diagnostic: statisticsActive() });
    end.record++;
    packet.lifecycle.at(-1).record++;
    packet.summary.records++;
    assert.equal(validateRestartEvidence(packet, context, 7).summary.stage, "complete");
    packet.observations.at(-1).record = 1;
    assert.throws(() => validateRestartEvidence(packet, context, 7));
  }
});
test("install successor CLI flag is confined to restart preflight and cannot enable signing inputs", async () => {
  // Arrange / Act / Assert
  for (const command of ["serve", "consume-install", "install-review", "judge", "review"])
    await assert.rejects(
      main([`reset-origin-restart-${command}`, "--private-root", "/missing", "--supersede-install-failure", "/missing"]),
      { code: "command_arguments" },
    );
  await assert.rejects(
    main([
      "reset-origin-restart-preflight",
      "--private-root",
      "/missing",
      "--supersede-install-failure",
      "/missing",
      "--pool-credentials",
      "/private",
    ]),
    { code: "command_arguments" },
  );
});

test("independent observation validation cannot ignore failed statistics metadata in an otherwise complete capture", async (t) => {
  // Arrange
  const f = await completeResetOriginFixture(t),
    path = resolve(f.root, "diagnostic-export-0000.json"),
    batch = await readJson(path);
  batch.observations.push({ ...statisticsActive(), state: "spawn_failed", errno: 12 });
  await writeFile(path, JSON.stringify(batch));
  // Act / Assert
  await assert.rejects(
    inspectResetOriginObservation(
      f.root,
      f.context,
      await readJson(resolve(f.root, "reset-origin-start.json")),
      await readJson(resolve(f.root, "reset-origin-end.json")),
    ),
    { code: "statistics_startup_not_active" },
  );
});

test("one validation deduplicates the accepted StageA reader but the next invocation validates again", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t);
  let reads = 0;
  const operations = {
    ...f.operations,
    readStageA: async (path) => {
      reads++;
      return f.operations.readStageA(path);
    },
    readInstallFailure: async (root, scope) => {
      await loadRestartContext(root, { historical: true, operations: scope });
      return f.operations.readInstallFailure(root);
    },
  };
  // Act / Assert
  await loadRestartContext(f.root, { operations });
  assert.equal(reads, 1);
  await loadRestartContext(f.root, { operations });
  assert.equal(reads, 2);
});

test("missing statistics decoder capability rejects successor preflight before assignment or child creation", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t, { preflight: false });
  await writeFile(resolve(f.options.gateRoot, BUNDLE), `fixture qualificationRestart statistics ${f.options.gateCommit}`);
  // Act / Assert
  await assert.rejects(restartPreflight(f.options, f.operations), { code: "restart_statistics_capability_missing" });
  const siblings = await readdir(dirname(f.root));
  assert.equal(siblings.includes(f.root.split("/").at(-1)), false);
  assert.equal(
    siblings.some((name) => name.endsWith("restart-assignment-2.json")),
    false,
  );
});

test("live successor validation checks current decoder bytes while historical failed contexts remain valid", async (t) => {
  // Arrange
  const f = await successorRestartFixture(t);
  await writeFile(resolve(f.options.gateRoot, BUNDLE), `fixture qualificationRestart ${f.options.gateCommit}`);
  // Act / Assert
  await assert.rejects(loadRestartContext(f.root, { operations: f.operations }), { code: "restart_statistics_capability_missing" });
  const old = await loadRestartContext(f.old.root, { historical: true, operations: f.operations });
  assert.equal(old.restart_attempt, undefined);
});
