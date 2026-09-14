import assert from "node:assert/strict";
import { copyFile, mkdir, readFile, readdir, stat, symlink, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import {
  ACCEPTED_PREMINING_INVENTORY,
  closePremining,
  preminingClosurePath,
  readPremining,
  reviewPremining,
} from "./cadence-premining.mjs";
import { preminingFixture } from "./cadence-premining-fixtures.mjs";
import { cadencePreflight, validateCadenceContext } from "./cadence-preflight.mjs";
import { closeUnissued } from "./cadence-unissued.mjs";
import { finishIterative } from "./iterative-judge.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { main } from "./main.mjs";

async function successor(f) {
  await closePremining(f.root, f.operations);
  const input = resolve(f.base, "software-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: ["a".repeat(64)],
  });
  f.operations.inspectSources = async () => ({ ...f.snapshot, firmware_commit: "9".repeat(40) });
  const options = {
    ...f.options,
    privateRoot: resolve(f.base, "attempts/preparation-3"),
    input,
    supersedeUnissued: undefined,
    supersedePremining: preminingClosurePath(f.root),
  };
  return options;
}
async function rewriteJournal(f, change) {
  change(f.records);
  await writeFile(resolve(f.root, "iterative.samples.jsonl"), f.records.map((row) => JSON.stringify(row) + "\n").join(""));
}

test("fan-only stopping preparation is reviewed and closed outside its sealed root without authority", async (t) => {
  // Arrange
  const f = await preminingFixture(t),
    before = await inventory(f.root),
    sealHash = await fileDigest(resolve(f.root, "failed-inventory.json"));
  // Act
  const reviewed = await reviewPremining(f.root, f.operations),
    closed = await closePremining(f.root, f.operations),
    receipt = await readPremining(preminingClosurePath(f.root), f.operations);
  // Assert
  assert.deepEqual(closed, reviewed);
  assert.equal(closed.result, "unverified");
  assert.equal(closed.continuation_authority, false);
  assert.equal(closed.cycle_count, 4);
  assert.equal(closed.flash_count, 5);
  assert.equal(receipt.earliest_failure.category, "probe_admission");
  assert.equal(receipt.final_sequence, 10);
  assert.equal((await stat(preminingClosurePath(f.root))).mode & 0o777, 0o600);
  assert.deepEqual(await inventory(f.root), before);
  assert.equal(await fileDigest(resolve(f.root, "failed-inventory.json")), sealHash);
  assert.deepEqual(await reviewPremining(f.root, f.operations), reviewed);
  await assert.rejects(closePremining(f.root, f.operations), { code: "private_path_exists" });
});

test("default accepted legacy anchor rejects an otherwise internally valid different fixture", async (t) => {
  // Arrange
  const f = await preminingFixture(t),
    operations = { ...f.operations };
  delete operations.expectedPreminingInventorySha256;
  assert.notEqual(await fileDigest(resolve(f.root, "failed-inventory.json")), ACCEPTED_PREMINING_INVENTORY);
  // Act / Assert
  await assert.rejects(reviewPremining(f.root, operations), { code: "cadence_premining_accepted_seal_required" });
});

test("coherent source and regenerated legacy inventory cannot replace the accepted sealed failure", async (t) => {
  // Arrange
  const f = await preminingFixture(t);
  await writeNew(resolve(f.root, "extra-fact.json"), { observed: true });
  await writeFile(resolve(f.root, "failed-inventory.json"), JSON.stringify({ ...f.seal, inventory: await inventory(f.root) }));
  // Act / Assert
  await assert.rejects(closePremining(f.root, f.operations), { code: "cadence_premining_accepted_seal_required" });
});

test("issued, consumed, loaded, renewed, work, fault or later phases prevent pre-mining classification", async (t) => {
  // Arrange / Act / Assert
  for (const name of [
    "issued.json",
    "consumed.json",
    "result.json",
    "cadence-usb-arm.json",
    "cadence-mining-arm.json",
    "cadence-probes.jsonl",
    "iterative.fault.json",
  ]) {
    const f = await preminingFixture(t);
    await writeNew(resolve(f.root, name), {});
    await assert.rejects(closePremining(f.root, f.operations), { code: "private_path_exists" });
  }
  for (const change of [
    { status: "window_loaded" },
    { running: true },
    { renewalsConfirmed: 1 },
    { serialFailureCategory: "timeout" },
    { admissionFailureStage: "hello" },
  ]) {
    const f = await preminingFixture(t);
    await rewriteJournal(f, (rows) => Object.assign(rows[8].state, change));
    await assert.rejects(closePremining(f.root, f.operations));
  }
  const f = await preminingFixture(t);
  await rewriteJournal(f, (rows) => {
    rows[8].state.cadence.firstWorkObservedAtMs = 0;
  });
  await assert.rejects(closePremining(f.root, f.operations), { code: "unexpected_work_or_collected_summary" });
});

test("flash, cycle, artifact, observer, accounting and cleanup source tampering reject", async (t) => {
  // Arrange / Act / Assert
  for (const [name, mutate] of [
    [
      "flash-0/flash-command-evidence.json",
      (v) => {
        v.nvs_seed_status = "provided";
      },
    ],
    [
      "flash-4.observation.json",
      (v) => {
        v.failures.push({ category: "fixture" });
      },
    ],
    [
      "cycle-2.observation-audit.json",
      (v) => {
        v.before_state_sha256 = "0".repeat(64);
      },
    ],
    [
      "cycle-3.browser-observation.json",
      (v) => {
        v.result.responsePayloadBytes = 65535;
      },
    ],
    [
      "cadence-observer-result.json",
      (v) => {
        v.cleanupComplete = false;
      },
    ],
    [
      "operator-failure.json",
      (v) => {
        v.ledger.pending = true;
      },
    ],
    [
      "operator-cleanup.json",
      (v) => {
        v.serial_holders_absent = false;
      },
    ],
  ]) {
    const f = await preminingFixture(t),
      path = resolve(f.root, name),
      value = await readJson(path);
    mutate(value);
    await writeFile(path, JSON.stringify(value));
    await assert.rejects(reviewPremining(f.root, f.operations));
  }
  const f = await preminingFixture(t);
  await writeFile(resolve(f.root, "qualified-artifacts/firmware/firmware_elf.bin"), "changed");
  await assert.rejects(reviewPremining(f.root, f.operations), { code: "snapshot_file_integrity" });
});

test("sibling tamper, partial write, aliases and ancestry reject without editing source evidence", async (t) => {
  // Arrange / Act / Assert
  const f = await preminingFixture(t);
  await closePremining(f.root, f.operations);
  await assert.rejects(
    readPremining(preminingClosurePath(f.root), {
      ...f.operations,
      expectedPreminingInventorySha256: f.operations.expectedPreminingInventorySha256,
      preminingAncestors: [f.root],
    }),
    { code: "cadence_premining_recursive_lineage" },
  );
  const alternate = resolve(f.base, "other.premining-closure.json");
  await copyFile(preminingClosurePath(f.root), alternate);
  await assert.rejects(readPremining(alternate, f.operations), { code: "cadence_premining_closure_path" });
  await writeFile(preminingClosurePath(f.root), "{");
  await assert.rejects(reviewPremining(f.root, f.operations));
  await assert.rejects(closePremining(f.root, f.operations), { code: "private_path_exists" });
  const alias = resolve(f.base, "alias");
  await symlink(f.root, alias);
  await assert.rejects(reviewPremining(alias, f.operations), { code: "directory_alias" });
});

test("archived task cannot create closure while historical evidence remains reviewable", async (t) => {
  // Arrange
  const f = await preminingFixture(t);
  await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Active\n");
  // Act / Assert
  assert.equal((await reviewPremining(f.root, f.operations)).result, "unverified");
  await assert.rejects(closePremining(f.root, f.operations), { code: "cadence_active_task_required" });
});

test("pre-mining successor has fresh identity and preserves both original preparation markers", async (t) => {
  // Arrange
  const f = await preminingFixture(t),
    options = await successor(f),
    parent = resolve(f.base, "attempts");
  const oldHashes = await Promise.all(
    ["ordinal-16.json", "ordinal-16-preparation-2.json"].map((name) => fileDigest(resolve(parent, name))),
  );
  // Act
  await cadencePreflight(options, f.operations);
  const { context } = await readJson(resolve(options.privateRoot, "context.json"));
  // Assert
  assert.equal(context.preparation_attempt, 3);
  assert.equal(context.qualification_attempt.ordinal, 16);
  assert.equal(context.expected_charged_ms, 1200000);
  assert.notEqual(context.qualification_attempt.id, f.context.qualification_attempt.id);
  assert.equal(context.unissued_predecessor, undefined);
  assert.equal(context.premining_predecessor.root, f.root);
  await validateCadenceContext(options.privateRoot, context, { operations: f.operations });
  assert.deepEqual(
    await Promise.all(["ordinal-16.json", "ordinal-16-preparation-2.json"].map((name) => fileDigest(resolve(parent, name)))),
    oldHashes,
  );
  await assert.rejects(cadencePreflight({ ...options, privateRoot: resolve(parent, "duplicate") }, f.operations), {
    code: "private_path_exists",
  });
  await assert.rejects(validateCadenceContext(f.root, f.context, { operations: f.operations }), { code: "private_path_exists" });
});

test("reserved assignment survives interruption before child creation or after copying the observer", async (t) => {
  // Arrange / Act / Assert
  for (const stage of ["mkdir", "copyFile"]) {
    const f = await preminingFixture(t),
      options = await successor(f),
      operations = { ...f.operations, expectedPreminingInventorySha256: f.operations.expectedPreminingInventorySha256 };
    operations[stage] = async (...args) => {
      if (stage === "copyFile") await copyFile(...args);
      throw new Error("simulated interruption");
    };
    await assert.rejects(cadencePreflight(options, operations), /simulated interruption/u);
    const marker = await readJson(resolve(f.base, "attempts/ordinal-16-preparation-3.json"));
    assert.equal(marker.attempt_root, options.privateRoot);
    await assert.rejects(cadencePreflight({ ...options, privateRoot: resolve(f.base, "attempts/reassignment") }, f.operations), {
      code: "private_path_exists",
    });
    await assert.rejects(readFile(resolve(options.privateRoot, "context.json")), { code: "ENOENT" });
  }
});

test("manual progress, unchanged source, mixed predecessors and changed charged lineage cannot issue a successor", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["manual", "pair", "mixed", "ledger"]) {
    const f = await preminingFixture(t),
      options = await successor(f);
    if (mode === "manual") {
      const p = await readJson(options.input);
      p.reason = "manual_remediation";
      await writeFile(options.input, JSON.stringify(p));
    }
    if (mode === "pair") f.operations.inspectSources = async () => f.snapshot;
    if (mode === "mixed") options.supersedeUnissued = f.options.supersedeUnissued;
    if (mode === "ledger") f.previous.total_charged_ms++;
    await assert.rejects(cadencePreflight(options, f.operations));
    assert(!(await readdir(resolve(f.base, "attempts"))).includes("ordinal-16-preparation-3.json"));
  }
});

test("permission-only closure and funded judge remain inapplicable to this failed preparation", async (t) => {
  // Arrange
  const f = await preminingFixture(t);
  const { ledger } = await readJson(resolve(f.root, "operator-failure.json"));
  const cleanup = { ...(await readJson(resolve(f.root, "operator-cleanup.json"))), schema: "worker-unissued-cleanup-v1" };
  // Act / Assert
  await assert.rejects(closeUnissued(f.root, { ledger, original_budget: f.previous.original_budget, cleanup }, f.operations), {
    code: "private_path_exists",
  });
  await assert.rejects(finishIterative(f.root, f.context, resolve(f.base, "missing-review.json")), { code: "private_path_exists" });
});

test("effect-free CLI routes reject unrelated inputs and mixed supersession fails before filesystem use", async () => {
  // Arrange / Act / Assert
  for (const command of ["cadence-close-premining", "cadence-review-premining"])
    await assert.rejects(main([command, "--private-root", "/missing", "--pool-credentials", "/never-read"]), { code: "command_arguments" });
  await assert.rejects(cadencePreflight({ supersedeUnissued: "/old", supersedePremining: "/new" }), {
    code: "cadence_supersession_exclusive",
  });
});

test("actual command dispatch resolves relative process evidence against the bound firmware root from another cwd", async (t) => {
  // Arrange
  const f = await preminingFixture(t), cwd = process.cwd();
  const before = await inventory(f.root);
  const source = await readJson(resolve(f.root, "cycle-1.before_flash-source.json"));
  assert(source.process_observation.startsWith("../"));
  // Act
  let result;
  try {
    process.chdir(f.base);
    result = await main(["cadence-review-premining", "--private-root", f.root], f.operations);
  } finally {
    process.chdir(cwd);
  }
  // Assert
  assert.equal(result.result, "unverified");
  assert.equal(result.cycle_count, 4);
  assert.deepEqual(await inventory(f.root), before);
});

test("relative process evidence cannot escape its exact preparation root", async (t) => {
  // Arrange
  const f = await preminingFixture(t), path = resolve(f.root, "cycle-1.before_flash-source.json");
  const source = await readJson(path);
  source.process_observation = "../outside-preparation.json";
  await writeFile(path, JSON.stringify(source));
  // Act / Assert
  await assert.rejects(reviewPremining(f.root, f.operations), { code: "cycle_process_source_path" });
});
