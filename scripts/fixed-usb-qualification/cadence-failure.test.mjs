import assert from "node:assert/strict";
import test from "node:test";
import { chmod, copyFile, readFile, readdir, symlink, unlink, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { cadenceFailureFixture, failedCadenceFixture } from "./cadence-failure-fixtures.mjs";
import { readCadenceFailure, inspectCadenceFailure, CADENCE_FAILURE_SHA256 } from "./cadence-failure.mjs";
import { cadencePreflight, validateCadenceContext } from "./cadence-preflight.mjs";
import { fileDigest, readJson, writeNew } from "./contract.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { main } from "./main.mjs";

test("reader rederives the exact USB percentile failure and retains its absent after-ledger/nonqualification facts", async (t) => {
  // Arrange
  const f = await failedCadenceFixture(t);
  // Act
  const read = await inspectCadenceFailure(f.root, f.context, f.operations);
  // Assert
  assert.equal(read.seal.observations.usb_within_750_ms, 89);
  assert.equal(read.seal.observations.usb_intervals, 95);
  assert.equal(read.seal.observations.usb_probes, 12);
  assert.equal(read.seal.observations.strict_validator_failure, "cadence_phase_unqualified");
  assert.equal(read.seal.post_failure_ledger_observed, false);
  assert.equal(read.seal.continuation_authority, false);
  await assert.rejects(readCadenceFailure(f.root, f.operations), { code: "cadence_failure_anchor" });
});
test("new preparation3 changes runtime without copying restart success or earlier cycle evidence", async (t) => {
  // Arrange
  const f = await cadenceFailureFixture(t),
    oldMarker = resolve(dirname(f.old.root), "ordinal-17-preparation-2.json"),
    markerHash = await fileDigest(oldMarker);
  // Act
  await cadencePreflight(f.options, f.operations);
  const { context } = await readJson(resolve(f.root, "context.json"));
  await validateCadenceContext(f.root, context, { operations: f.operations });
  // Assert
  assert.equal(context.preparation_attempt, 3);
  assert.equal(context.qualification_attempt.ordinal, 17);
  assert.equal(context.qualification_attempt.maximumActiveMilliseconds, 180000);
  assert.equal(context.expected_charged_ms, 1380000);
  assert.equal(context.previous_receipt, f.old.context.previous_receipt);
  assert.equal(context.restart_predecessor, undefined);
  assert.equal(context.cycle_source, undefined);
  assert.notEqual(context.qualification_attempt.id, f.old.context.qualification_attempt.id);
  assert.notEqual(context.firmware_commit, f.old.context.firmware_commit);
  assert.notEqual(context.app_elf_sha256, f.old.context.app_elf_sha256);
  assert.equal(context.gate_commit, f.old.context.gate_commit);
  assert.equal(await fileDigest(oldMarker), markerHash);
  assert.equal(context.cadence_failure_predecessor.failed_inventory_sha256, CADENCE_FAILURE_SHA256);
  assert.equal(
    (await readdir(f.root)).some((name) => /^cycle-/.test(name)),
    false,
  );
  await assert.rejects(cadencePreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "duplicate") }, f.operations), {
    code: "private_path_exists",
  });
});
test("failure scope is fresh for every independent validation and detects later inventory drift", async (t) => {
  // Arrange
  const f = await cadenceFailureFixture(t);
  await cadencePreflight(f.options, f.operations);
  const { context } = await readJson(resolve(f.root, "context.json"));
  let calls = 0;
  const operations = {
    ...f.operations,
    readCadenceFailure: async (...args) => {
      calls++;
      return f.operations.readCadenceFailure(...args);
    },
  };
  // Act / Assert
  await validateCadenceContext(f.root, context, { operations });
  assert.equal(calls, 1);
  await writeNew(resolve(f.old.root, "unexpected-extra.json"), { unexpected: true });
  await assert.rejects(validateCadenceContext(f.root, context, { operations }), { code: "cadence_failure_evidence_changed" });
  assert.equal(calls, 2);
});
test("issued, pending or working failed evidence cannot become a new unspent preparation", async (t) => {
  // Arrange
  const f = await failedCadenceFixture(t),
    coolPath = resolve(f.root, "cooling.json"),
    coolBytes = await readFile(coolPath),
    cooling = JSON.parse(coolBytes);
  // Act / Assert
  cooling.budget_after.pending = true;
  await writeFile(coolPath, JSON.stringify(cooling));
  await assert.rejects(inspectCadenceFailure(f.root, f.context, f.operations), { code: "iterative_ledger_admission" });
  await writeFile(coolPath, coolBytes);
  const journal = resolve(f.root, "iterative.samples.jsonl"),
    original = await readFile(journal),
    rows = original.toString().trim().split("\n").map(JSON.parse);
  rows[10].state.running = true;
  await writeFile(journal, rows.map((v) => JSON.stringify(v) + "\n").join(""));
  await assert.rejects(inspectCadenceFailure(f.root, f.context, f.operations));
  await writeFile(journal, original);
  await writeNew(resolve(f.root, "issued.json"), {});
  await assert.rejects(inspectCadenceFailure(f.root, f.context, f.operations), { code: "private_path_exists" });
});
test("a coherent replacement inventory cannot satisfy the immutable production anchor", async (t) => {
  // Arrange
  const f = await failedCadenceFixture(t),
    sealPath = resolve(f.root, "failed-inventory.json"),
    seal = await readJson(sealPath);
  // Act
  await writeNew(resolve(f.root, "new-member.json"), {});
  seal.files = await inventory(f.root);
  await writeFile(sealPath, JSON.stringify(seal));
  // Assert
  await inspectCadenceFailure(f.root, f.context, f.operations);
  await assert.rejects(readCadenceFailure(f.root, f.operations), { code: "cadence_failure_anchor" });
});
test("only a distinct verified regression plus the failure seal admits the changed image", async (t) => {
  // Arrange
  const f = await cadenceFailureFixture(t),
    progress = await readJson(f.options.input);
  // Act / Assert
  for (const evidence of [[CADENCE_FAILURE_SHA256], ["1".repeat(64)]]) {
    await writeFile(f.options.input, JSON.stringify({ ...progress, evidence_sha256: evidence }));
    await assert.rejects(cadencePreflight({ ...f.options }, f.operations), { code: "cadence_failure_progress" });
  }
  await writeFile(f.options.input, JSON.stringify(progress));
  for (const change of [
    { firmware_commit: f.old.context.firmware_commit },
    { app_elf_sha256: f.old.context.app_elf_sha256 },
    { gate_commit: "0".repeat(40) },
    { trust_sha256: "0".repeat(64) },
  ])
    await assert.rejects(
      cadencePreflight({ ...f.options }, { ...f.operations, inspectSources: async () => ({ ...f.source, ...change }) }),
      { code: "cadence_failure_runtime_delta" },
    );
});
test("exclusive preparation3 survives interrupted creation and partial observer copying", async (t) => {
  // Arrange
  const f = await cadenceFailureFixture(t);
  // Act / Assert
  await assert.rejects(
    cadencePreflight(f.options, {
      ...f.operations,
      copyFile: async (...args) => {
        await copyFile(...args);
        throw Error("fixture interrupted after copy");
      },
    }),
    /fixture interrupted after copy/u,
  );
  assert.equal((await readdir(f.root)).includes("cadence-observer.bin"), true);
  assert.equal((await readdir(f.root)).includes("context.json"), false);
  await assert.rejects(cadencePreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "after-partial") }, f.operations), {
    code: "private_path_exists",
  });
});
test("unsafe paths and archived effects fail without changing old evidence", async (t) => {
  // Arrange
  const f = await cadenceFailureFixture(t),
    alias = resolve(dirname(f.old.root), "failure-alias");
  await symlink(f.old.root, alias);
  // Act / Assert
  await assert.rejects(readCadenceFailure(alias), { code: "directory_alias" });
  await chmod(resolve(f.old.root, "failed-inventory.json"), 0o644);
  await assert.rejects(readCadenceFailure(f.old.root), { code: "private_path_policy" });
  await chmod(resolve(f.old.root, "failed-inventory.json"), 0o600);
  await writeFile(
    resolve(f.options.firmwareRoot, "TASKS.md"),
    "## Future\n### task-cpu0-telemetry-cadence-qualification | archived fixture\n",
  );
  await assert.rejects(cadencePreflight(f.options, f.operations), { code: "cadence_active_task_required" });
});
test("read-only CLI rejects effect inputs and all supersession routes remain mutually exclusive", async () => {
  // Arrange / Act / Assert
  for (const flag of ["--authority-directory", "--pool-credentials", "--input", "--supersede-restart"])
    await assert.rejects(main(["cadence-review-failure", "--private-root", "/missing", flag, "/never-read"]), {
      code: "command_arguments",
    });
  for (const field of ["supersedeRestart", "supersedeStartup", "supersedePremining", "supersedeUnissued"])
    await assert.rejects(cadencePreflight({ privateRoot: "/missing", supersedeCadenceFailure: "/missing", [field]: "/missing" }), {
      code: "cadence_supersession_exclusive",
    });
});

test("new failure reader rejects missing artifacts, cleanup conflicts, grants, renewals, suppression and charged drift", async (t) => {
  // Arrange
  const f = await failedCadenceFixture(t),
    snapshot = await readJson(resolve(f.root, "artifact-snapshot.json"));
  const artifact = resolve(f.root, "qualified-artifacts", snapshot.files[0].path),
    cleanup = resolve(f.root, "operator-cleanup.json"),
    journal = resolve(f.root, "iterative.samples.jsonl"),
    cooling = resolve(f.root, "cooling.json");
  const originals = new Map(await Promise.all([artifact, cleanup, journal, cooling].map(async (path) => [path, await readFile(path)])));
  const changeJson = async (path, change) => {
    const value = JSON.parse(originals.get(path));
    change(value);
    await writeFile(path, JSON.stringify(value));
  };
  const changeState = async (change) => {
    const rows = originals.get(journal).toString().trim().split("\n").map(JSON.parse);
    change(rows[8].state);
    await writeFile(journal, rows.map((row) => JSON.stringify(row) + "\n").join(""));
  };
  const cases = [
    [artifact, () => unlink(artifact), "ENOENT"],
    [artifact, () => writeFile(artifact, Buffer.concat([originals.get(artifact), Buffer.from("\n")])), "snapshot_file_integrity"],
    [
      cleanup,
      () =>
        changeJson(cleanup, (value) => {
          value.browser_closed = false;
        }),
      "actual_cleanup",
    ],
    [
      journal,
      () =>
        changeState((state) => {
          state.failure = "connect_failed";
        }),
      "unexpected_work_or_failure",
    ],
    [
      journal,
      () =>
        changeState((state) => {
          state.status = "window_loaded";
        }),
      "unexpected_work_or_failure",
    ],
    [
      journal,
      () =>
        changeState((state) => {
          state.renewalsConfirmed = 1;
        }),
      "unexpected_work_or_failure",
    ],
    [
      journal,
      () =>
        changeState((state) => {
          state.heartbeatSuppressed = true;
        }),
      "unexpected_work_or_failure",
    ],
    [
      cooling,
      () =>
        changeJson(cooling, (value) => {
          value.budget_after.total_charged_ms++;
        }),
      "iterative_ledger_admission",
    ],
  ];
  // Act / Assert
  for (const [path, change, code] of cases) {
    try {
      await change();
      await assert.rejects(inspectCadenceFailure(f.root, f.context, f.operations), { code });
    } finally {
      await writeFile(path, originals.get(path), { mode: 0o600 });
    }
  }
  await inspectCadenceFailure(f.root, f.context, f.operations);
  await assert.rejects(
    inspectCadenceFailure(
      f.root,
      { ...f.context, cadence_failure_predecessor: { root: f.root, failed_inventory_sha256: CADENCE_FAILURE_SHA256 } },
      f.operations,
    ),
    { code: "cadence_failure_class" },
  );
});

test("preparation assignment survives interruption before child directory creation", async (t) => {
  // Arrange
  const f = await cadenceFailureFixture(t);
  // Act / Assert
  await assert.rejects(
    cadencePreflight(f.options, {
      ...f.operations,
      mkdir: async () => {
        throw Error("fixture interrupted before mkdir");
      },
    }),
    /fixture interrupted before mkdir/u,
  );
  assert.equal((await readdir(dirname(f.root))).includes(f.root.split("/").at(-1)), false);
  await assert.rejects(cadencePreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "after-before-mkdir") }, f.operations), {
    code: "private_path_exists",
  });
});

test("failure reader evaluates an inherited read-only restart adapter once per invocation", async (t) => {
  // Arrange
  const f = await failedCadenceFixture(t);
  let reads = 0;
  const inherited = Object.create(f.operations);
  Object.defineProperty(inherited, "readCadenceRestart", {
    enumerable: true,
    get() {
      reads++;
      return f.operations.readCadenceRestart;
    },
  });
  const operations = Object.create(inherited);
  Object.defineProperty(operations, "retainedReadOnlyOption", { value: true, enumerable: true });
  // Act / Assert
  await inspectCadenceFailure(f.root, f.context, operations);
  assert.equal(reads, 1);
  const path = resolve(f.root, "operator-cleanup.json"),
    cleanup = await readJson(path);
  await writeFile(path, JSON.stringify({ ...cleanup, browser_closed: false }));
  await assert.rejects(inspectCadenceFailure(f.root, f.context, operations), { code: "actual_cleanup" });
  assert.equal(reads, 2);
});

test("restart bytes changed after full ancestry validation cannot be substituted during seal reconstruction", async (t) => {
  // Arrange
  const f = await failedCadenceFixture(t);
  const operations = {
    ...f.operations,
    readCadenceRestart: async (path, ...args) => {
      const verified = await f.operations.readCadenceRestart(path, ...args);
      await writeFile(path, Buffer.concat([await readFile(path), Buffer.from("\n")]));
      return verified;
    },
  };
  // Act / Assert
  await assert.rejects(inspectCadenceFailure(f.root, f.context, operations), { code: "cadence_failure_restart_changed" });
});
