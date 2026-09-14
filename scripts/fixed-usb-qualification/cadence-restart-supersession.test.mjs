import { RESTART_INSTALL_FAILURE_SHA256 } from "./reset-origin-restart-install-failure.mjs";
import assert from "node:assert/strict";
import { copyFile, readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { cadencePreflight, validateCadenceContext } from "./cadence-preflight.mjs";
import { cadenceRestartFixture } from "./cadence-restart-supersession-fixtures.mjs";
import {
  CADENCE_RESTART_STAGE_A_SHA256,
  readCadenceRestart,
  validateCadenceRestartEvidence,
  requireCadenceRestartLineage,
} from "./cadence-restart-supersession.mjs";
import { RESET_ORIGIN_SOURCE_SEAL } from "./reset-origin-runtime-source.mjs";
import { STARTUP_FAILURE_SHA256 } from "./cadence-startup-failure.mjs";
import { fileDigest, readJson } from "./contract.mjs";
import { main } from "./main.mjs";

function verifiedFacts(f) {
  const receipt = structuredClone(f.prior.receipt),
    stage = structuredClone(f.prior.stage_a),
    failed = f.failedContext;
  receipt.context.stage_a.sha256 = CADENCE_RESTART_STAGE_A_SHA256;
  stage.context.observation_attempt = 5;
  stage.context.runtime_source.failed_inventory_sha256 = RESET_ORIGIN_SOURCE_SEAL;
  return [
    receipt,
    stage,
    {
      failed_inventory_sha256: STARTUP_FAILURE_SHA256,
      previous_receipt: failed.previous_receipt,
      previous_receipt_sha256: failed.previous_receipt_sha256,
    },
    failed,
    CADENCE_RESTART_STAGE_A_SHA256,
    STARTUP_FAILURE_SHA256,
  ];
}

test("verified StageB admits fresh preparation2 of ordinal17 without replacing its original assignment or importing cycles", async (t) => {
  // Arrange
  const f = await cadenceRestartFixture(t),
    marker = resolve(f.base, "attempts/ordinal-17.json"),
    before = await fileDigest(marker);
  // Act
  await cadencePreflight(f.options, f.operations);
  const { context } = await readJson(resolve(f.options.privateRoot, "context.json"));
  await validateCadenceContext(f.options.privateRoot, context, { operations: f.operations });
  // Assert
  assert.equal(context.preparation_attempt, 2);
  assert.equal(context.qualification_attempt.ordinal, 17);
  assert.equal(context.qualification_attempt.maximumActiveMilliseconds, 180000);
  assert.equal(context.required_no_mining_cycles, 4);
  assert.notEqual(context.qualification_attempt.id, f.failedContext.qualification_attempt.id);
  assert.equal(context.expected_charged_ms, 1380000);
  assert.equal(context.original_campaign_id, f.failedContext.original_campaign_id);
  assert.equal(context.previous_receipt, f.failedContext.previous_receipt);
  assert.equal(context.cycle_source, undefined);
  assert.notEqual(context.supervisor_client_sha256, f.context.supervisor_client_sha256);
  assert.equal(await fileDigest(marker), before);
  assert.deepEqual(context.restart_predecessor, f.prior.binding);
  for (const key of ["previous_receipt", "previous_receipt_sha256", "original_campaign_id", "expected_charged_ms"]) {
    const changed = { ...context, [key]: key === "expected_charged_ms" ? 0 : "changed" };
    assert.throws(() => requireCadenceRestartLineage(f.options.privateRoot, changed, f.prior), { code: "cadence_restart_ledger_lineage" });
  }
  await assert.rejects(cadencePreflight({ ...f.options, privateRoot: resolve(f.base, "attempts/duplicate") }, f.operations), {
    code: "private_path_exists",
  });
});

test("production reader retains exact actual StageA/recovery/original-preparation anchors", async (t) => {
  // Arrange
  const f = await cadenceRestartFixture(t),
    valid = verifiedFacts(f);
  // Act / Assert
  validateCadenceRestartEvidence(...valid);
  const successor = structuredClone(valid);
  Object.assign(successor[0].context, {
    restart_attempt: 2,
    statistics_startup_required: true,
    install_failure_predecessor: { root: "/verified-fixture", failed_inventory_sha256: RESTART_INSTALL_FAILURE_SHA256 },
  });
  validateCadenceRestartEvidence(...successor);
  successor[0].context.install_failure_predecessor.failed_inventory_sha256 = "0".repeat(64);
  assert.throws(() => validateCadenceRestartEvidence(...successor), { code: "cadence_restart_successor_class" });
  for (const change of [
    (args) => {
      args[4] = "0".repeat(64);
    },
    (args) => {
      args[1].context.observation_attempt = 4;
    },
    (args) => {
      args[1].context.runtime_source.failed_inventory_sha256 = "0".repeat(64);
    },
    (args) => {
      args[5] = "0".repeat(64);
    },
    (args) => {
      args[0].result = "unverified";
    },
    (args) => {
      args[0].ledger.pending = true;
    },
    (args) => {
      args[0].final_state.serialOwnershipReleased = false;
    },
  ]) {
    const args = structuredClone(valid);
    change(args);
    assert.throws(() => validateCadenceRestartEvidence(...args));
  }
  // Synthetic receipt hashes cannot pass the production reader without the explicit trusted-reader fixture boundary.
  await assert.rejects(readCadenceRestart(f.receiptPath, f.operations));
});

test("restart handoff rejects changed runtime artifacts, charged ancestry, and nonsoftware progress before assignment", async (t) => {
  // Arrange
  const f = await cadenceRestartFixture(t),
    originalPlan = await readJson(f.options.input);
  // Act / Assert
  for (const key of ["firmware_commit", "gate_commit", "manifest_sha256", "app_elf_sha256", "trust_sha256", "update_segments"]) {
    const source = { ...f.source, [key]: key === "update_segments" ? [] : "0".repeat(key.endsWith("commit") ? 40 : 64) };
    await assert.rejects(cadencePreflight(f.options, { ...f.operations, inspectSources: async () => source }), {
      code: "cadence_restart_pair_changed",
    });
  }
  await writeFile(f.options.input, JSON.stringify({ ...originalPlan, reason: "manual_remediation" }));
  await assert.rejects(cadencePreflight(f.options, f.operations));
  assert(!(await readdir(resolve(f.base, "attempts"))).includes("ordinal-17-preparation-2.json"));
});

test("restart preparation reservation survives interrupted creation/copy and cannot be reassigned", async (t) => {
  // Arrange / Act / Assert
  for (const stage of ["mkdir", "copyFile"]) {
    const f = await cadenceRestartFixture(t),
      operations = { ...f.operations };
    operations[stage] = async (...args) => {
      if (stage === "copyFile") await copyFile(...args);
      throw Error("fixture interruption");
    };
    await assert.rejects(cadencePreflight(f.options, operations), /fixture interruption/u);
    assert.equal((await readJson(resolve(f.base, "attempts/ordinal-17-preparation-2.json"))).attempt_root, f.options.privateRoot);
    await assert.rejects(cadencePreflight({ ...f.options, privateRoot: resolve(f.base, "attempts/reassigned") }, f.operations), {
      code: "private_path_exists",
    });
    await assert.rejects(readFile(resolve(f.options.privateRoot, "context.json")), { code: "ENOENT" });
  }
});

test("mutated restart evidence blocks subsequent context validation and archived tasks block fresh preparation", async (t) => {
  // Arrange
  const f = await cadenceRestartFixture(t);
  await cadencePreflight(f.options, f.operations);
  const { context } = await readJson(resolve(f.options.privateRoot, "context.json"));
  // Act / Assert
  await writeFile(resolve(f.root, "host-cleanup.json"), "{}\n");
  await assert.rejects(validateCadenceContext(f.options.privateRoot, context, { operations: f.operations }));
  const another = await cadenceRestartFixture(t);
  await writeFile(resolve(another.options.firmwareRoot, "TASKS.md"), "## Active\n");
  await assert.rejects(cadencePreflight(another.options, another.operations), { code: "cadence_active_task_required" });
});

test("restart supersession conflicts with every existing route before nonexistent paths are inspected", async () => {
  // Arrange / Act / Assert
  for (const option of ["supersedeUnissued", "supersedePremining", "supersedeStartup"])
    await assert.rejects(cadencePreflight({ privateRoot: "/missing", supersedeRestart: "/missing", [option]: "/missing" }), {
      code: "cadence_supersession_exclusive",
    });
  const args = [
    "cadence-preflight",
    "--private-root",
    "/missing",
    "--firmware-root",
    "/missing",
    "--gate-root",
    "/missing",
    "--firmware-commit",
    "a",
    "--gate-commit",
    "b",
    "--manifest",
    "/missing",
    "--authority-directory",
    "/missing",
    "--previous-receipt",
    "/missing",
    "--input",
    "/missing",
    "--observer-binary",
    "/missing",
    "--supersede-restart",
    "/missing",
    "--supersede-startup",
    "/missing",
  ];
  await assert.rejects(main(args), { code: "cadence_supersession_exclusive" });
});
