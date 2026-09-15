import { networkRestartFixture } from "./reset-origin-restart-network-fixtures.mjs";
import { copyFile, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { knownFailureDiagnostics, statisticsLine } from "./reset-origin-restart-successor-fixtures.mjs";
import { recordRestartFixture, installRestartFixture } from "./reset-origin-restart-fixtures.mjs";
import { saveRestartAccounting } from "./reset-origin-restart-state.mjs";
import { savePreinstallFailure } from "./reset-origin-restart-preinstall.mjs";
import { restartPreflight, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { recoveryState } from "./cadence-startup-fixtures.mjs";
import { artifactFixture } from "./cadence-premining-fixtures.mjs";
import { digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { saveRestartFailure } from "./reset-origin-restart-failure.mjs";
import {
  inspectStorageFailureCapture,
  inspectRestartStorageFailure,
  RESTART_STORAGE_FAILURE_SHA256,
  RESTART_STORAGE_FAILURE_PRODUCER,
} from "./reset-origin-restart-storage-failure.mjs";
export async function failedStorageRestartFixture(t) {
  const f = await networkRestartFixture(t),
    inner = restartInnerContext(f.root, f.context, "before-install"),
    scope = resolve(f.root, "before-install"),
    state = recoveryState(inner);
  await recordRestartFixture(scope, inner, state);
  await saveRestartAccounting(scope, inner, { stage: "before", ledger: f.ledger, original_budget: f.original, state });
  await savePreinstallFailure(f.root, f.context, knownFailureDiagnostics(f.context));
  await recordRestartFixture(scope, inner, recoveryState(inner, true));
  const installed = await installRestartFixture(f, { review: false, additionalCapture: statisticsLine() });
  const capturePath = resolve(f.root, "install-001/flash-monitor.log");
  const bytes = Buffer.from(
    (await readFile(capturePath, "utf8")).replaceAll("state=complete first_failure=none", "state=failed first_failure=storage_http") +
      "storage_http_failure schema=v1 phase=http_server error=http_task redacted=true\n",
  );
  await writeFile(capturePath, bytes);
  const input = await readJson(installed.reviewInput);
  input.command_exit_code = 1;
  input.capture_sha256 = await fileDigest(capturePath);
  await writeNew(resolve(f.root, "install-review-input.json"), input);
  const flashPath = resolve(f.root, "install-001/flash-command-evidence.json"),
    flash = await readJson(flashPath);
  Object.assign(flash, {
    capture_status: "timed_out_without_trusted_output",
    trusted_output: false,
    monitor_evidence_status: "untrusted",
    trust_basis: "none",
  });
  Object.assign(flash.fixed_serial_assessment, {
    startup_complete: false,
    startup_failed: true,
    issues: ["error_diagnostic", "startup_failed", "startup_incomplete"],
  });
  await writeFile(flashPath, JSON.stringify(flash));
  await saveRestartFailure(f.root, f.context, "restart_install_identity_startup");
  await writeNew(resolve(f.root, "host-cleanup.json"), {
    schema: "worker-restart-host-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
  const owner = { pid: 202, pgid: 202, startedAt: "synthetic-supervisor" };
  await writeNew(resolve(f.root, "supervisor.host-root.json"), owner);
  await writeNew(resolve(f.root, "supervisor-stop-request.json"), {
    schema: "parent-observed-supervisor-stop-v1",
    owner,
    owned: [owner],
    target: owner,
    signal: "SIGTERM",
  });
  await writeNew(resolve(f.root, "supervisor.observation.json"), {
    schema: "hello-passive-command-observation-v1",
    rootObserved: true,
    observations: 10,
    seen: [owner],
    failures: [],
    complete: false,
  });
  const detectorOwner = { pid: 303, pgid: 303, startedAt: "synthetic-detector" };
  await writeNew(resolve(f.root, "detector.detect.host-root.json"), detectorOwner);
  await writeNew(resolve(f.root, "detector.detect.observer-armed.json"), detectorOwner);
  await writeNew(resolve(f.root, "detector.detect.observation.json"), {
    schema: "hello-passive-command-observation-v1",
    rootObserved: true,
    observations: 2,
    seen: [detectorOwner],
    remaining: [],
    failures: [],
    started_at_unix_ms: 1000,
    finished_at_unix_ms: 2000,
  });
  const facts = inspectStorageFailureCapture(bytes, f.context);
  const seal = {
    schema: "fixed-usb-restart-storage-install-failed-inventory-v1",
    outcome: "unverified_storage_http_startup_failure",
    context_sha256: digest(JSON.stringify(f.context)),
    auditor_sha256: RESTART_STORAGE_FAILURE_PRODUCER,
    artifact_snapshot_sha256: await fileDigest(resolve(f.root, "artifact-snapshot.json")),
    initial_flash_sha256: await fileDigest(flashPath),
    capture_sha256: input.capture_sha256,
    install_input_sha256: await fileDigest(resolve(f.root, "install-review-input.json")),
    install_observer_sha256: await fileDigest(resolve(f.root, "install-001.observation.json")),
    detector_observer_sha256: await fileDigest(resolve(f.root, "detector.detect.observation.json")),
    failure_sha256: await fileDigest(resolve(f.root, "restart-failure.json")),
    cleanup_sha256: await fileDigest(resolve(f.root, "host-cleanup.json")),
    before_accounting_sha256: await fileDigest(resolve(scope, "no-mining-accounting-before.json")),
    preinstall_failure_review_sha256: await fileDigest(resolve(f.root, "before-install-failure-review.json")),
    last_authenticated_ledger: f.ledger,
    installation_consumed: true,
    controlled_restart_consumed: false,
    fresh_accounting_after_install: false,
    authenticated_preservation_after_install: false,
    qualification_pass: false,
    continuation_authority: false,
    mining_authorized: false,
    ...facts,
    supplemental_observer: {
      sha256: await fileDigest(resolve(f.root, "supervisor.observation.json")),
      complete: false,
      observations: 10,
      cleanup_proven_by_supplement: false,
    },
    files: await inventory(f.root),
  };
  await writeNew(resolve(f.root, "failed-inventory.json"), seal);
  return f;
}
export async function storageRestartFixture(t, { preflight = true } = {}) {
  const old = await failedStorageRestartFixture(t),
    nextCommit = "e".repeat(40),
    artifacts = await artifactFixture({ base: old.base, oldContext: { gate_commit: old.context.gate_commit } }, nextCommit);
  const manifest = await readJson(resolve(artifacts.sourceRoot, "firmware/bitaxe-ultra205-package.json"));
  await writeFile(old.options.manifest, JSON.stringify(manifest));
  for (const item of manifest.artifacts)
    await copyFile(
      resolve(artifacts.sourceRoot, "firmware", item.path),
      resolve(item.kind === "partition_table" ? old.options.firmwareRoot : dirname(old.options.manifest), item.path),
    );
  for (const name of ["license-inventory", "provenance-manifest"])
    await copyFile(
      resolve(artifacts.sourceRoot, `firmware/docs/release/${name}.md`),
      resolve(old.options.firmwareRoot, `docs/release/${name}.md`),
    );
  const input = resolve(old.base, "restart-storage-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [RESTART_STORAGE_FAILURE_SHA256],
  });
  const options = {
    ...old.options,
    input,
    privateRoot: resolve(dirname(old.root), "restart-4"),
    firmwareCommit: nextCommit,
    supersedeInstallFailure: old.root,
  };
  const operations = {
    ...old.operations,
    readInstallFailure: async (root, ...args) => {
      if (root !== old.root) return old.operations.readInstallFailure(root, ...args);
      const verified = await inspectRestartStorageFailure(root, old.context);
      return { ...verified, binding: { root, failed_inventory_sha256: RESTART_STORAGE_FAILURE_SHA256 } };
    },
  };
  if (preflight) await restartPreflight(options, operations);
  return {
    ...old,
    old,
    root: options.privateRoot,
    options,
    operations,
    context: preflight ? (await readJson(resolve(options.privateRoot, "context.json"))).context : undefined,
  };
}
