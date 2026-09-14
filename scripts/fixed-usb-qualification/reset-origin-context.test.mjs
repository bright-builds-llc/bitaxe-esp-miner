import assert from "node:assert/strict";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { inventory } from "./cadence-premining-evidence.mjs";
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

async function unusedObservation(t) {
  const f = await resetOriginFixture(t),
    hash = digest(JSON.stringify(f.context));
  await writeNew(resolve(f.root, "page-serving-failure.json"), {
    schema: "reset-origin-page-serving-failure-v1",
    source: "parent-observed",
    http_status: 200,
    content_type: "text/html",
    body_sha256: "a".repeat(64),
    json_encoded_html: true,
    browser_control_opened: false,
    browser_closed: true,
    first_failure: "html_response_json_encoded",
  });
  await writeNew(resolve(f.root, "unused-host-cleanup.json"), {
    schema: "reset-origin-unused-host-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    browser_control_opened: false,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
    observation_started: false,
  });
  await writeNew(resolve(f.root, "reset-origin-server-claim.json"), {
    schema: "fixed-usb-reset-origin-server-claim-v1",
    context_sha256: hash,
  });
  await writeNew(resolve(f.root, "reset-origin-failure.json"), {
    schema: "fixed-usb-reset-origin-failure-v1",
    context_sha256: hash,
    code: "reset_origin_server_closed_before_end",
  });
  const seal = {
    schema: "fixed-usb-reset-origin-unstarted-failed-inventory-v1",
    outcome: "unverified",
    first_failure: "html_response_json_encoded",
    secondary_failure: "reset_origin_server_closed_before_end",
    observation_started: false,
    qualification_pass: false,
    device_recovery_claimed: false,
    continuation_authorized: false,
    context_sha256: hash,
    artifact_snapshot_sha256: await fileDigest(resolve(f.root, "artifact-snapshot.json")),
    page_failure_sha256: await fileDigest(resolve(f.root, "page-serving-failure.json")),
    cleanup_sha256: await fileDigest(resolve(f.root, "unused-host-cleanup.json")),
    secondary_failure_sha256: await fileDigest(resolve(f.root, "reset-origin-failure.json")),
    inventory: await inventory(f.root),
  };
  await writeNew(resolve(f.root, "failed-inventory.json"), seal);
  f.operations.expectedResetOriginUnstartedSeal = await fileDigest(resolve(f.root, "failed-inventory.json"));
  const plan = resolve(dirname(f.root), "observation-correction.json");
  await writeNew(plan, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [f.operations.expectedResetOriginUnstartedSeal],
  });
  f.successor = {
    ...f.options,
    privateRoot: resolve(dirname(f.root), "observation-2"),
    input: plan,
    qualificationSourceCommit: "f".repeat(40),
    supersedeUnstarted: f.root,
  };
  return f;
}

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
