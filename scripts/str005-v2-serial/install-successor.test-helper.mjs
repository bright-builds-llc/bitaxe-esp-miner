// Explicit synthetic device/publication prerequisites; real file readers and finalizer remain in use.
import { mkdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { cleanupContextFixture } from "./context-fixtures.mjs";
import { state, ledger, original } from "../str005-noise-serial/test-fixture.mjs";
import { qualification, installed } from "./completed-fixture.mjs";
import { createJournal, saveAccounting } from "./journal.mjs";
import { inventory, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";
import { finalize, review } from "./finalize.mjs";
import { loadContext } from "./context.mjs";
import { sourceInventory } from "./context-sources.mjs";
import { SUCCESSOR_MODULES } from "./successor-sources.mjs";

const supportNames = ["operator/error-1789683483424.json", "operator/error-1789684044094.json", "operator/initial-detection.log",
  "operator/parent-root.json", "operator/parent.mjs", "operator/readiness-adapter.json", "operator/supervisor-root.json",
  "supervisor.serve.stdout.log", "supervisor.serve.raw.stdout.log", "supervisor.serve.stderr.log", "qualified-node-preflight-launch.json",
  "channel-003-unclaimed-parent-start.json", "channel-003-preflight.log", "channel-003-qualified-node-preflight.log"];

async function initialStates(f) {
  const { root, context } = f;
  await writeNew(resolve(f.previous.root, "install-0.claim.json"), { detector: { physical: "c".repeat(64) } });
  await writeNew(resolve(f.previous.root, "sealed-inventory.json"), { files: [{ path: "install-0.claim.json",
    sha256: (await proof(f.previous.root, "install-0.claim.json")).sha256 }] });
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
  const { root, context } = f, { person } = await installed(f, 0, true), unix = 1_800_000_000_000;
  const detector = (await proof(root, "install-0.detect.observation.json")).value.seen[0];
  await writeNew(resolve(root, "install-0.detect.host-root.json"), detector);
  await writeNew(resolve(root, "install-0.detect.observer-armed.json"), detector);
  await mkdir(resolve(root, "install-0"), { mode: 0o700 });
  await writeNew(resolve(root, "install-0/flash-command-evidence.json"), {
    command_kind: "flash-monitor", board: "205", flash_status: "completed", capture_mode: "noninteractive", capture_status: "timed_out_after_trusted_output",
    monitor_evidence_status: "trusted", trusted_output: true, commit_ready: true, firmware_commit: context.firmware_commit,
    observed_firmware_commit: context.firmware_commit, reference_commit: context.reference_commit, trust_basis: "fixed_serial",
    nvs_seed_status: "not_provided", redaction_mode: "commit-redacted", capture_timeout_seconds: 30, manifest_path: "[redacted-path]",
    timestamp: String(Math.floor(unix / 1000)), fixed_serial_assessment: { execution_present: true, safe_baseline_confirmed: true,
      startup_complete: true, startup_failed: false, stable_boot: true, issues: [] } });
  await f.put(resolve(root, "install-0/flash-monitor.log"), ["worker_owner_prepare", "usb_installed", "wifi_driver_prepared"].map(stage =>
    `usb_memory_checkpoint stage=${stage} free_bytes=200000 largest_block_bytes=100000 reserve_bytes=98304 redacted=true`).join("\n"));
  await writeNew(resolve(root, "install-0.observation.json"), { schema: "hello-passive-command-observation-v1", rootObserved: true,
    observations: 10, started_at_unix_ms: unix - 1000, finished_at_unix_ms: unix + 1000, seen: [person], remaining: [], failures: [], observer_effects: "process-metadata-only" });
  await writeNew(resolve(root, "install-0.exit.json"), { schema: "noise-serial-command-exit-v2", contextSha256: sha256(JSON.stringify(context)), index: 0,
    code: 0, ownerSha256: (await proof(root, "install-0.host-root.json")).sha256, observationSha256: (await proof(root, "install-0.observation.json")).sha256 });
  for (const name of ["install-0.stdout.log", "install-0.stderr.log", "install-0.detect.stderr.log"]) await f.put(resolve(root, name), "");
}
async function permissionFacts(f, hash) {
  const { root } = f;
  await writeNew(resolve(root, "failure.json"), { schema: "str005-v2-first-failure-v1", contextSha256: hash, code: "v2_operation_failed",
    atHostMs: 703528, deviceCause: null, sourceSequence: null });
  const entries = [{ path: "install-0", modeBefore: "0o755", kind: "directory" }];
  for (const name of ["flash-command-evidence.json", "flash-monitor.log"]) {
    const path = `install-0/${name}`, data = await readFile(resolve(root, path));
    entries.push({ path, modeBefore: "0o644", kind: "file", sha256: sha256(data), length: data.length });
  }
  await writeNew(resolve(root, "parent-install-permissions.json"), { schema: "str005-v2-parent-install-permissions-v1", source: "parent-observed",
    contextSha256: hash, installation: 0, originalFailureSha256: (await proof(root, "failure.json")).sha256, readOnlyDiagnosis: "private_path_policy",
    permissionRepairOnly: true, acceptanceRestored: false, entries });
}
async function parentFacts(f, journal, hash) {
  const { root } = f, parent = { pid: 81000, ppid: 91000, pgid: 81000, startedAt: "synthetic-parent" };
  await mkdir(resolve(root, "parent-observations"), { mode: 0o700 });
  const server = { pid: 81001, ppid: parent.pid, pgid: 81001, startedAt: "synthetic-server" }, now = Date.now() - 1000;
  await writeNew(resolve(root, "server.claim.json"), { schema: "str005-v2-server-claim-v1", contextSha256: hash });
  await writeNew(resolve(root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256: hash, owner: server,
    origin: "http://127.0.0.1:32123", port: 32123, atHostMs: 0 });
  await writeNew(resolve(root, "parent-observations/browser-closure.json"), { schema: "noise-serial-browser-closure-v2", source: "parent-observed",
    contextSha256: hash, closed: true, lastSequence: 9, lastStateSha256: sha256(JSON.stringify(journal.lastState())), observedAtUnixMs: now });
  await writeNew(resolve(root, "parent-observations/browser-provenance.json"), { schema: "str005-v2-parent-browser-observation-v1",
    source: "CUA-owned-tab-close-and-parent-journal-join", ownedTabClosed: true, candidateFreshRestorationCollected: false, deviceControlAfterFailedReview: false });
  await writeNew(resolve(root, "parent-cleanup-failure.json"), { schema: "str005-v2-parent-cleanup-failure-v1", source: "parent-observed", contextSha256: hash,
    stage: "browser_witness", code: "v2_parent_cleanup_state", observedAtUnixMs: now + 10 });
  const exited = { schema: "noise-serial-process-exit-v2", source: "parent-observed", contextSha256: hash, owner: server, code: 0,
    observedAtUnixMs: now + 20, clock: "node-hrtime-ms-v1", stopRequestedAtMs: 1000, exitedAtMs: 1010 };
  await writeNew(resolve(root, "parent-cleanup-supervisor.json"), exited);
  await writeNew(resolve(root, "parent-cleanup-supervisor-observation.json"), { ...exited, schema: "str005-v2-parent-process-observation-v1", signal: null });
  const special = {
    "operator/error-1789683483424.json": { event: "operator_error", code: "v2_install_review_failed" },
    "operator/error-1789684044094.json": { event: "operator_error", code: "v2_parent_cleanup_state" },
    "operator/parent-root.json": parent, "operator/supervisor-root.json": server,
    "channel-003-unclaimed-parent-start.json": { schema: "str005-v2-unclaimed-parent-observation-v1", source: "parent-observed", contextSha256: hash,
      reason: "correct_stdin_routing_before_serve_launch", parentToolExitCode: 1, serverClaimAbsent: true, serverOwnerAbsent: true, operatorDirectoryAbsent: true, deviceEffects: false },
  };
  const files = [];
  for (const name of supportNames) {
    const data = special[name] ? `${JSON.stringify(special[name], null, 2)}\n` : "explicit synthetic parent support; no effects\n";
    const path = `parent-observations/${name}`; await f.put(resolve(root, path), data);
    files.push({ path, sha256: sha256(data), length: Buffer.byteLength(data) });
  }
  await writeNew(resolve(root, "parent-observations/support-inventory.json"), { schema: "str005-v2-parent-support-v1", source: "parent-observed", files,
    operatorExitCode: 0, deviceFreshRestorationCollected: false, noWorkOrFixtureStarted: true });
  return { server, parent };
}

export async function installSuccessorFixture(t) {
  const f = await cleanupContextFixture(t), hash = sha256(JSON.stringify(f.context));
  const { journal, closing } = await initialStates(f); f.time = 526032;
  await flashFacts(f); await permissionFacts(f, hash);
  await journal.state("before", { ...closing, serialOwnershipReleased: true }, 1206272);
  await journal.state("before", { ...closing, status: "closed", serialOwnershipReleased: true }, 1206280);
  const owners = await parentFacts(f, journal, hash);
  f.operations.processSnapshot = async () => [];
  const sealed = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  if (sealed.status !== "unverified" || (await proof(f.root, "judgment-failure.json")).value.code !== "v2_recorded_failure")
    throw new Error("synthetic original failure was not exercised");
  const required = [...f.context.native_source_files, ...f.context.native_auditor_sources, ...SUCCESSOR_MODULES];
  const publishedSources = await sourceInventory(f.context.firmware_root, required);
  const operations = { ...f.operations,
    inspectHistoricalInstallChannel: async path => ({ context: await loadContext(path, { historical: true, operations: f.operations }), reviewed: await review(path, f.operations) }),
    git: () => "f".repeat(40), publishedCheckerPaths: async () => publishedSources.map(row => row.path),
    publishedCheckerSources: async (_repo, _commit, paths) => paths.map(path => structuredClone(publishedSources.find(row => row.path === path))),
  };
  return { ...f, operations, sealed, ...owners };
}

/** Mutation tests rebuild only their explicitly synthetic seal; real evidence is never changed. */
export async function resealInstallFixture(f) {
  const path = resolve(f.root, "sealed-inventory.json"), value = (await proof(f.root, "sealed-inventory.json")).value;
  value.files = await inventory(f.root, new Set(["sealed-inventory.json", "projection.json"]));
  await f.put(path, `${JSON.stringify(value, null, 2)}\n`);
}
