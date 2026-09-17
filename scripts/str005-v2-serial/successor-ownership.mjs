import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { canonical, proof, protectedPath } from "../str005-noise-serial/files.mjs";
import { serialNodes } from "../str005-noise-serial/host-resources.mjs";
import { checkedOwner, installationResources, processSnapshot, requireGone, requireLsofAbsent, requireNoHolders, sameProcess } from "./host-resources.mjs";
import { baseline } from "./journal.mjs";
import { bytes, check, digest, object, port, sha256, uint } from "./values.mjs";

const SUPPORT = "parent-observations/operator";
const SUPPORT_NAMES = ["browser.json", "error-1789625890504.json", "foreground-detection.log", "initial-detection.log", "launch-001-unclaimed.json",
  "parent-launch-002.mjs", "parent.mjs", "supervisor-exit-launch-002.json", "supervisor-root-launch-002.json", "supervisor-root.json"];

async function parentSupport(root) {
  const parent = (await proof(root, "parent-observations/parent-observation.json")).value;
  object(parent, ["schema", "source", "earliestCode", "proofScope", "poolPortAbsence", "privatePortExported", "operatorFiles"]);
  check(parent.schema === "str005-v2-parent-cleanup-failure-v1" && parent.source === "parent-observed" &&
    parent.earliestCode === "v2_listener_inventory_shape" && parent.proofScope === "protocol-and-restoration-observed;complete-host-cleanup-unverified" &&
    parent.poolPortAbsence === "not_proven" && parent.privatePortExported === false && Array.isArray(parent.operatorFiles) &&
    canonical(parent.operatorFiles.map(row => row.path)) === canonical(SUPPORT_NAMES), "v2_successor_parent_provenance");
  for (const row of parent.operatorFiles) {
    object(row, ["path", "sha256", "length"]);
    const file = await proofBytes(root, `${SUPPORT}/${row.path}`);
    check(file.sha256 === row.sha256 && file.length === row.length, "v2_successor_parent_copy");
  }
  const error = (await proof(root, `${SUPPORT}/error-1789625890504.json`)).value;
  object(error, ["event", "code"]);
  check(error.event === "operator_error" && error.code === parent.earliestCode, "v2_successor_parent_cause");
}
async function proofBytes(root, path) {
  await protectedPath(resolve(root, path)); const bytes = await readFile(resolve(root, path));
  return { sha256: sha256(bytes), length: bytes.length };
}

/** Closed historical facts prove release of owned actors, never the missing old port observation. */
export async function inspectOwnershipEvidence(root, context, states, protocol) {
  await parentSupport(root);
  check(canonical((await readdir(`${root}/final-inputs/cleanup`)).sort()) === canonical(["browser.json", "supervisor.json"]), "v2_successor_partial_cleanup");
  const hash = sha256(JSON.stringify(context)), last = states.at(-1); baseline(last?.state, true);
  const browser = await proof(root, "final-inputs/cleanup/browser.json"), supervisor = await proof(root, "final-inputs/cleanup/supervisor.json");
  check(browser.sha256 === (await proof(root, `${SUPPORT}/browser.json`)).sha256 &&
    supervisor.sha256 === (await proof(root, `${SUPPORT}/supervisor-exit-launch-002.json`)).sha256, "v2_successor_parent_witness_copy");
  const b = browser.value;
  object(b, ["schema", "source", "contextSha256", "closed", "lastSequence", "lastStateSha256", "observedAtUnixMs"]);
  uint(b.observedAtUnixMs);
  check(b.schema === "noise-serial-browser-closure-v2" && b.source === "parent-observed" && b.contextSha256 === hash && b.closed === true &&
    b.lastSequence === last.sequence && b.lastStateSha256 === sha256(JSON.stringify(last)), "v2_successor_browser_closed");
  const server = (await proof(root, "server-owner.json")).value;
  object(server, ["schema", "contextSha256", "owner", "origin", "port", "atHostMs"]); checkedOwner(server.owner); port(server.port); uint(server.atHostMs);
  check(server.schema === "str005-v2-server-owner-v1" && server.contextSha256 === hash && server.origin === `http://127.0.0.1:${server.port}`, "v2_successor_server_owner");
  const s = supervisor.value;
  object(s, ["schema", "source", "contextSha256", "owner", "code", "observedAtUnixMs", "clock", "stopRequestedAtMs", "exitedAtMs"]); checkedOwner(s.owner);
  for (const key of ["observedAtUnixMs", "stopRequestedAtMs", "exitedAtMs"]) uint(s[key]);
  check(s.schema === "noise-serial-process-exit-v2" && s.source === "parent-observed" && s.contextSha256 === hash && sameProcess(s.owner, server.owner) &&
    s.code === 0 && s.clock === "node-hrtime-ms-v1" && s.exitedAtMs >= s.stopRequestedAtMs && s.exitedAtMs - s.stopRequestedAtMs <= 5000,
    "v2_successor_supervisor_exit");
  const fixture = (await proof(root, "fixture-owner.json")).value, exited = (await proof(root, "fixture-exit.json")).value;
  object(fixture, ["schema", "contextSha256", "owner", "binarySha256", "atHostMs"]); checkedOwner(fixture.owner); uint(fixture.atHostMs);
  check(fixture.schema === "str005-v2-fixture-owner-v1" && fixture.contextSha256 === hash && fixture.binarySha256 === context.fixture_sha256, "v2_successor_fixture_owner");
  object(exited, ["schema", "contextSha256", "code", "signal", "atHostMs", "owner", "stderrBytes", "lifetimeMs"]); checkedOwner(exited.owner);
  for (const key of ["atHostMs", "stderrBytes", "lifetimeMs"]) uint(exited[key]);
  check(exited.schema === "str005-v2-fixture-exit-v1" && exited.contextSha256 === hash && sameProcess(exited.owner, fixture.owner) &&
    exited.code === 0 && exited.signal === null && exited.stderrBytes === 0 && exited.lifetimeMs <= 150000, "v2_successor_fixture_exit");
  const ready = (await proof(root, "fixture-ready.json")).value, reap = (await proof(root, "fixture-reap.json")).value;
  object(ready, ["schema", "contextSha256", "scope", "attemptId", "instanceId", "authorityPublicKeySha256", "owner", "readyAtMs"]);
  bytes(ready.attemptId, 16); bytes(ready.instanceId, 16); digest(ready.authorityPublicKeySha256); checkedOwner(ready.owner); uint(ready.readyAtMs);
  check(ready.schema === "str005-v2-fixture-ready-facts-v1" && ready.contextSha256 === hash && ready.scope === "channel" && ready.attemptId === context.attemptId && sameProcess(ready.owner, fixture.owner) &&
    ready.instanceId === protocol.fixtureTerminal.instanceId && ready.readyAtMs >= fixture.atHostMs && ready.readyAtMs - fixture.atHostMs <= 5000 &&
    exited.atHostMs - ready.readyAtMs === exited.lifetimeMs, "v2_successor_fixture_ready");
  object(reap, ["schema", "contextSha256", "kind", "requestedAtHostMs", "completedAtHostMs", "durationMs"]); uint(reap.completedAtHostMs);
  check(reap.schema === "str005-v2-fixture-reap-v1" && reap.contextSha256 === hash && reap.kind === "natural_exit" &&
    reap.requestedAtHostMs === null && reap.durationMs === null && reap.completedAtHostMs >= exited.atHostMs, "v2_successor_fixture_reap");
  const unclaimed = (await proof(root, `${SUPPORT}/launch-001-unclaimed.json`)).value;
  object(unclaimed, ["schema", "source", "contextUnclaimed", "serverOwnerAbsent", "serverClaimAbsent", "failureRecordAbsent", "owner", "childExitCode", "cause", "observedAtUnixMs"]);
  checkedOwner(unclaimed.owner); uint(unclaimed.observedAtUnixMs);
  check(unclaimed.schema === "str005-v2-unclaimed-launch-observation-v1" && unclaimed.source === "parent-observed" && unclaimed.contextUnclaimed === true &&
    unclaimed.serverOwnerAbsent === true && unclaimed.serverClaimAbsent === true && unclaimed.failureRecordAbsent === true &&
    unclaimed.childExitCode === "not_collected" && unclaimed.cause === "parent_initialization_wait_30000ms_exhausted", "v2_successor_unclaimed_launch");
  check(sameProcess((await proof(root, `${SUPPORT}/supervisor-root.json`)).value, unclaimed.owner) &&
    sameProcess((await proof(root, `${SUPPORT}/supervisor-root-launch-002.json`)).value, server.owner), "v2_successor_launch_owners");
  const installs = await installationResources(root, context), owners = [], parents = new Set(), installationOwners = [];
  for (const name of (await readdir(root)).filter(name => /^install-[0-4](?:\.detect)?\.host-root\.json$/u.test(name)).sort())
    installationOwners.push((await proof(root, name)).value);
  for (const owner of [server.owner, fixture.owner, unclaimed.owner, ...installationOwners]) {
    const identity = checkedOwner(owner); if (!owners.some(old => sameProcess(old, identity))) owners.push(identity);
    if (owner.ppid !== undefined) { uint(owner.ppid, 0x7fffffff); if (owner.ppid > 0) parents.add(owner.ppid); }
  }
  for (const owner of owners) parents.delete(owner.pid);
  return { owners, parentPids: [...parents].sort((a, b) => a - b), serialPorts: installs.serialPorts, supervisorPort: server.port,
    unclaimedOwner: checkedOwner(unclaimed.owner), minimumObservedAtUnixMs: Math.max(s.observedAtUnixMs, b.observedAtUnixMs) };
}

export function validateOwnershipObservation(value, evidence) {
  object(value, ["schema", "source", "observedAtUnixMs", "ownerCount", "processGroupsAndChildrenAbsent", "serialNodeCount", "serialHoldersAbsent",
    "supervisorListenerAbsent", "unclaimedOwnerAbsent", "parentReferenceCount", "parentReferencesAbsent"]);
  uint(value.observedAtUnixMs);
  check(value.schema === "str005-v2-successor-ownership-v1" && value.source === "successor-collector" &&
    value.observedAtUnixMs >= evidence.minimumObservedAtUnixMs && value.ownerCount === evidence.owners.length &&
    value.serialNodeCount === new Set(evidence.serialPorts.flatMap(serialNodes)).size && value.parentReferenceCount === evidence.parentPids.length &&
    ["processGroupsAndChildrenAbsent", "serialHoldersAbsent", "supervisorListenerAbsent", "unclaimedOwnerAbsent", "parentReferencesAbsent"].every(key => value[key] === true),
    "v2_successor_ownership_observation");
}

/** A separate present-time gate; never called by historical receipt review. */
export async function collectCurrentOwnership(evidence, operations = {}) {
  const current = await (operations.processSnapshot ?? processSnapshot)();
  await requireGone(evidence.owners, { processSnapshot: async () => current });
  check(!current.some(row => evidence.parentPids.includes(row.pid)), "v2_successor_parent_reference_present");
  requireLsofAbsent(["-nP", `-iTCP:${evidence.supervisorPort}`, "-sTCP:LISTEN", "-t"], operations);
  for (const path of evidence.serialPorts) requireNoHolders(path, operations);
  const value = { schema: "str005-v2-successor-ownership-v1", source: "successor-collector", observedAtUnixMs: (operations.now ?? Date.now)(),
    ownerCount: evidence.owners.length, processGroupsAndChildrenAbsent: true, serialNodeCount: new Set(evidence.serialPorts.flatMap(serialNodes)).size,
    serialHoldersAbsent: true, supervisorListenerAbsent: true, unclaimedOwnerAbsent: true,
    parentReferenceCount: evidence.parentPids.length, parentReferencesAbsent: true };
  validateOwnershipObservation(value, evidence); return value;
}
