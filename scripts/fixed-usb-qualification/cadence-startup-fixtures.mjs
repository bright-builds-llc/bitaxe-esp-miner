import { generateKeyPairSync } from "node:crypto";
import { chmod, copyFile, mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { BUNDLE, PAGE, digest, fileDigest, nonce, readJson, writeNew } from "./contract.mjs";
import { CADENCE_SCHEMA, CADENCE_TASK, CADENCE_LIMITS, cadenceValidatorDigest } from "./cadence-contract.mjs";
import { artifactFixture } from "./cadence-premining-fixtures.mjs";
import { inventory } from "./cadence-premining-evidence.mjs";
import { verifyArtifactSnapshot } from "./snapshot.mjs";
import { inspectStartupSources } from "./cadence-startup-context.mjs";
import { saveNoMiningAccounting } from "./no-mining-accounting.mjs";
import { startupRecoveryPreflight, consumeStartupRecoveryInstall } from "./cadence-startup-recovery.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const FAILED = "a".repeat(40),
  CORRECTED = "d".repeat(40),
  GATE = "b".repeat(40);
async function sources(f, commit) {
  const data = await artifactFixture({ base: f.base, oldContext: { gate_commit: GATE } }, commit);
  const manifest = await readJson(resolve(data.sourceRoot, "firmware/bitaxe-ultra205-package.json"));
  await mkdir(dirname(f.options.manifest), { recursive: true, mode: 0o700 });
  await writeFile(f.options.manifest, JSON.stringify(manifest), { mode: 0o600 });
  for (const a of manifest.artifacts)
    await copyFile(
      resolve(data.sourceRoot, "firmware", a.path),
      resolve(a.kind === "partition_table" ? f.options.firmwareRoot : dirname(f.options.manifest), a.path),
    );
  for (const name of ["license-inventory", "provenance-manifest"]) {
    const target = resolve(f.options.firmwareRoot, `docs/release/${name}.md`);
    await mkdir(dirname(target), { recursive: true, mode: 0o700 });
    await copyFile(resolve(data.sourceRoot, `firmware/docs/release/${name}.md`), target);
  }
  for (const [name, content] of [
    [BUNDLE, `fixture bundle ${GATE}`],
    [PAGE, "fixture page"],
  ]) {
    const path = resolve(f.options.gateRoot, name);
    await mkdir(dirname(path), { recursive: true, mode: 0o700 });
    await writeFile(path, content, { mode: 0o600 });
  }
  return inspectStartupSources({ ...f.options, firmwareCommit: commit }, f.operations);
}
async function retainSnapshot(root, context, f) {
  const entries = [],
    manifest = await readJson(context.manifest);
  async function retain(name, source) {
    const target = resolve(root, "qualified-artifacts", name);
    await mkdir(dirname(target), { recursive: true, mode: 0o700 });
    const bytes = await readFile(source);
    await writeFile(target, bytes, { mode: 0o600 });
    entries.push({ path: name, sha256: digest(bytes), length: bytes.length });
  }
  await retain("firmware/bitaxe-ultra205-package.json", context.manifest);
  for (const a of manifest.artifacts)
    await retain(`firmware/${a.path}`, resolve(a.kind === "partition_table" ? f.options.firmwareRoot : dirname(context.manifest), a.path));
  for (const name of ["license-inventory", "provenance-manifest"])
    await retain(`firmware/docs/release/${name}.md`, resolve(f.options.firmwareRoot, `docs/release/${name}.md`));
  await retain(`gate/${BUNDLE}`, resolve(f.options.gateRoot, BUNDLE));
  await retain(`gate/${PAGE}`, resolve(f.options.gateRoot, PAGE));
  await writeNew(resolve(root, "artifact-snapshot.json"), {
    schema: "fixed-usb-qualified-artifacts-v1",
    context_sha256: digest(JSON.stringify(context)),
    files: entries,
  });
}
function publicTrust() {
  const key = generateKeyPairSync("ed25519").publicKey.export({ format: "jwk" });
  const keys = [{ kid: "fixture", kty: "OKP", crv: "Ed25519", x: key.x, alg: "Ed25519", use: "sig", key_ops: ["verify"] }];
  return {
    profile: "bwg-worker-deployment-trust/0.2",
    updateAuthority: { issuer: "fixture-update", audience: "bwg-reference-firmware-capability/0.2", keys },
    workLeaseAuthority: { issuer: "fixture-lease", audience: "bwg-worker-controller/0.4", keys },
  };
}
export async function startupFixture(t) {
  const base = await realpath(await mkdtemp(resolve(tmpdir(), "cadence-startup-")));
  await chmod(base, 0o700);
  t.after(() => rm(base, { recursive: true, force: true }));
  for (const name of [
    "firmware/scripts/fixed-usb-qualification",
    "firmware/firmware/bitaxe/bwg",
    "gate",
    "authority",
    "attempts/prior",
    "attempts/failed",
  ])
    await mkdir(resolve(base, name), { recursive: true, mode: 0o700 });
  const firmwareRoot = resolve(base, "firmware"),
    failedRoot = resolve(base, "attempts/failed"),
    previousPath = resolve(base, "attempts/prior/result.json");
  await writeFile(resolve(firmwareRoot, "TASKS.md"), `## Active\n### ${CADENCE_TASK} | fixture\n`);
  await writeFile(resolve(firmwareRoot, "scripts/fixed-usb-qualification/fixture.mjs"), "export const fixture = true;\n");
  await writeFile(resolve(firmwareRoot, "firmware/bitaxe/bwg/deployment-trust.json"), JSON.stringify(publicTrust()));
  await writeNew(previousPath, { fixture: "previous charged result" });
  const original = {
    schema: "worker-budget-review-v1",
    campaign_match: true,
    reserved_mask: 7,
    completed_mask: 7,
    charged_ms: 240000,
    pending: false,
  };
  const previous = {
    context: { firmware_commit: "0".repeat(40), gate_commit: GATE },
    cleanup_confirmed: true,
    result: "unverified",
    original_budget: original,
    next_ordinal: 17,
    total_charged_ms: 1380000,
    original_campaign_id: nonce(),
  };
  const input = resolve(base, "progress.json");
  await writeNew(input, {
    schema: "worker-qualification-progress-v1",
    review: "verified",
    reason: "software_correction",
    evidence_sha256: ["e".repeat(64)],
  });
  const campaign = resolve(base, "campaign.json");
  await writeNew(campaign, { schema: "fixed-usb-campaign-v1", campaign_id: previous.original_campaign_id });
  const options = {
    privateRoot: resolve(base, "attempts/recovery"),
    predecessorRoot: failedRoot,
    previousReceipt: previousPath,
    input,
    originalCampaignRecord: campaign,
    firmwareRoot,
    gateRoot: resolve(base, "gate"),
    firmwareCommit: CORRECTED,
    gateCommit: GATE,
    manifest: resolve(firmwareRoot, "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"),
  };
  const operations = { ignored: () => undefined, cleanPushed: () => undefined, readPrevious: async () => previous, now: () => 2000 };
  operations.verifyArtifactSnapshot = async (root, context) =>
    root === dirname(previousPath) ? undefined : verifyArtifactSnapshot(root, context);
  const f = { base, options, operations, previous, failedRoot, original };
  const oldSource = await sources(f, FAILED);
  const observer = resolve(firmwareRoot, "bazel-bin/tools/http-transport/cadence_observer");
  await mkdir(dirname(observer), { recursive: true, mode: 0o700 });
  await writeFile(observer, "fixture observer", { mode: 0o700 });
  await writeFile(resolve(failedRoot, "cadence-observer.bin"), "fixture observer", { mode: 0o600 });
  const context = {
    schema: CADENCE_SCHEMA,
    cadence_diagnostics_version: 2,
    ...oldSource,
    firmware_root: firmwareRoot,
    gate_root: options.gateRoot,
    manifest: options.manifest,
    owner_stack_minimum_bytes: 4096,
    suggested_difficulty: 1000,
    required_no_mining_cycles: 4,
    cadence_limits: CADENCE_LIMITS,
    cadence_observer: { path: observer, sha256: await fileDigest(observer) },
    cadence_validator_sha256: await cadenceValidatorDigest(firmwareRoot),
    qualification_attempt: {
      schema: "worker-qualification-attempt-v1",
      id: nonce(),
      ordinal: 17,
      purpose: "normal",
      maximumActiveMilliseconds: 180000,
    },
    previous_receipt: previousPath,
    previous_receipt_sha256: await fileDigest(previousPath),
    expected_charged_ms: 1380000,
    original_campaign_id: previous.original_campaign_id,
    progress_path: input,
    progress_sha256: await fileDigest(input),
  };
  await writeNew(resolve(failedRoot, "context.json"), { context, sha256: digest(JSON.stringify(context)) });
  await writeNew(resolve(base, "attempts/ordinal-17.json"), { context_sha256: digest(JSON.stringify(context)), attempt_root: failedRoot });
  await retainSnapshot(failedRoot, context, f);
  await mkdir(resolve(failedRoot, "flash-0"), { mode: 0o700 });
  await writeNew(resolve(failedRoot, "flash-0/flash-command-evidence.json"), {
    flash_status: "completed",
    monitor_evidence_status: "untrusted",
    trusted_output: false,
    firmware_commit: FAILED,
    nvs_seed_status: "not_provided",
    fixed_serial_assessment: { startup_complete: false, startup_failed: true, stable_boot: false, retained_failure_history: true },
  });
  const cleanup = {
    schema: "worker-cadence-unused-host-cleanup-v1",
    source: "parent-observed",
    browser_opened: false,
    supervisor_exit_code: 0,
    flash_wrapper_exit_code: 1,
    flash_observer_exit_code: 0,
    host_cleanup_checked: true,
    supervisor_absent: true,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
    journal_absent: true,
    observer_activity_absent: true,
    issuance_artifacts_absent: true,
    device_restoration_claimed: false,
    fresh_ledger_claimed: false,
    startup_qualification_pass: false,
  };
  await writeNew(resolve(failedRoot, "early-host-cleanup.json"), cleanup);
  await writeNew(resolve(failedRoot, "failed-inventory.json"), {
    schema: "cpu0-cadence-initial-startup-failed-inventory-v1",
    auditor_sha256: "0e23b0bc4298bcc08d9e130b1d8246831ccb52f9260d72f84c8f62b7f408dffd",
    outcome: "unverified_initial_startup_failure",
    qualification_pass: false,
    continuation_authority: false,
    device_restoration_confirmed: false,
    fresh_device_ledger_observed: false,
    context_sha256: digest(JSON.stringify(context)),
    previous_result_sha256: await fileDigest(previousPath),
    artifact_snapshot_sha256: await fileDigest(resolve(failedRoot, "artifact-snapshot.json")),
    initial_flash_sha256: await fileDigest(resolve(failedRoot, "flash-0/flash-command-evidence.json")),
    early_host_cleanup_sha256: await fileDigest(resolve(failedRoot, "early-host-cleanup.json")),
    inventory: await inventory(failedRoot),
  });
  Object.defineProperty(operations, "expectedStartupFailureSha256", {
    value: await fileDigest(resolve(failedRoot, "failed-inventory.json")),
    enumerable: true,
  });
  f.source = await sources(f, CORRECTED);
  f.failedContext = context;
  f.observer = observer;
  return f;
}
export async function installedRecovery(t) {
  const f = await startupFixture(t);
  await startupRecoveryPreflight(f.options, f.operations);
  await consumeStartupRecoveryInstall(f.options.privateRoot, f.operations);
  const root = f.options.privateRoot,
    { context } = await readJson(resolve(root, "context.json"));
  await mkdir(resolve(root, "install-001"), { mode: 0o700 });
  await writeNew(resolve(root, "install-001/flash-command-evidence.json"), {
    flash_status: "completed",
    monitor_evidence_status: "trusted",
    trusted_output: true,
    commit_ready: true,
    firmware_commit: context.firmware_commit,
    observed_firmware_commit: context.firmware_commit,
    reference_commit: context.reference_commit,
    trust_basis: "fixed_serial",
    nvs_seed_status: "not_provided",
    redaction_mode: "commit-redacted",
    manifest_path: "[redacted-path]",
    capture_timeout_seconds: 30,
    timestamp: "3",
    fixed_serial_assessment: {
      execution_present: true,
      safe_baseline_confirmed: true,
      startup_complete: true,
      startup_failed: false,
      stable_boot: true,
      issues: [],
    },
  });
  const identity = { pid: 101, pgid: 101, startedAt: "fixture" };
  await writeNew(resolve(root, "install-001.host-root.json"), identity);
  await writeNew(resolve(root, "install-001.observer-armed.json"), identity);
  await writeNew(resolve(root, "install-001.observation.json"), {
    schema: "hello-passive-command-observation-v1",
    rootObserved: true,
    observations: 3,
    seen: [identity],
    failures: [],
    remaining: [],
    started_at_unix_ms: 1000,
    finished_at_unix_ms: 35000,
  });
  await writeNew(resolve(root, "install-owner-cleanup.json"), {
    schema: "worker-cadence-recovery-install-cleanup-v1",
    source: "parent-observed",
    command_exit_code: 0,
    root_observed: true,
    owned_children_absent: true,
    serial_holders_absent: true,
    observation_sha256: await fileDigest(resolve(root, "install-001.observation.json")),
    root_sha256: await fileDigest(resolve(root, "install-001.host-root.json")),
  });
  return { ...f, root, context };
}
export function recoveryState(context, released = false) {
  return {
    schema: "worker-serial-acceptance-v1",
    gateCommit: context.gate_commit,
    expectedFirmwareSourceCommit: context.firmware_commit,
    expectedAppElfSha256: context.app_elf_sha256,
    status: released ? "closed" : "ready",
    connected: !released,
    running: false,
    heartbeatSuppressed: false,
    renewalsConfirmed: 0,
    deviceRestorationConfirmed: false,
    deviceBaselineConfirmed: true,
    deviceLeaseInactive: true,
    serialOwnershipReleased: released,
    preservation: {
      schema: "worker-preservation-continuity-v1",
      baseline_id: Buffer.alloc(16, 3).toString("base64url"),
      device_identity_match: true,
      settings_match: true,
      authorization_high_water_match: true,
      mine_on_boot: false,
    },
  };
}
export async function recordRecoveryState(root, inner, state) {
  const { readdir } = await import("node:fs/promises");
  const count = (await readdir(root)).filter((name) => /^no-mining-state-[0-9]{4}\.json$/u.test(name)).length;
  await writeNew(resolve(root, `no-mining-state-${String(count + 1).padStart(4, "0")}.json`), {
    schema: "fixed-usb-no-mining-state-v1",
    context_sha256: digest(JSON.stringify(inner)),
    sequence: count + 1,
    state,
  });
}
export function hostCleanup() {
  return {
    schema: "worker-cadence-startup-recovery-cleanup-v1",
    source: "parent-observed",
    browser_closed: true,
    supervisor_exited: true,
    supervisor_exit_code: 0,
    listener_absent: true,
    owned_children_absent: true,
    serial_holders_absent: true,
  };
}

export async function completeRecoveryFixture(t) {
  const f = await installedRecovery(t),
    inner = f.context.no_mining_context;
  const ledger = {
    schema: "worker-qualification-ledger-v1",
    next_ordinal: 17,
    total_charged_ms: 1380000,
    pending: false,
    last_completed_ordinal: 16,
  };
  for (const stage of ["before", "after"]) {
    const state = recoveryState(inner);
    await recordRecoveryState(f.root, inner, state);
    await saveNoMiningAccounting(f.root, inner, { stage, ledger, original_budget: f.original, state });
  }
  await recordRecoveryState(f.root, inner, recoveryState(inner, true));
  const cleanupPath = resolve(f.root, "host-cleanup.json");
  await writeNew(cleanupPath, hostCleanup());
  return { ...f, ledger, cleanupPath };
}
