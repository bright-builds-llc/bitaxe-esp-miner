import assert from "node:assert/strict";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { inventory } from "./cadence-premining-evidence.mjs";
import { unusedObservation, preparationReview, journalFailure } from "./reset-origin-successor-fixtures.mjs";
import { resetOriginFixture } from "./reset-origin-fixtures.mjs";
import { loadResetOriginContext, resetOriginPreflight, RESET_ORIGIN_POLICY } from "./reset-origin-context.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";

test("observe-only preflight separates published host driver from unchanged retained runtime", async (t) => {
  // Arrange
  const f = await resetOriginFixture(t);
  // Act
  const context = await loadResetOriginContext(f.root, { operations: f.operations });
  // Assert
  assert.notEqual(context.qualification_driver.source_commit, context.firmware_commit);
  assert.equal(context.firmware_commit, f.context.firmware_commit);
  assert.deepEqual(context.observation_policy, RESET_ORIGIN_POLICY);
  assert.equal(context.mining_authorized, false);
  assert.equal(context.restart_authorized, false);
  assert.equal(context.installation_authorized, false);
  assert.equal(context.manifest, resolve(f.root, "qualified-artifacts/firmware/bitaxe-ultra205-package.json"));
  assert.equal((await readJson(resolve(f.root, "artifact-snapshot.json"))).files.length, 13);
  assert.equal(context.runtime_source.failed_inventory_sha256, await fileDigest(resolve(f.sourceRoot, "failed-inventory.json")));
});

test("driver permits only declared host changes and never firmware, trust, deletion or nonregular files", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["firmware", "trust", "delete", "symlink"]) {
    const f = await resetOriginFixture(t, { preflight: false });
    const base = f.operations.git;
    f.operations.git = (root, args) =>
      args[0] === "diff"
        ? mode === "firmware"
          ? "M\tfirmware/bitaxe/src/main.rs"
          : mode === "trust"
            ? "M\tfirmware/bitaxe/bwg/deployment-trust.json"
            : mode === "delete"
              ? "D\tTASKS.md"
              : "M\tTASKS.md"
        : args[0] === "ls-tree" && mode === "symlink"
          ? "120000 blob fixture"
          : base(root, args);
    await assert.rejects(resetOriginPreflight(f.options, f.operations), { code: "reset_origin_driver_forbidden_change" });
    assert(!(await readdir(dirname(f.root))).includes("observation"));
  }
});

test("transitive contract drift is detected even when the five new files are unchanged", async (t) => {
  // Arrange
  const f = await resetOriginFixture(t),
    path = resolve(f.options.firmwareRoot, "scripts/fixed-usb-qualification/contract.mjs");
  await writeFile(path, (await readFile(path, "utf8")) + "\n// fixture drift\n");
  // Act / Assert
  await assert.rejects(loadResetOriginContext(f.root, { operations: f.operations }), { code: "reset_origin_driver_drift" });
});

test("observation assignment cannot be reused after interruption before root creation", async (t) => {
  // Arrange
  const f = await resetOriginFixture(t, { preflight: false });
  // Act / Assert
  await assert.rejects(
    resetOriginPreflight(f.options, {
      ...f.operations,
      mkdir: async () => {
        throw Error("fixture interruption");
      },
    }),
    /fixture interruption/u,
  );
  await assert.rejects(resetOriginPreflight({ ...f.options, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), {
    code: "EEXIST",
  });
  await assert.rejects(readFile(resolve(f.root, "context.json")), { code: "ENOENT" });
});

test("source anchor, campaign, driver ancestry and active task remain mandatory", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["anchor", "campaign", "ancestry", "archived"]) {
    const f = await resetOriginFixture(t, { preflight: false });
    if (mode === "anchor") delete f.operations.expectedResetOriginSourceSeal;
    if (mode === "campaign")
      await writeFile(
        f.options.originalCampaignRecord,
        JSON.stringify({ schema: "fixed-usb-campaign-v1", campaign_id: Buffer.alloc(16, 8).toString("base64url") }),
      );
    if (mode === "ancestry") {
      const base = f.operations.git;
      f.operations.git = (root, args) => (args[0] === "merge-base" ? "0".repeat(40) : base(root, args));
    }
    if (mode === "archived") await writeFile(resolve(f.options.firmwareRoot, "TASKS.md"), "## Active\n");
    await assert.rejects(resetOriginPreflight(f.options, f.operations));
  }
});

test("authority and pool inputs fail before source or credential paths can be inspected", async () => {
  // Arrange / Act / Assert
  for (const field of ["authorityDirectory", "poolCredentials"])
    await assert.rejects(resetOriginPreflight({ privateRoot: "/missing", [field]: "/never-read" }), {
      code: "reset_origin_credentials_forbidden",
    });
});

test("exact unstarted failure supports a new driver and exclusive observation assignment without changing runtime or old evidence", async (t) => {
  // Arrange
  const f = await unusedObservation(t),
    old = await inventory(f.root),
    marker = await fileDigest(`${f.sourceRoot}.reset-origin-assignment.json`);
  // Act
  await resetOriginPreflight(f.successor, f.operations);
  const context = await loadResetOriginContext(f.successor.privateRoot, { operations: f.operations });
  // Assert
  assert.equal(context.observation_attempt, 2);
  assert.notEqual(context.observation_id, f.context.observation_id);
  assert.deepEqual(context.runtime_source, f.context.runtime_source);
  assert.equal(context.firmware_commit, f.context.firmware_commit);
  assert.equal(context.expected_next_ordinal, 17);
  assert.equal(context.expected_charged_ms, 1380000);
  assert.equal(context.mining_authorized, false);
  assert.deepEqual(await inventory(f.root), old);
  assert.equal(await fileDigest(`${f.sourceRoot}.reset-origin-assignment.json`), marker);
  assert.deepEqual(await loadResetOriginContext(f.root, { historical: true, operations: f.operations }), f.context);
  await assert.rejects(loadResetOriginContext(f.root, { operations: f.operations }), { code: "private_path_exists" });
});

test("unstarted successor rejects a default-anchor mismatch and coherently rewritten old inventory", async (t) => {
  // Arrange
  const f = await unusedObservation(t);
  // Act / Assert
  const operations = { ...f.operations };
  delete operations.expectedResetOriginUnstartedSeal;
  await assert.rejects(resetOriginPreflight(f.successor, operations), { code: "reset_origin_unstarted_anchor" });
  await writeNew(resolve(f.root, "diagnostic-export-0000.json"), {});
  const path = resolve(f.root, "failed-inventory.json"),
    seal = await readJson(path);
  seal.inventory = await inventory(f.root);
  await writeFile(path, JSON.stringify(seal));
  await assert.rejects(resetOriginPreflight(f.successor, f.operations), { code: "reset_origin_unstarted_anchor" });
});

test("unstarted classifier independently rejects activity, dirty cleanup and changed cause even with a test fixture anchor", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["record", "start", "accounting", "issued", "cleanup", "cause"]) {
    const f = await unusedObservation(t);
    if (mode === "cleanup" || mode === "cause") {
      const name = mode === "cleanup" ? "unused-host-cleanup.json" : "page-serving-failure.json";
      const value = await readJson(resolve(f.root, name));
      if (mode === "cleanup") value.serial_holders_absent = false;
      else value.first_failure = "other";
      await writeFile(resolve(f.root, name), JSON.stringify(value));
    } else {
      const names = {
        record: "no-mining-state-0001.json",
        start: "reset-origin-start.json",
        accounting: "no-mining-accounting-before.json",
        issued: "issued.json",
      };
      await writeNew(resolve(f.root, names[mode]), {});
    }
    const path = resolve(f.root, "failed-inventory.json"),
      seal = await readJson(path);
    seal.inventory = await inventory(f.root);
    seal.cleanup_sha256 = await fileDigest(resolve(f.root, "unused-host-cleanup.json"));
    seal.page_failure_sha256 = await fileDigest(resolve(f.root, "page-serving-failure.json"));
    await writeFile(path, JSON.stringify(seal));
    f.operations.expectedResetOriginUnstartedSeal = await fileDigest(path);
    await assert.rejects(resetOriginPreflight(f.successor, f.operations), {
      code:
        mode === "cleanup"
          ? "reset_origin_unstarted_cleanup"
          : mode === "cause"
            ? "reset_origin_unstarted_failure"
            : "reset_origin_unstarted_activity",
    });
  }
});

test("unstarted successor requires changed published driver and seal-linked software correction", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["driver", "plan", "link"]) {
    const f = await unusedObservation(t);
    if (mode === "driver") f.successor.qualificationSourceCommit = f.context.qualification_driver.source_commit;
    else {
      const plan = await readJson(f.successor.input);
      if (mode === "plan") plan.reason = "prospective_observation";
      else plan.evidence_sha256 = ["0".repeat(64)];
      await writeFile(f.successor.input, JSON.stringify(plan));
    }
    await assert.rejects(resetOriginPreflight(f.successor, f.operations), {
      code: mode === "driver" ? "reset_origin_unstarted_lineage" : "reset_origin_unstarted_correction",
    });
  }
});

test("successor reservation survives interruption before or after child creation and blocks alternate-root reassignment", async (t) => {
  // Arrange / Act / Assert
  for (const createChild of [false, true]) {
    const f = await unusedObservation(t);
    await assert.rejects(
      resetOriginPreflight(f.successor, {
        ...f.operations,
        mkdir: async (root, options) => {
          if (createChild) await mkdir(root, options);
          throw Error("fixture interrupted");
        },
      }),
      /fixture interrupted/u,
    );
    await assert.rejects(resetOriginPreflight({ ...f.successor, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), {
      code: "EEXIST",
    });
    await assert.rejects(loadResetOriginContext(f.successor.privateRoot, { operations: f.operations }), { code: "ENOENT" });
  }
});

test("successor admission and later live validation reject changed predecessor bytes", async (t) => {
  // Arrange
  const f = await unusedObservation(t);
  await resetOriginPreflight(f.successor, f.operations);
  await writeFile(resolve(f.root, "page-serving-failure.json"), "{}\n");
  // Act / Assert
  await assert.rejects(loadResetOriginContext(f.successor.privateRoot, { operations: f.operations }), {
    code: "reset_origin_unstarted_changed",
  });
  await assert.rejects(resetOriginPreflight({ ...f.successor, privateRoot: resolve(dirname(f.root), "alternate") }, f.operations), {
    code: "reset_origin_unstarted_changed",
  });
});

test("exact pre-capture review admits only a new observation with both sealed ancestors unchanged", async (t) => {
  // Arrange
  const f = await preparationReview(t),
    before = await inventory(f.reviewRoot),
    old = await inventory(f.root);
  // Act
  await resetOriginPreflight(f.third, f.operations);
  const context = await loadResetOriginContext(f.third.privateRoot, { operations: f.operations });
  // Assert
  assert.equal(context.observation_attempt, 3);
  assert.notEqual(context.observation_id, f.reviewContext.observation_id);
  assert.equal(context.unstarted_predecessor, undefined);
  assert.deepEqual(context.runtime_source, f.context.runtime_source);
  assert.equal(context.expected_next_ordinal, 17);
  assert.equal(context.expected_charged_ms, 1380000);
  assert.equal(context.mining_authorized, false);
  assert.deepEqual(await inventory(f.reviewRoot), before);
  assert.deepEqual(await inventory(f.root), old);
  await assert.rejects(readFile(resolve(f.reviewRoot, "no-mining-accounting-after.json")), { code: "ENOENT" });
  await assert.rejects(loadResetOriginContext(f.reviewRoot, { operations: f.operations }), { code: "private_path_exists" });
  assert.deepEqual(await loadResetOriginContext(f.reviewRoot, { historical: true, operations: f.operations }), f.reviewContext);
});

test("preparation-review guard rejects default anchor, coherent reseal and cross-class predecessors", async (t) => {
  // Arrange
  const f = await preparationReview(t),
    operations = { ...f.operations };
  delete operations.expectedResetOriginPreparationReviewSeal;
  // Act / Assert
  await assert.rejects(resetOriginPreflight(f.third, operations), { code: "reset_origin_preparation_review_anchor" });
  await assert.rejects(resetOriginPreflight({ ...f.third, supersedePreparationReview: f.root }, f.operations), {
    code: "reset_origin_preparation_review_anchor",
  });
  await writeNew(resolve(f.reviewRoot, "diagnostic-export-0000.json"), {});
  const path = resolve(f.reviewRoot, "failed-inventory.json"),
    seal = await readJson(path);
  seal.inventory = await inventory(f.reviewRoot);
  await writeFile(path, JSON.stringify(seal));
  await assert.rejects(resetOriginPreflight(f.third, f.operations), { code: "reset_origin_preparation_review_anchor" });
});

test("preparation-review facts independently reject captured evidence, wrong receipt origin, dirty cleanup and changed ledger", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["capture", "receipt", "cleanup", "ledger"]) {
    const f = await preparationReview(t),
      root = f.reviewRoot;
    const names = { receipt: "parent-failure-review.json", cleanup: "host-cleanup.json", ledger: "no-mining-accounting-before.json" };
    if (mode === "capture") await writeNew(resolve(root, "reset-origin-start.json"), {});
    else {
      const value = await readJson(resolve(root, names[mode]));
      if (mode === "receipt") value.preparation[0].origin = "current_boot";
      if (mode === "cleanup") value.listener_absent = false;
      if (mode === "ledger") value.ledger.pending = true;
      await writeFile(resolve(root, names[mode]), JSON.stringify(value));
    }
    const path = resolve(root, "failed-inventory.json"),
      seal = await readJson(path);
    seal.inventory = await inventory(root);
    for (const [field, name] of [
      ["parent_failure_sha256", names.receipt],
      ["cleanup_sha256", names.cleanup],
      ["before_accounting_sha256", names.ledger],
    ])
      seal[field] = await fileDigest(resolve(root, name));
    await writeFile(path, JSON.stringify(seal));
    f.operations.expectedResetOriginPreparationReviewSeal = await fileDigest(path);
    await assert.rejects(resetOriginPreflight(f.third, f.operations), {
      code: {
        capture: "reset_origin_preparation_review_activity",
        receipt: "reset_origin_preparation_review_failure",
        cleanup: "reset_origin_preparation_review_cleanup",
        ledger: "iterative_ledger_admission",
      }[mode],
    });
  }
});

test("third observation reservation is exclusive through partial creation and conflicting flags fail before path inspection", async (t) => {
  // Arrange
  const f = await preparationReview(t);
  // Act / Assert
  await assert.rejects(
    resetOriginPreflight({ privateRoot: "/missing", supersedeUnstarted: "/missing", supersedePreparationReview: "/missing" }),
    {
      code: "reset_origin_supersession_conflict",
    },
  );
  await assert.rejects(
    resetOriginPreflight(f.third, {
      ...f.operations,
      mkdir: async (root, options) => {
        await mkdir(root, options);
        throw Error("fixture partial third");
      },
    }),
    /fixture partial third/u,
  );
  await assert.rejects(resetOriginPreflight({ ...f.third, privateRoot: resolve(dirname(f.root), "alternate-third") }, f.operations), {
    code: "EEXIST",
  });
  await assert.rejects(loadResetOriginContext(f.third.privateRoot, { operations: f.operations }), { code: "ENOENT" });
});

test("shared runtime is validated once per invocation and changed source is rejected on the next invocation", async (t) => {
  // Arrange
  const f = await preparationReview(t);
  await resetOriginPreflight(f.third, f.operations);
  const expectedSeal = f.operations.expectedResetOriginSourceSeal;
  let validations = 0;
  Object.defineProperty(f.operations, "expectedResetOriginSourceSeal", {
    get() {
      validations++;
      return expectedSeal;
    },
  });
  // Act
  await loadResetOriginContext(f.third.privateRoot, { operations: f.operations });
  assert.equal(validations, 1);
  await loadResetOriginContext(f.third.privateRoot, { operations: f.operations });
  assert.equal(validations, 2);
  const page = resolve(f.sourceRoot, "qualified-artifacts/gate", f.context.gate_page_relative_path);
  await writeFile(page, (await readFile(page, "utf8")) + "\nfixture changed source\n");
  // Assert
  await assert.rejects(loadResetOriginContext(f.third.privateRoot, { operations: f.operations }));
  assert.equal(validations, 3);
});

test("per-invocation shared runtime verification still checks every ancestor inventory", async (t) => {
  // Arrange
  const f = await preparationReview(t);
  await resetOriginPreflight(f.third, f.operations);
  await writeFile(resolve(f.root, "page-serving-failure.json"), "{}\n");
  // Act / Assert
  await assert.rejects(loadResetOriginContext(f.third.privateRoot, { operations: f.operations }), {
    code: "reset_origin_unstarted_changed",
  });
});

test("exact journal failure admits a separate host-client binding while preserving all incomplete evidence", async (t) => {
  // Arrange
  const f = await journalFailure(t),
    old = await inventory(f.journalRoot);
  // Act
  await resetOriginPreflight(f.fourth, f.operations);
  const context = await loadResetOriginContext(f.fourth.privateRoot, { operations: f.operations });
  // Assert
  assert.equal(context.observation_attempt, 4);
  assert.notEqual(context.observation_id, f.journalContext.observation_id);
  assert.equal(context.supervisor_client_sha256, f.context.supervisor_client_sha256);
  assert.equal(context.no_mining_context.supervisor_client_sha256, context.qualification_driver.no_mining_client_sha256);
  assert.equal(context.qualification_driver.no_mining_client_sha256, await fileDigest(new URL("./no-mining-client.mjs", import.meta.url)));
  assert.equal(context.mining_authorized, false);
  assert.equal(context.expected_next_ordinal, 17);
  assert.equal(context.expected_charged_ms, 1380000);
  assert.deepEqual(await inventory(f.journalRoot), old);
  for (const file of ["result.json", "reset-origin-end.json", "no-mining-accounting-after.json"])
    await assert.rejects(readFile(resolve(f.journalRoot, file)), { code: "ENOENT" });
  assert.equal((await readJson(resolve(f.journalRoot, "no-mining-state-0083.json"))).state.status, "ready");
  assert.deepEqual(await loadResetOriginContext(f.journalRoot, { historical: true, operations: f.operations }), f.journalContext);
});

test("journal successor requires its exact seal and the pinned independently verified regression", async (t) => {
  // Arrange
  const f = await journalFailure(t),
    operations = { ...f.operations };
  delete operations.expectedResetOriginJournalFailureSeal;
  // Act / Assert
  await assert.rejects(resetOriginPreflight(f.fourth, operations), { code: "reset_origin_journal_failure_anchor" });
  const plan = await readJson(f.fourth.input);
  plan.evidence_sha256 = [f.operations.expectedResetOriginJournalFailureSeal, "0".repeat(64)];
  await writeFile(f.fourth.input, JSON.stringify(plan));
  await assert.rejects(resetOriginPreflight(f.fourth, f.operations), { code: "reset_origin_unstarted_correction" });
});

test("journal successor rejects fabricated final evidence, cleanup claims and captured batch changes", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["end", "closure", "batch"]) {
    const f = await journalFailure(t),
      root = f.journalRoot;
    if (mode === "end") await writeNew(resolve(root, "reset-origin-end.json"), {});
    if (mode === "closure") {
      const path = resolve(root, "parent-journal-failure.json"),
        value = await readJson(path);
      value.worker.serialOwnershipReleased = false;
      await writeFile(path, JSON.stringify(value));
    }
    if (mode === "batch") {
      const path = resolve(root, "diagnostic-export-0445.json"),
        value = await readJson(path);
      value.sequence = 9;
      await writeFile(path, JSON.stringify(value));
    }
    const path = resolve(root, "failed-inventory.json"),
      seal = await readJson(path);
    seal.inventory = await inventory(root);
    seal.parent_failure_sha256 = await fileDigest(resolve(root, "parent-journal-failure.json"));
    await writeFile(path, JSON.stringify(seal));
    await assert.rejects(resetOriginPreflight(f.fourth, f.operations), { code: "reset_origin_journal_failure_anchor" });
    f.operations.expectedResetOriginJournalFailureSeal = await fileDigest(path);
    await assert.rejects(resetOriginPreflight(f.fourth, f.operations), {
      code: {
        end: "reset_origin_journal_failure_activity",
        closure: "reset_origin_journal_failure_provenance",
        batch: "reset_origin_journal_failure_batch",
      }[mode],
    });
  }
});

test("host-client override is unavailable to old contexts and required for the fourth observation", async (t) => {
  // Arrange
  const f = await journalFailure(t);
  await resetOriginPreflight(f.fourth, f.operations);
  // Act / Assert
  for (const root of [f.root, f.fourth.privateRoot]) {
    const path = resolve(root, "context.json"),
      saved = await readJson(path);
    if (root === f.root) saved.context.qualification_driver.no_mining_client_sha256 = "a".repeat(64);
    else delete saved.context.qualification_driver.no_mining_client_sha256;
    saved.sha256 = digest(JSON.stringify(saved.context));
    await writeFile(path, JSON.stringify(saved));
    await assert.rejects(loadResetOriginContext(root, { historical: true, operations: f.operations }), {
      code: "reset_origin_host_client_binding",
    });
  }
});

test("fourth observation keeps exclusive reservation and rejects every conflicting predecessor flag", async (t) => {
  // Arrange
  const f = await journalFailure(t);
  // Act / Assert
  for (const field of ["supersedeUnstarted", "supersedePreparationReview"])
    await assert.rejects(resetOriginPreflight({ privateRoot: "/missing", supersedeJournalFailure: "/missing", [field]: "/missing" }), {
      code: "reset_origin_supersession_conflict",
    });
  await assert.rejects(
    resetOriginPreflight(f.fourth, {
      ...f.operations,
      mkdir: async (root, options) => {
        await mkdir(root, options);
        throw Error("fixture partial fourth");
      },
    }),
    /fixture partial fourth/u,
  );
  await assert.rejects(resetOriginPreflight({ ...f.fourth, privateRoot: resolve(dirname(f.root), "alternate-fourth") }, f.operations), {
    code: "EEXIST",
  });
  await assert.rejects(loadResetOriginContext(f.fourth.privateRoot, { operations: f.operations }), { code: "ENOENT" });
});

test("observation attempts require numeric values and the corrected client must match its pinned regression", async (t) => {
  // Arrange
  const f = await journalFailure(t);
  await resetOriginPreflight(f.fourth, f.operations);
  const path = resolve(f.fourth.privateRoot, "context.json"),
    original = await readJson(path);
  // Act / Assert
  for (const attempt of ["4", "toString", 5]) {
    const saved = structuredClone(original);
    saved.context.observation_attempt = attempt;
    delete saved.context.qualification_driver.no_mining_client_sha256;
    saved.sha256 = digest(JSON.stringify(saved.context));
    await writeFile(path, JSON.stringify(saved));
    await assert.rejects(loadResetOriginContext(f.fourth.privateRoot, { historical: true, operations: f.operations }), {
      code: "reset_origin_observation_attempt",
    });
  }
  const saved = structuredClone(original);
  saved.context.qualification_driver.no_mining_client_sha256 = "a".repeat(64);
  saved.sha256 = digest(JSON.stringify(saved.context));
  await writeFile(path, JSON.stringify(saved));
  await assert.rejects(loadResetOriginContext(f.fourth.privateRoot, { historical: true, operations: f.operations }), {
    code: "reset_origin_host_client_binding",
  });
});
