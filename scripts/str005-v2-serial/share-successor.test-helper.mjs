// Explicit synthetic device/publication prerequisites; real file readers and finalizer remain in use.
import { mkdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { installContextFixture } from "./context-fixtures.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { qualification, installed } from "./completed-fixture.mjs";
import { createJournal, saveAccounting } from "./journal.mjs";
import { inventory, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";
import { finalize, review } from "./finalize.mjs";
import { loadContext } from "./context.mjs";
import { sourceInventory } from "./context-sources.mjs";
import { SUCCESSOR_MODULES } from "./successor-sources.mjs";

async function initialStates(f) {
  const { root, context } = f;
  const journal = await createJournal(root, context), ready = state(context, "before");
  ready.qualification = { ...qualification(), budget_reserved_ms: 240000 };
  const configured = { ...ready, status: "configured", connected: false, deviceBaselineConfirmed: false,
    deviceRestorationConfirmed: false, deviceLeaseInactive: false, serialOwnershipReleased: true };
  delete configured.preservation; delete configured.qualification;
  await journal.state("before", configured, 407803);
  await journal.state("before", { ...configured, serialOwnershipReleased: false }, 451890);
  await journal.state("before", ready, 484310); await journal.state("before", ready, 517046);
  await saveAccounting(root, context, { stage: "before-install", ledger, original_budget: original, state: ready }, journal.lastState());
  const closing = { ...ready, status: "closing", connected: false };
  await journal.state("before", closing, 526006);
  await journal.state("before", { ...closing, serialOwnershipReleased: true }, 526026);
  await journal.state("before", { ...closing, status: "closed", serialOwnershipReleased: true }, 526032);
  return { journal, closing };
}
async function flashFacts(f) {
  const { root, context } = f, { person } = await installed(f, 1, true), unix = 1_800_000_010_000;
  const detector = (await proof(root, "install-1.detect.observation.json")).value.seen[0];
  await writeNew(resolve(root, "install-1.detect.host-root.json"), detector);
  await writeNew(resolve(root, "install-1.detect.observer-armed.json"), detector);
  await mkdir(resolve(root, "install-1"), { mode: 0o700 });
  await writeNew(resolve(root, "install-1/flash-command-evidence.json"), {
    command_kind: "flash-monitor", board: "205", flash_status: "completed", capture_mode: "noninteractive", capture_status: "timed_out_after_trusted_output",
    monitor_evidence_status: "trusted", trusted_output: true, commit_ready: true, firmware_commit: context.firmware_commit,
    observed_firmware_commit: context.firmware_commit, reference_commit: context.reference_commit, trust_basis: "fixed_serial",
    nvs_seed_status: "not_provided", redaction_mode: "commit-redacted", capture_timeout_seconds: 30, manifest_path: "[redacted-path]",
    timestamp: String(Math.floor(unix / 1000)), fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true,
      startup_complete: true, startup_failed: false, stable_boot: true, issues: [] } });
  await f.put(resolve(root, "install-1/flash-monitor.log"), ["worker_owner_prepare", "usb_installed", "wifi_driver_prepared"].map(stage =>
    `usb_memory_checkpoint stage=${stage} free_bytes=200000 largest_block_bytes=100000 reserve_bytes=98304 redacted=true`).join("\n"));
  await writeNew(resolve(root, "install-1.observation.json"), { schema: "hello-passive-command-observation-v1", rootObserved: true,
    observations: 10, started_at_unix_ms: unix - 1000, finished_at_unix_ms: unix + 1000, seen: [person], remaining: [], failures: [], observer_effects: "process-metadata-only" });
  await writeNew(resolve(root, "install-1.exit.json"), { schema: "noise-serial-command-exit-v2", contextSha256: sha256(JSON.stringify(context)), index: 1,
    code: 0, ownerSha256: (await proof(root, "install-1.host-root.json")).sha256, observationSha256: (await proof(root, "install-1.observation.json")).sha256 });
  for (const name of ["install-1.stdout.log", "install-1.stderr.log", "install-1.detect.stderr.log"]) await f.put(resolve(root, name), "");
}

async function parentFacts(f, hash) {
  const parent = { pid: 81000, ppid: 91000, pgid: 81000, startedAt: "synthetic-parent" };
  const server = { pid: 81001, ppid: parent.pid, pgid: 81001, startedAt: "synthetic-server" }, now = Date.now() - 1000;
  const put = (name, value) => f.put(resolve(f.root, name), JSON.stringify(value));
  await put("server.claim.json", { schema: "str005-v2-server-claim-v1", contextSha256: hash });
  await put("server-owner.json", { schema: "str005-v2-server-owner-v1", contextSha256: hash, owner: server,
    origin: "http://127.0.0.1:32123", port: 32123, atHostMs: 0 });
  const support = ["operator/parent-root.json", "operator/supervisor-root.json", "operator/parent.mjs", "operator/initial-detection.log", "serve.stdout.log", "serve.stderr.log"];
  const files = [];
  for (const name of support) {
    const value = name.includes("parent-root") ? parent : name.includes("supervisor-root") ? server : "explicit synthetic support";
    const path = `parent-observations/${name}`; await put(path, value);
    const bytes = await readFile(resolve(f.root, path)); files.push({ path, sha256: sha256(bytes), length: bytes.length });
  }
  await put("parent-observations/support-inventory.json", { schema: "str005-v2-interrupted-parent-support-v1", source: "resumed-operator",
    contextSha256: hash, files, originalParentExitCode: null, supervisorExitCode: null, freshPostInstallBaselineObserved: false });
  await put("parent-observations/interruption.json", { schema: "str005-v2-resumed-parent-absence-v1", source: "resumed-operator",
    contextSha256: hash, observedAtUnixMs: now, originalParent: parent, originalParentAbsent: true, originalParentExitCode: null,
    supervisor: server, supervisorObservedAlive: true, ownedBrowserTabClosed: true,
    browserObservation: "native-ui-retained-state-closed-and-serial-released", freshPostInstallBaselineObserved: false,
    completedInstallationIndices: [1], miningIssuanceObserved: false, fixtureStarted: false, originalFiles: files });
  await put("parent-observations/resumed-cleanup.json", { schema: "str005-v2-resumed-cleanup-v1", source: "resumed-operator", contextSha256: hash,
    requestedAtUnixMs: now + 1, completedAtUnixMs: now + 2, supervisorSignal: "SIGTERM", supervisorAbsent: true, ownedResourcesAbsent: true,
    supervisorExitCode: null, exitCodeObserved: false, originalParentExitCode: null, completeHistoricalCleanupClaimed: false });
  return { parent, server };
}
export async function shareSuccessorFailureFixture(t) {
  const f = await installContextFixture(t, { scope: "share" }), hash = sha256(JSON.stringify(f.context));
  await initialStates(f); f.time = 526032; await flashFacts(f);
  const { inspectInstall } = await import("../str005-noise-serial/install.mjs");
  const install = await inspectInstall(f.root, f.context, 1);
  await writeNew(resolve(f.root, "install-1.review.json"), { schema: "str005-v2-install-review-v1", contextSha256: hash, index: 1,
    beforeSequence: 7, flashSha256: install.flashSha256, observationSha256: install.observationSha256 });
  await writeNew(resolve(f.root, "signer-01.exit.json"), { schema: "str005-v2-signer-exit-v1", contextSha256: hash, index: 1, operation: "public-trust",
    observation: { pid: 81999, code: 0, signal: null, elapsedMs: 83, stdoutBytes: 712, stderrBytes: 0, overflow: false, inputFailed: false } });
  const owners = await parentFacts(f, hash); f.operations.processSnapshot = async () => [];
  const sealed = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  const required = [...f.context.native_source_files, ...f.context.native_auditor_sources, ...SUCCESSOR_MODULES];
  const publishedSources = await sourceInventory(f.context.firmware_root, required);
  const operations = { ...f.operations,
    inspectHistoricalShare: async path => ({ context: await loadContext(path, { historical: true, operations: f.operations }), reviewed: await review(path, f.operations) }),
    git: () => "f".repeat(40), publishedCheckerPaths: async () => publishedSources.map(row => row.path),
    publishedCheckerSources: async (_repo, _commit, paths) => paths.map(path => structuredClone(publishedSources.find(row => row.path === path))),
  };
  return { ...f, operations, sealed, ...owners };
}
export async function resealShareFixture(f) {
  const path = resolve(f.root, "sealed-inventory.json"), value = (await proof(f.root, "sealed-inventory.json")).value;
  value.files = await inventory(f.root, new Set(["sealed-inventory.json"]));
  await f.put(path, `${JSON.stringify(value, null, 2)}\n`);
}
