import assert from "node:assert/strict";
import test from "node:test";
import { chmod, readFile, writeFile, readdir, symlink, mkdir, rename } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { failedNetworkRestartFixture, networkRestartFixture } from "./reset-origin-restart-network-fixtures.mjs";
import {
  inspectRestartNetworkFailure,
  readRestartNetworkFailure,
  RESTART_NETWORK_FAILURE_SHA256,
} from "./reset-origin-restart-network-failure.mjs";
import { readRestartInstallPredecessor, restartPreflight, loadRestartContext } from "./reset-origin-restart-context.mjs";
import { inspectPreinstallFailure } from "./reset-origin-restart-preinstall.mjs";
import { knownFailureDiagnostics } from "./reset-origin-restart-successor-fixtures.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";

test("network failure audit preserves active statistics and cannot turn a synthetic or coherently resealed failure into the anchored predecessor", async (t) => {
  // Arrange
  const f = await failedNetworkRestartFixture(t);
  // Act / Assert
  const facts = await inspectRestartNetworkFailure(f.root, f.context);
  assert.equal(facts.known_failure.first_failure, "network");
  assert.equal(facts.known_failure.statistics_active.state, "active");
  await assert.rejects(readRestartNetworkFailure(f.root, f.operations), { code: "restart_network_failure_anchor" });
  const capture = resolve(f.root, "install-001/flash-monitor.log");
  await writeFile(capture, (await readFile(capture, "utf8")) + "\n");
  await assert.rejects(inspectRestartNetworkFailure(f.root, f.context), { code: "restart_network_inventory" });
  const sealPath = resolve(f.root, "failed-inventory.json"),
    seal = await readJson(sealPath);
  seal.files = await inventory(f.root);
  await writeFile(sealPath, JSON.stringify(seal));
  await assert.rejects(readRestartInstallPredecessor(f.root, f.operations), { code: "restart_install_failure_anchor" });
});
test("attempt3 uses the failed002 installed pair and preserves earlier exclusive assignments", async (t) => {
  // Arrange
  const f = await networkRestartFixture(t),
    markers = [`${f.stageARoot}.restart-assignment.json`, `${f.stageARoot}.restart-assignment-2.json`],
    hashes = await Promise.all(markers.map(fileDigest));
  // Act
  const context = await loadRestartContext(f.root, { operations: f.operations });
  // Assert
  assert.equal(context.restart_attempt, 3);
  assert.equal(context.before_source.firmware_commit, f.old.context.firmware_commit);
  assert.equal(context.gate_commit, f.old.context.gate_commit);
  assert.equal(context.before_install_failure.first_failure, "network");
  assert.equal(context.install_failure_predecessor.failed_inventory_sha256, RESTART_NETWORK_FAILURE_SHA256);
  assert.notEqual(context.request_nonce, f.old.context.request_nonce);
  assert.notEqual(context.restart_id, f.old.context.restart_id);
  assert.deepEqual(await Promise.all(markers.map(fileDigest)), hashes);
  await assert.rejects(restartPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "duplicate-network") }, f.operations), {
    code: "EEXIST",
  });
});
test("known network inspection accepts only the exact Gate shape, same boot, current identity and original active statistics receipt", async (t) => {
  // Arrange
  const f = await networkRestartFixture(t),
    input = knownFailureDiagnostics(f.context);
  // Act / Assert
  assert.equal(inspectPreinstallFailure(input, f.context).find((v) => v.category === "network_failure").phase, "reconnect_spawn");
  for (const mutate of [
    (v) => (v.observations.find((d) => d.category === "network_failure").phase = "driver_start"),
    (v) => (v.observations.find((d) => d.category === "network_failure").error = "timeout"),
    (v) => (v.observations.find((d) => d.category === "network_failure").private = "forbidden"),
    (v) => v.observations.find((d) => d.category === "boot").boot_ordinal++,
    (v) => (v.observations.find((d) => d.category === "boot").uptime_ms = 0),
    (v) => (v.observations.find((d) => d.category === "startup").first_failure = "none"),
    (v) => (v.observations.find((d) => d.category === "statistics_startup").state = "prepared"),
    (v) => v.observations.find((d) => d.category === "statistics_startup").before_free_bytes++,
    (v) => (v.observations = v.observations.filter((d) => d.category !== "network_failure")),
    (v) => (v.observations = v.observations.filter((d) => d.category !== "statistics_startup")),
  ]) {
    const bad = structuredClone(input);
    mutate(bad);
    assert.throws(() => inspectPreinstallFailure(bad, f.context));
  }
  assert.throws(() => inspectPreinstallFailure(input, { ...f.context, restart_attempt: 2 }));
});
test("network assignment stays consumed across interrupted creation and partial copying", async (t) => {
  // Arrange
  const f = await networkRestartFixture(t, { preflight: false });
  // Act / Assert
  await assert.rejects(
    restartPreflight(f.options, {
      ...f.operations,
      mkdir: async () => {
        throw Error("fixture creation interruption");
      },
    }),
    /fixture creation interruption/u,
  );
  await assert.rejects(restartPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "alternative-network") }, f.operations), {
    code: "EEXIST",
  });
  await assert.rejects(loadRestartContext(f.root, { operations: f.operations }));
});
test("network lineage rejects class substitution, nonnumeric attempts, evidence mutation and serving a sealed root", async (t) => {
  // Arrange
  const f = await networkRestartFixture(t),
    path = resolve(f.root, "context.json"),
    original = await readFile(path),
    saved = JSON.parse(original);
  // Act / Assert
  for (const attempt of ["3", 4, 2]) {
    saved.context.restart_attempt = attempt;
    saved.sha256 = digest(JSON.stringify(saved.context));
    await writeFile(path, JSON.stringify(saved));
    await assert.rejects(loadRestartContext(f.root, { operations: f.operations }));
  }
  await writeFile(path, original);
  await loadRestartContext(f.root, { operations: f.operations });
  await writeNew(resolve(f.root, "failed-inventory.json"), {});
  await assert.rejects(loadRestartContext(f.root, { operations: f.operations }), { code: "private_path_exists" });
  await writeNew(resolve(f.old.root, "issued.json"), {});
  await assert.rejects(inspectRestartNetworkFailure(f.old.root, f.old.context), { code: "restart_network_inventory" });
});
test("network preflight rejects unchanged firmware and archived task before new reservation", async (t) => {
  // Arrange
  const f = await networkRestartFixture(t, { preflight: false }),
    { inspectStartupSources } = await import("./cadence-startup-context.mjs");
  // Act / Assert
  await assert.rejects(
    restartPreflight(
      { ...f.options },
      {
        ...f.operations,
        inspectSources: async (...args) => ({ ...(await inspectStartupSources(...args)), firmware_commit: f.old.context.firmware_commit }),
      },
    ),
    { code: "restart_new_pair_required" },
  );
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "# Archived task\n");
  await assert.rejects(restartPreflight({ ...f.options }, f.operations), { code: "cadence_active_task_required" });
  assert.equal(
    (await readdir(dirname(f.root))).some((name) => name.endsWith("restart-assignment-3.json")),
    false,
  );
});
test("failure dispatch refuses root aliases, public permissions and unknown seal before reading context", async (t) => {
  // Arrange
  const f = await failedNetworkRestartFixture(t),
    alias = resolve(dirname(f.root), "alias-network");
  await symlink(f.root, alias);
  // Act / Assert
  await assert.rejects(readRestartInstallPredecessor(alias), { code: "directory_alias" });
  const seal = resolve(f.root, "failed-inventory.json");
  await chmod(seal, 0o644);
  await assert.rejects(readRestartInstallPredecessor(f.root), { code: "private_path_policy" });
  await chmod(seal, 0o600);
  await writeFile(resolve(f.root, "context.json"), "not JSON");
  await assert.rejects(readRestartInstallPredecessor(f.root), { code: "restart_install_failure_anchor" });
});

test("partial artifact preparation leaves assignment3 reserved and the child unservable", async (t) => {
  // Arrange
  const f = await networkRestartFixture(t, { preflight: false }),
    backup = `${f.options.manifest}.fixture-backup`;
  // Act / Assert
  await assert.rejects(
    restartPreflight(f.options, {
      ...f.operations,
      mkdir: async (path, options) => {
        await mkdir(path, options);
        await rename(f.options.manifest, backup);
      },
    }),
    { code: "ENOENT" },
  );
  await rename(backup, f.options.manifest);
  assert.equal((await readdir(f.root)).includes("context.json"), true);
  await assert.rejects(loadRestartContext(f.root, { operations: f.operations }));
  await assert.rejects(restartPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "after-partial-copy") }, f.operations), {
    code: "EEXIST",
  });
});

test("restart failure supersession conflicts fail at CLI admission before nonexistent paths are read", async () => {
  // Arrange
  const { main } = await import("./main.mjs");
  // Act / Assert
  for (const flag of ["--supersede-restart", "--supersede-premining", "--supersede-unissued", "--supersede-startup"])
    await assert.rejects(
      main([
        "reset-origin-restart-preflight",
        "--private-root",
        "/missing",
        "--supersede-install-failure",
        "/missing-predecessor",
        flag,
        "/missing-conflict",
      ]),
      { code: "command_arguments" },
    );
});
