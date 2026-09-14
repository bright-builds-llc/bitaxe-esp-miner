import { chmod, copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { completeResetOriginFixture } from "./reset-origin-fixtures.mjs";
import { judgeResetOrigin, readResetOrigin } from "./reset-origin-judge.mjs";
import { artifactFixture } from "./cadence-premining-fixtures.mjs";
import { recoveryState } from "./cadence-startup-fixtures.mjs";
import { BUNDLE, PAGE, digest, fileDigest, readJson, writeNew } from "./contract.mjs";
import { restartPreflight, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { saveRestartAccounting } from "./reset-origin-restart-state.mjs";
import { consumeRestartInstall, reviewRestartInstallation } from "./reset-origin-restart-install.mjs";

export function restartPacket(context, { reopen = false, ordinal = 7 } = {}) {
  const diag = (record, atMs, diagnostic) => ({ record, atMs, diagnostic: { authoritative: false, ...diagnostic } });
  const boot = (record) =>
    diag(record, record * 10, { category: "boot", boot_ordinal: ordinal + 1, reset_reason: "software_cpu", uptime_ms: 1000 });
  const identity = (record) =>
    diag(record, record * 10, {
      category: "runtime_identity",
      firmware_commit: context.firmware_commit,
      app_elf_sha256: context.app_elf_sha256,
    });
  const startup = (record, uptime) =>
    diag(record, record * 10, { category: "startup", stage: "runtime_ready", state: "complete", first_failure: "none", uptime_ms: uptime });
  const observations = [boot(2), identity(3), startup(4, 1000), startup(5, 2000)];
  const lifecycle = [
    { record: 0, atMs: 0, event: "prearmed" },
    { record: 1, atMs: 5, event: "acknowledged" },
  ];
  if (reopen) {
    lifecycle.push({ record: 5, atMs: 55, event: "stream_interrupted" }, { record: 5, atMs: 56, event: "same_port_reopened" });
    observations.push(boot(6), identity(7), startup(8, 3000), startup(9, 4000));
  }
  lifecycle.push(
    { record: reopen ? 9 : 5, atMs: reopen ? 95 : 55, event: "hello_started" },
    { record: reopen ? 10 : 6, atMs: 150, event: "complete" },
  );
  return {
    summary: {
      schema: "worker-qualification-restart-observation-v1",
      stage: "complete",
      ackMatched: true,
      expectedBootOrdinal: ordinal,
      nextBootOrdinal: ordinal + 1,
      bootObserved: true,
      softwareResetObserved: true,
      identityObserved: true,
      identityMatched: true,
      runtimeReadyObserved: true,
      records: reopen ? 10 : 6,
      bytes: 2000,
      durationMs: 150,
      portReopens: reopen ? 1 : 0,
      streamInterrupted: reopen,
      continuity: reopen ? "same_port_reopened" : "uninterrupted",
    },
    ack: {
      schema: "worker-qualification-restart-v1",
      requestNonceSha256: digest(context.request_nonce),
      bootOrdinal: ordinal,
      nextBootOrdinal: ordinal + 1,
    },
    observations,
    lifecycle,
  };
}
export function startupCapture(context, reason = "software_cpu") {
  return Buffer.from(
    [1000, 3000]
      .map(
        (uptime) =>
          `usb_reboot_discriminator schema=v1 boot_ordinal=7 reset_reason=${reason} uptime_ms=${uptime} redacted=true\n` +
          `usb_runtime_identity schema=v1 firmware_commit=${context.firmware_commit} app_elf_sha256=${context.app_elf_sha256} redacted=true\n` +
          `usb_boot_profile=${JSON.stringify({ schema_version: 1, transport: "serial_jtag_runtime", reason: "worker_started", baseline: "confirmed", firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256, boot_ordinal: 7 })}\n` +
          `usb_startup schema=v1 stage=runtime_ready state=complete first_failure=none uptime_ms=${uptime} redacted=true\n`,
      )
      .join(""),
  );
}
export async function restartFixture(t, { preflight = true } = {}) {
  const f = await completeResetOriginFixture(t);
  await judgeResetOrigin(f.root, f.cleanupPath, f.operations);
  const stageAResult = resolve(f.root, "result.json"),
    newCommit = "8".repeat(40),
    newGate = "9".repeat(40);
  const artifacts = await artifactFixture({ base: f.base, oldContext: { gate_commit: newGate } }, newCommit);
  const manifest = await readJson(resolve(artifacts.sourceRoot, "firmware/bitaxe-ultra205-package.json"));
  const newManifest = resolve(f.options.firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json");
  await mkdir(dirname(newManifest), { recursive: true, mode: 0o700 });
  await writeFile(newManifest, JSON.stringify(manifest), { mode: 0o600 });
  for (const item of manifest.artifacts)
    await copyFile(
      resolve(artifacts.sourceRoot, "firmware", item.path),
      resolve(item.kind === "partition_table" ? f.options.firmwareRoot : dirname(newManifest), item.path),
    );
  for (const name of ["license-inventory", "provenance-manifest"])
    await copyFile(
      resolve(artifacts.sourceRoot, `firmware/docs/release/${name}.md`),
      resolve(f.options.firmwareRoot, `docs/release/${name}.md`),
    );
  await writeFile(resolve(f.options.gateRoot, BUNDLE), `fixture qualificationRestart bundle ${newGate}`);
  await writeFile(resolve(f.options.gateRoot, PAGE), '<html><body><pre id="state"></pre></body></html>');
  const input = resolve(f.base, "restart-progress.json"),
    observerScript = resolve(f.base, "process-observer.mjs");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: [await fileDigest(stageAResult)],
  });
  await writeFile(observerScript, "// fixture process observer\n", { mode: 0o600 });
  const options = {
    privateRoot: resolve(dirname(f.root), "restart"),
    firmwareRoot: f.options.firmwareRoot,
    gateRoot: f.options.gateRoot,
    firmwareCommit: newCommit,
    gateCommit: newGate,
    manifest: newManifest,
    originalCampaignRecord: f.options.originalCampaignRecord,
    stageAResult,
    input,
    observerScript,
  };
  const operations = { ...f.operations, readStageA: (path) => readResetOrigin(path, f.operations), now: () => 4000 };
  if (preflight) await restartPreflight(options, operations);
  const context = preflight ? (await readJson(resolve(options.privateRoot, "context.json"))).context : undefined;
  return { ...f, stageARoot: f.root, root: options.privateRoot, options, operations, context };
}
export async function recordRestartFixture(root, context, state) {
  const { readdir } = await import("node:fs/promises");
  const count = (await readdir(root)).filter((name) => /^no-mining-state-[0-9]{4}\.json$/u.test(name)).length;
  await writeNew(resolve(root, `no-mining-state-${String(count + 1).padStart(4, "0")}.json`), {
    schema: "fixed-usb-restart-state-v1",
    context_sha256: digest(JSON.stringify(context)),
    sequence: count + 1,
    state,
  });
}
export async function installedRestartFixture(t, { panic = false } = {}) {
  const f = await restartFixture(t),
    before = restartInnerContext(f.root, f.context, "before-install"),
    scope = resolve(f.root, "before-install");
  await recordRestartFixture(scope, before, recoveryState(before));
  await saveRestartAccounting(scope, before, {
    stage: "before",
    ledger: f.ledger,
    original_budget: f.original,
    state: recoveryState(before),
  });
  await recordRestartFixture(scope, before, recoveryState(before, true));
  return installRestartFixture(f, { panic });
}
export async function installRestartFixture(f, { panic = false, review = true, additionalCapture = "" } = {}) {
  const owner = { pid: 101, pgid: 101, startedAt: "fixture-process" };
  await writeNew(resolve(f.root, "install-001.host-root.json"), owner);
  await writeNew(resolve(f.root, "install-001.observer-armed.json"), owner);
  await consumeRestartInstall(f.root, f.operations);
  await mkdir(resolve(f.root, "install-001"), { mode: 0o700 });
  await writeNew(resolve(f.root, "install-001/flash-command-evidence.json"), {
    flash_status: "completed",
    capture_mode: "noninteractive",
    capture_status: panic ? "timed_out_without_trusted_output" : "timed_out_after_trusted_output",
    command_kind: "flash-monitor",
    board: "205",
    firmware_commit: f.context.firmware_commit,
    observed_firmware_commit: f.context.firmware_commit,
    reference_commit: f.context.reference_commit,
    trust_basis: "fixed_serial",
    nvs_seed_status: "not_provided",
    redaction_mode: "commit-redacted",
    manifest_path: "[redacted-path]",
    capture_timeout_seconds: 30,
    timestamp: "4",
    trusted_output: !panic,
    commit_ready: !panic,
    monitor_evidence_status: panic ? "untrusted" : "trusted",
    fixed_serial_assessment: {
      execution_present: true,
      safe_baseline_confirmed: true,
      startup_complete: true,
      startup_failed: false,
      stable_boot: !panic,
      retained_failure_history: false,
      issues: panic ? ["reboot_observed", "insufficient_advancing_samples"] : [],
    },
  });
  await writeNew(resolve(f.root, "install-001.observation.json"), {
    schema: "hello-passive-command-observation-v1",
    rootObserved: true,
    observations: 10,
    failures: [],
    remaining: [],
    complete: true,
    seen: [owner],
    started_at_unix_ms: 3000,
    finished_at_unix_ms: 5000,
  });
  const captureFile = "install-001/flash-monitor.log";
  await writeFile(
    resolve(f.root, captureFile),
    Buffer.concat([startupCapture(f.context, panic ? "panic" : "software_cpu"), Buffer.from(additionalCapture)]),
    { mode: 0o600 },
  );
  const reviewInput = resolve(f.root, "installation-input.json");
  await writeNew(reviewInput, {
    schema: "worker-restart-install-review-input-v1",
    source: "parent-observed",
    capture_file: captureFile,
    capture_sha256: await fileDigest(resolve(f.root, captureFile)),
    command_exit_code: panic ? 1 : 0,
    owned_children_absent: true,
    serial_holders_absent: true,
  });
  if (review) await reviewRestartInstallation(f.root, reviewInput, f.operations);
  return { ...f, reviewInput };
}
