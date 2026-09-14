import assert from "node:assert/strict";
import { copyFile, readFile, readdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import test from "node:test";
import { usbPreminingFixture } from "./cadence-premining-usb-fixtures.mjs";
import { ACCEPTED_USB_PREMINING_INVENTORY, reviewUsbPreminingEvidence } from "./cadence-premining-usb.mjs";
import { closePremining, preminingClosurePath, readPremining, reviewPremining } from "./cadence-premining.mjs";
import { cadencePreflight, validateCadenceContext } from "./cadence-preflight.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { main } from "./main.mjs";

async function nextOptions(f) {
  await closePremining(f.root, f.operations);
  const input = resolve(f.base, "usb-successor-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: ["a".repeat(64)],
  });
  f.operations.inspectSources = async () => ({ ...f.snapshot, firmware_commit: "7".repeat(40) });
  return { ...f.options, privateRoot: resolve(f.base, "attempts/preparation-4"), input, supersedePremining: preminingClosurePath(f.root) };
}
async function journal(f, change) {
  change(f.records);
  await writeFile(resolve(f.root, "iterative.samples.jsonl"), f.records.map((row) => JSON.stringify(row) + "\n").join(""));
}

test("v2 closure binds separate failed USB reviews and preserves v1 ancestry and every original byte", async (t) => {
  // Arrange
  const f = await usbPreminingFixture(t),
    before = await inventory(f.root),
    ancestorHash = await fileDigest(preminingClosurePath(f.priorRoot));
  // Act
  const result = await closePremining(f.root, f.operations),
    receipt = await readPremining(preminingClosurePath(f.root), f.operations);
  // Assert
  assert.equal(result.classification, "premining_usb_failure");
  assert.equal(result.result, "unverified");
  assert.equal(result.continuation_authority, false);
  assert.equal(receipt.schema, "worker-cadence-premining-closure-v2");
  assert.equal(receipt.first_rejected_review.sequence, 11);
  assert.equal(receipt.later_retained_review.sequence, 15);
  assert.deepEqual(await inventory(f.root), before);
  assert.equal(await fileDigest(preminingClosurePath(f.priorRoot)), ancestorHash);
  assert.equal((await readPremining(preminingClosurePath(f.priorRoot), f.operations)).schema, "worker-cadence-premining-closure-v1");
  assert.deepEqual(await reviewPremining(f.root, f.operations), result);
  await assert.rejects(closePremining(f.root, f.operations), { code: "private_path_exists" });
});

test("production v2 anchor rejects unknown internally valid and coherently resealed failures", async (t) => {
  // Arrange
  const f = await usbPreminingFixture(t),
    operations = { ...f.operations };
  delete operations.expectedUsbPreminingInventorySha256;
  assert.notEqual(await fileDigest(resolve(f.root, "failed-inventory.json")), ACCEPTED_USB_PREMINING_INVENTORY);
  // Act / Assert
  await assert.rejects(reviewPremining(f.root, operations), { code: "cadence_usb_premining_accepted_seal_required" });
  await writeNew(resolve(f.root, "extra.json"), { fixture: true });
  await writeFile(resolve(f.root, "failed-inventory.json"), JSON.stringify({ ...f.seal, inventory: await inventory(f.root) }));
  await assert.rejects(closePremining(f.root, f.operations), { code: "cadence_usb_premining_accepted_seal_required" });
});

test("v1 cannot enter the v2 matcher and sibling schema swaps cannot cross failure classes", async (t) => {
  // Arrange
  const f = await usbPreminingFixture(t);
  // Act / Assert
  await assert.rejects(reviewUsbPreminingEvidence(f.priorRoot, f.operations), { code: "cadence_usb_premining_accepted_seal_required" });
  await closePremining(f.root, f.operations);
  const path = preminingClosurePath(f.root),
    saved = await readJson(path);
  saved.receipt.schema = "worker-cadence-premining-closure-v1";
  saved.sha256 = digest(JSON.stringify(saved.receipt));
  await writeFile(path, JSON.stringify(saved));
  await assert.rejects(reviewPremining(f.root, f.operations), { code: "cadence_premining_closure_changed" });
});

test("issuance, load, work, renewals, faults and mining arm remain ineligible", async (t) => {
  // Arrange / Act / Assert
  for (const name of ["issued.json", "consumed.json", "cadence-mining-arm.json", "iterative.fault.json", "result.json"]) {
    const f = await usbPreminingFixture(t);
    await writeNew(resolve(f.root, name), {});
    await assert.rejects(closePremining(f.root, f.operations), { code: "private_path_exists" });
  }
  for (const change of [{ status: "window_loaded" }, { running: true }, { renewalsConfirmed: 1 }]) {
    const f = await usbPreminingFixture(t);
    await journal(f, (rows) => Object.assign(rows[11].state, change));
    await assert.rejects(reviewPremining(f.root, f.operations), { code: "unexpected_funding_or_work" });
  }
  const f = await usbPreminingFixture(t);
  await journal(f, (rows) => {
    rows[11].state.cadence.latestWork = { atMs: 1, generation: 1, workDispatched: 1 };
  });
  await assert.rejects(reviewPremining(f.root, f.operations), { code: "unexpected_funding_or_work" });
});

test("bad probe timing, missing probes and false witnesses cannot qualify the failed preparation", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["count", "timing", "witness"]) {
    const f = await usbPreminingFixture(t),
      name = mode === "witness" ? "cadence-probe-witnesses.jsonl" : "cadence-probes.jsonl",
      path = resolve(f.root, name);
    const rows = (await readFile(path, "utf8")).trim().split("\n").map(JSON.parse);
    if (mode === "count") rows.pop();
    if (mode === "timing") {
      rows[0].startedAtMs = 58000;
      rows[0].completedAtMs = 58010;
    }
    if (mode === "witness") rows[0].sequence = 1;
    await writeFile(path, rows.map((row) => JSON.stringify(row) + "\n").join(""));
    await assert.rejects(reviewPremining(f.root, f.operations));
  }
});

test("first rejected review and later retained review cannot hide changed numbers or repair the original failure", async (t) => {
  // Arrange / Act / Assert
  for (const sequence of [11, 15]) {
    const f = await usbPreminingFixture(t);
    await journal(f, (rows) => {
      rows[sequence - 1].state.cadence.review.phases[1].pendingSends = 0;
    });
    await assert.rejects(reviewPremining(f.root, f.operations));
  }
  const f = await usbPreminingFixture(t);
  await journal(f, (rows) => {
    rows[14].state.cadence.review.phases[0].maximumIntervalUs++;
  });
  await assert.rejects(reviewPremining(f.root, f.operations), { code: "later_review_changed" });
});

test("v2 ancestry, archived task and partial sibling reject without altering sealed originals", async (t) => {
  // Arrange
  const f = await usbPreminingFixture(t);
  await closePremining(f.root, f.operations);
  // Act / Assert
  await assert.rejects(readPremining(preminingClosurePath(f.root), { ...f.operations, preminingAncestors: [f.root] }), {
    code: "cadence_premining_recursive_lineage",
  });
  const old = await readJson(preminingClosurePath(f.priorRoot));
  old.receipt.ledger.pending = true;
  old.sha256 = digest(JSON.stringify(old.receipt));
  await writeFile(preminingClosurePath(f.priorRoot), JSON.stringify(old));
  await assert.rejects(readPremining(preminingClosurePath(f.root), f.operations));
  const g = await usbPreminingFixture(t);
  await writeFile(resolve(g.options.firmwareRoot, "TASKS.md"), "## Active\n");
  await assert.rejects(closePremining(g.root, g.operations), { code: "cadence_active_task_required" });
  const h = await usbPreminingFixture(t);
  await writeFile(preminingClosurePath(h.root), "{", { mode: 0o600 });
  await assert.rejects(reviewPremining(h.root, h.operations));
  await assert.rejects(closePremining(h.root, h.operations), { code: "private_path_exists" });
});

test("v2 supersession reserves preparation4 before effects and retains unchanged unspent device accounting", async (t) => {
  // Arrange
  const f = await usbPreminingFixture(t),
    options = await nextOptions(f),
    oldHash = await fileDigest(resolve(f.root, "failed-inventory.json"));
  // Act
  await cadencePreflight(options, f.operations);
  const { context } = await readJson(resolve(options.privateRoot, "context.json"));
  // Assert
  assert.equal(context.preparation_attempt, 4);
  assert.equal(context.qualification_attempt.ordinal, 16);
  assert.equal(context.expected_charged_ms, 1200000);
  assert.notEqual(context.qualification_attempt.id, f.context.qualification_attempt.id);
  assert.equal(context.premining_predecessor.root, f.root);
  await validateCadenceContext(options.privateRoot, context, { operations: f.operations });
  assert.equal(await fileDigest(resolve(f.root, "failed-inventory.json")), oldHash);
  await assert.rejects(cadencePreflight({ ...options, privateRoot: resolve(f.base, "attempts/duplicate4") }, f.operations), {
    code: "private_path_exists",
  });
});

test("v2 interrupted child creation retains exclusive assignment and rejects alternate-root retry", async (t) => {
  // Arrange / Act / Assert
  for (const stage of ["mkdir", "copyFile"]) {
    const f = await usbPreminingFixture(t),
      options = await nextOptions(f),
      operations = { ...f.operations };
    operations[stage] = async (...args) => {
      if (stage === "copyFile") await copyFile(...args);
      throw new Error("interrupted fixture");
    };
    await assert.rejects(cadencePreflight(options, operations), /interrupted fixture/u);
    assert.equal((await readJson(resolve(f.base, "attempts/ordinal-16-preparation-4.json"))).attempt_root, options.privateRoot);
    await assert.rejects(cadencePreflight({ ...options, privateRoot: resolve(f.base, "attempts/alternate4") }, f.operations), {
      code: "private_path_exists",
    });
    await assert.rejects(readFile(resolve(options.privateRoot, "context.json")), { code: "ENOENT" });
  }
});

test("v2 continuation still requires software correction and a changed published pair", async (t) => {
  // Arrange / Act / Assert
  for (const mode of ["manual", "unchanged"]) {
    const f = await usbPreminingFixture(t),
      options = await nextOptions(f);
    if (mode === "manual") {
      const p = await readJson(options.input);
      p.reason = "manual_remediation";
      await writeFile(options.input, JSON.stringify(p));
    } else f.operations.inspectSources = async () => f.snapshot;
    await assert.rejects(cadencePreflight(options, f.operations));
    assert(!(await readdir(resolve(f.base, "attempts"))).includes("ordinal-16-preparation-4.json"));
  }
});

test("same read-only CLI dispatch validates v2 relative sources from a foreign working directory", async (t) => {
  // Arrange
  const f = await usbPreminingFixture(t),
    cwd = process.cwd();
  // Act / Assert
  try {
    process.chdir(f.base);
    const result = await main(["cadence-review-premining", "--private-root", f.root], f.operations);
    assert.equal(result.classification, "premining_usb_failure");
  } finally {
    process.chdir(cwd);
  }
});
