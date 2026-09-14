import { copyFile, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { restartFixture, recordRestartFixture, installRestartFixture } from "./reset-origin-restart-fixtures.mjs";
import { restartInnerContext, restartPreflight } from "./reset-origin-restart-context.mjs";
import { saveRestartAccounting } from "./reset-origin-restart-state.mjs";
import { recoveryState } from "./cadence-startup-fixtures.mjs";
import { artifactFixture } from "./cadence-premining-fixtures.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { BUNDLE, digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { saveRestartFailure } from "./reset-origin-restart-failure.mjs";
import {
  inspectStatisticsFailureCapture,
  inspectRestartInstallFailure,
  RESTART_INSTALL_FAILURE_SHA256,
} from "./reset-origin-restart-install-failure.mjs";

export const statisticsActive = () => ({
  category: "statistics_startup",
  authoritative: false,
  state: "active",
  errno: "unavailable",
  stack_bytes: 8192,
  stack_caps: 2052,
  before_free_bytes: 40000,
  before_largest_block_bytes: 20000,
  after_free_bytes: 42000,
  after_largest_block_bytes: 22000,
});
export const statisticsLine = (state = "active") =>
  `statistics_startup schema=v1 state=${state} errno=unavailable stack_bytes=8192 stack_caps=2052 before_free_bytes=40000 before_largest_block_bytes=20000 after_free_bytes=42000 after_largest_block_bytes=22000 redacted=true\n`;

export async function failedRestartFixture(t) {
  const f = await restartFixture(t),
    inner = restartInnerContext(f.root, f.context, "before-install"),
    scope = resolve(f.root, "before-install");
  await recordRestartFixture(scope, inner, recoveryState(inner));
  await saveRestartAccounting(scope, inner, {
    stage: "before",
    ledger: f.ledger,
    original_budget: f.original,
    state: recoveryState(inner),
  });
  await recordRestartFixture(scope, inner, recoveryState(inner, true));
  const installed = await installRestartFixture(f, { review: false }),
    capturePath = resolve(f.root, "install-001/flash-monitor.log");
  await writeFile(capturePath, (await readFile(capturePath, "utf8")).replaceAll("first_failure=none", "first_failure=statistics"));
  const input = await readJson(installed.reviewInput);
  input.capture_sha256 = await fileDigest(capturePath);
  input.command_exit_code = 1;
  await writeNew(resolve(f.root, "install-review-input.json"), input);
  const flashPath = resolve(f.root, "install-001/flash-command-evidence.json"),
    flash = await readJson(flashPath);
  flash.capture_status = "timed_out_without_trusted_output";
  flash.trusted_output = false;
  flash.monitor_evidence_status = "untrusted";
  flash.fixed_serial_assessment.startup_failed = true;
  flash.fixed_serial_assessment.issues = ["startup_failed"];
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
  const seal = {
    schema: "fixed-usb-restart-install-failed-inventory-v1",
    outcome: "unverified_statistics_startup_failure",
    auditor_sha256: "72064ed17475fcf45c96d61d7ed95b7330e4830f8c6982ec6f644c7c7abd7101",
    context_sha256: digest(JSON.stringify(f.context)),
    installation_consumed: true,
    fresh_accounting_after_install: false,
    authenticated_preservation_after_install: false,
    controlled_restart_consumed: false,
    qualification_pass: false,
    continuation_authority: false,
    mining_authorized: false,
    artifact_snapshot_sha256: await fileDigest(resolve(f.root, "artifact-snapshot.json")),
    failure_sha256: await fileDigest(resolve(f.root, "restart-failure.json")),
    cleanup_sha256: await fileDigest(resolve(f.root, "host-cleanup.json")),
    last_authenticated_ledger: f.ledger,
    before_accounting_sha256: await fileDigest(resolve(scope, "no-mining-accounting-before.json")),
    initial_flash_sha256: await fileDigest(flashPath),
    observations: inspectStatisticsFailureCapture(await readFile(capturePath), f.context).observations,
    files: await inventory(f.root),
  };
  await writeNew(resolve(f.root, "failed-inventory.json"), seal);
  return f;
}
export async function successorRestartFixture(t, { preflight = true } = {}) {
  const old = await failedRestartFixture(t),
    nextCommit = "a".repeat(40),
    nextGate = "c".repeat(40),
    artifacts = await artifactFixture({ base: old.base, oldContext: { gate_commit: nextGate } }, nextCommit);
  // Explicit synthetic decoder marker; this fixture is not an executable Gate bundle.
  await writeFile(
    resolve(old.options.gateRoot, BUNDLE),
    `fixture qualificationRestart statistics_startup schema=v1 state=active ${nextGate}`,
  );
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
  const input = resolve(old.base, "restart-successor-progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [RESTART_INSTALL_FAILURE_SHA256],
  });
  const options = {
    ...old.options,
    input,
    privateRoot: resolve(dirname(old.root), "restart-2"),
    firmwareCommit: nextCommit,
    gateCommit: nextGate,
    supersedeInstallFailure: old.root,
  };
  // The real default reader's hard anchor is tested separately; only its verified domain result is injected here.
  const operations = {
    ...old.operations,
    readInstallFailure: async (root) => {
      const verified = await inspectRestartInstallFailure(root, old.context);
      return { ...verified, binding: { root, failed_inventory_sha256: RESTART_INSTALL_FAILURE_SHA256 } };
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
export function knownFailureDiagnostics(context) {
  const known = context.before_install_failure;
  return {
    schema: "worker-diagnostic-export-v1",
    observations: [
      {
        category: "boot",
        authoritative: false,
        boot_ordinal: known.boot_ordinal,
        reset_reason: known.reset_reason,
        uptime_ms: known.last_boot_uptime_ms + 5000,
      },
      {
        category: "runtime_identity",
        authoritative: false,
        firmware_commit: context.before_source.firmware_commit,
        app_elf_sha256: context.before_source.app_elf_sha256,
      },
      {
        category: "startup",
        authoritative: false,
        stage: "runtime_ready",
        state: "complete",
        first_failure: "statistics",
        uptime_ms: known.last_startup_uptime_ms + 5000,
      },
      { category: "storage_http_status", authoritative: false, http_ready: "true", spiffs_available: "true" },
    ],
  };
}
