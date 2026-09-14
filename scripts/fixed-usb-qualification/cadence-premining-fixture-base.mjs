import assert from "node:assert/strict";
import { chmod, mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { CADENCE_TASK } from "./cadence-contract.mjs";
import { cadencePreflight } from "./cadence-preflight.mjs";
import { digest, fileDigest, writeNew } from "./contract.mjs";
import { closeUnissued } from "./cadence-unissued.mjs";

async function fixture(t) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "cadence-preflight-")));
  await chmod(base, 0o700);
  t.after(() => rm(base, { recursive: true, force: true }));
  for (const path of [
    "firmware/scripts/fixed-usb-qualification",
    "firmware/bazel-bin/tools/http-transport",
    "gate",
    "authority",
    "attempts/prior",
  ])
    await mkdir(resolve(base, path), { recursive: true, mode: 0o700 });
  const firmwareRoot = resolve(base, "firmware");
  await writeFile(resolve(firmwareRoot, "TASKS.md"), `## Active\n### ${CADENCE_TASK} | fixture\n`);
  await writeFile(resolve(firmwareRoot, "scripts/fixed-usb-qualification/fixture.mjs"), "export const bounded = true;\n");
  const previousReceipt = resolve(base, "attempts/prior/result.json");
  await writeNew(previousReceipt, { fixture: true });
  const input = resolve(base, "progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: ["d".repeat(64)],
  });
  const observerBinary = resolve(firmwareRoot, "bazel-bin/tools/http-transport/cadence_observer");
  await writeFile(observerBinary, "binary fixture", { mode: 0o700 });
  const options = {
    suggestedDifficulty: "1000",
    observerBinary,
    firmwareRoot,
    gateRoot: resolve(base, "gate"),
    authorityDirectory: resolve(base, "authority"),
    privateRoot: resolve(base, "attempts/current"),
    previousReceipt,
    input,
    manifest: resolve(base, "manifest.json"),
  };
  const previous = {
    context: { firmware_commit: "a".repeat(40), gate_commit: "b".repeat(40) },
    cleanup_confirmed: true,
    original_budget: {
      schema: "worker-budget-review-v1",
      campaign_match: true,
      reserved_mask: 7,
      completed_mask: 7,
      charged_ms: 240000,
      pending: false,
    },
    next_ordinal: 16,
    total_charged_ms: 1200000,
    original_campaign_id: Buffer.alloc(16, 1).toString("base64url"),
  };
  let inspections = 0,
    snapshotChecks = 0;
  const operations = {
    ignored: () => undefined,
    readPrevious: async () => previous,
    verifyArtifactSnapshot: async (root, context) => {
      assert.equal(root, resolve(base, "attempts/prior"));
      assert.equal(context, previous.context);
      snapshotChecks++;
    },
    inspectSources: async () => {
      assert(snapshotChecks > 0);
      inspections += 1;
      return { firmware_commit: "c".repeat(40), gate_commit: "b".repeat(40) };
    },
  };
  return { base, options, previous, operations, inspections: () => inspections };
}
export async function unissuedFixture(t) {
  const f = await fixture(t);
  await cadencePreflight(f.options, f.operations);
  const oldRoot = f.options.privateRoot,
    { context } = JSON.parse(await readFile(resolve(oldRoot, "context.json"), "utf8"));
  f.operations.verifyArtifactSnapshot = async () => undefined;
  await writeNew(resolve(oldRoot, "artifact-snapshot.json"), { fixture: "retained artifacts" });
  const final = {
    schema: "worker-serial-acceptance-v1",
    gateCommit: context.gate_commit,
    expectedFirmwareSourceCommit: context.firmware_commit,
    status: "closed",
    connected: false,
    running: false,
    heartbeatSuppressed: false,
    renewalsConfirmed: 0,
    deviceRestorationConfirmed: false,
    deviceBaselineConfirmed: true,
    deviceLeaseInactive: true,
    serialOwnershipReleased: true,
    preservation: {
      schema: "worker-preservation-continuity-v1",
      baseline_id: Buffer.alloc(16, 2).toString("base64url"),
      device_identity_match: true,
      settings_match: true,
      authorization_high_water_match: true,
      mine_on_boot: false,
    },
  };
  const records = [
    {
      sequence: 1,
      state: {
        ...final,
        status: "failed",
        failure: "connect_failed",
        admissionFailureStage: "permission",
        serialFailureCategory: "operation_failed",
      },
    },
    { sequence: 2, state: final },
  ];
  await writeFile(resolve(oldRoot, "iterative.samples.jsonl"), records.map((record) => JSON.stringify(record) + "\n").join(""), {
    mode: 0o600,
  });
  await writeNew(resolve(oldRoot, "first-failure.json"), {
    schema: "worker-iterative-first-failure-v1",
    ordinal: 16,
    sequence: 1,
    browser: "connect_failed",
    serial: "operation_failed",
    admission: "permission",
  });
  await writeNew(resolve(oldRoot, "native-gesture-remediation.json"), {
    schema: "cadence-preparation-remediation-v1",
    original_boundary: { failure: "connect_failed", stage: "permission", serial: "operation_failed" },
    remediation: "foreground_native_accessibility_click",
    native_chooser_observed: true,
    exact_firmware_admitted: true,
    baseline_confirmed: true,
    ledger_read_method: "workerAcceptance.reviewQualificationAttempts",
    original_budget_read_method: "workerAcceptance.reviewBudget",
    private_values_exported: false,
    qualification_claimed: false,
  });
  const input = {
    ledger: {
      schema: "worker-qualification-ledger-v1",
      next_ordinal: 16,
      total_charged_ms: 1200000,
      pending: false,
      last_completed_ordinal: 15,
    },
    original_budget: f.previous.original_budget,
    cleanup: {
      schema: "worker-unissued-cleanup-v1",
      source: "parent-observed",
      browser_closed: true,
      supervisor_exited: true,
      supervisor_exit_code: 0,
      listener_absent: true,
      owned_children_absent: true,
      serial_holders_absent: true,
    },
  };
  await closeUnissued(oldRoot, input, f.operations);
  const successorInput = resolve(f.base, "successor-progress.json");
  await writeNew(successorInput, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "manual_remediation",
    evidence_sha256: [await fileDigest(resolve(oldRoot, "native-gesture-remediation.json"))],
  });
  const options = {
    ...f.options,
    input: successorInput,
    privateRoot: resolve(f.base, "attempts/successor"),
    supersedeUnissued: resolve(oldRoot, "unissued-closure.json"),
  };
  f.operations.inspectSources = async () => ({ firmware_commit: "f".repeat(40), gate_commit: context.gate_commit });
  return { ...f, options, oldRoot, oldContext: context };
}
