import assert from "node:assert/strict";
import { copyFile, readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { completeRecoveryFixture } from "./cadence-startup-fixtures.mjs";
import { judgeStartupRecovery } from "./cadence-startup-recovery.mjs";
import { cadencePreflight, validateCadenceContext } from "./cadence-preflight.mjs";
import { fileDigest, readJson, writeNew } from "./contract.mjs";
import { main } from "./main.mjs";

async function next(t) {
  const f = await completeRecoveryFixture(t);
  await judgeStartupRecovery(f.root, f.cleanupPath, f.operations);
  const input = resolve(f.base, "cadence-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: ["e".repeat(64)],
  });
  f.operations.inspectSources = async () => f.source;
  return {
    f,
    options: {
      ...f.options,
      privateRoot: resolve(f.base, "attempts/cadence-preparation-2"),
      input,
      supersedeStartup: resolve(f.root, "result.json"),
      authorityDirectory: resolve(f.base, "authority"),
      suggestedDifficulty: "1000",
      observerBinary: f.observer,
    },
  };
}
test("recovered pair admits fresh ordinal17 preparation2 without replacing failed assignment or importing cycles", async (t) => {
  // Arrange
  const { f, options } = await next(t),
    marker = resolve(f.base, "attempts/ordinal-17.json"),
    oldHash = await fileDigest(marker),
    recoveryHash = await fileDigest(options.supersedeStartup);
  // Act
  await cadencePreflight(options, f.operations);
  const { context } = await readJson(resolve(options.privateRoot, "context.json"));
  // Assert
  assert.equal(context.qualification_attempt.ordinal, 17);
  assert.equal(context.preparation_attempt, 2);
  assert.equal(context.expected_charged_ms, 1380000);
  assert.notEqual(context.qualification_attempt.id, f.failedContext.qualification_attempt.id);
  assert.equal(context.startup_predecessor.receipt_sha256, recoveryHash);
  assert.equal(context.cycle_source, undefined);
  assert.equal(await fileDigest(marker), oldHash);
  await validateCadenceContext(options.privateRoot, context, { operations: f.operations });
  await assert.rejects(cadencePreflight({ ...options, privateRoot: resolve(f.base, "attempts/duplicate") }, f.operations), {
    code: "private_path_exists",
  });
});
test("startup successor requires exact recovered artifacts, same charged predecessor and software progress", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["pair", "artifact", "ledger", "manual"]) {
    const { f, options } = await next(t);
    if (mode === "pair") f.operations.inspectSources = async () => ({ ...f.source, firmware_commit: "7".repeat(40) });
    if (mode === "artifact") f.operations.inspectSources = async () => ({ ...f.source, app_elf_sha256: "7".repeat(64) });
    if (mode === "ledger") f.previous.total_charged_ms++;
    if (mode === "manual") {
      const p = await readJson(options.input);
      p.reason = "manual_remediation";
      await writeFile(options.input, JSON.stringify(p));
    }
    await assert.rejects(cadencePreflight(options, f.operations));
    assert(!(await readdir(resolve(f.base, "attempts"))).includes("ordinal-17-preparation-2.json"));
  }
});
test("startup preparation assignment remains consumed after partial child creation or artifact copying", async (t) => {
  // Arrange / Act / Assert
  for (const stage of ["mkdir", "copyFile"]) {
    const { f, options } = await next(t),
      operations = { ...f.operations };
    operations[stage] = async (...args) => {
      if (stage === "copyFile") await copyFile(...args);
      throw new Error("interrupted fixture");
    };
    await assert.rejects(cadencePreflight(options, operations), /interrupted fixture/u);
    assert.equal((await readJson(resolve(f.base, "attempts/ordinal-17-preparation-2.json"))).attempt_root, options.privateRoot);
    await assert.rejects(cadencePreflight({ ...options, privateRoot: resolve(f.base, "attempts/alternate") }, f.operations), {
      code: "private_path_exists",
    });
    await assert.rejects(readFile(resolve(options.privateRoot, "context.json")), { code: "ENOENT" });
  }
});
test("main rejects conflicting startup supersession before nonexistent paths are inspected", async () => {
  // Arrange
  const common = [
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
    "--supersede-startup",
    "/missing",
  ];
  // Act / Assert
  for (const other of ["--supersede-unissued", "--supersede-premining"])
    await assert.rejects(main([...common, other, "/missing"]), { code: "cadence_supersession_exclusive" });
});
