import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { canonical, proof, protectedPath } from "../str005-noise-serial/files.mjs";
import { serialNodes } from "../str005-noise-serial/host-resources.mjs";
import { validateCommandObservation } from "../str005-noise-serial/install.mjs";
import { checkedOwner, processSnapshot, requireGone, requireLsofAbsent, requireNoHolders, sameProcess } from "./host-resources.mjs";
import { check, object, port, sha256, uint } from "./values.mjs";

const SUPPORT_FILES = ["operator/error-1789683483424.json", "operator/error-1789684044094.json", "operator/initial-detection.log",
  "operator/parent-root.json", "operator/parent.mjs", "operator/readiness-adapter.json", "operator/supervisor-root.json",
  "supervisor.serve.stdout.log", "supervisor.serve.raw.stdout.log", "supervisor.serve.stderr.log", "qualified-node-preflight-launch.json",
  "channel-003-unclaimed-parent-start.json", "channel-003-preflight.log", "channel-003-qualified-node-preflight.log"].map(path => `parent-observations/${path}`);
async function supportEvidence(root, hash) {
  const support = (await proof(root, "parent-observations/support-inventory.json")).value;
  object(support, ["schema", "source", "files", "operatorExitCode", "deviceFreshRestorationCollected", "noWorkOrFixtureStarted"]);
  check(support.schema === "str005-v2-parent-support-v1" && support.source === "parent-observed" && support.operatorExitCode === 0 &&
    support.deviceFreshRestorationCollected === false && support.noWorkOrFixtureStarted === true && Array.isArray(support.files) &&
    canonical(support.files.map(row => row.path)) === canonical(SUPPORT_FILES), "v2_install_successor_support");
  for (const entry of support.files) {
    object(entry, ["path", "sha256", "length"]); uint(entry.length);
    const path = resolve(root, entry.path); await protectedPath(path); const data = await readFile(path);
    check(data.length === entry.length && sha256(data) === entry.sha256, "v2_install_successor_support_bytes");
  }
  for (const [name, code] of [["error-1789683483424.json", "v2_install_review_failed"], ["error-1789684044094.json", "v2_parent_cleanup_state"]])
    check(canonical((await proof(root, `parent-observations/operator/${name}`)).value) === canonical({ event: "operator_error", code }),
      "v2_install_successor_parent_error");
  const unclaimed = (await proof(root, "parent-observations/channel-003-unclaimed-parent-start.json")).value;
  check(canonical(unclaimed) === canonical({ schema: "str005-v2-unclaimed-parent-observation-v1", source: "parent-observed", contextSha256: hash,
    reason: "correct_stdin_routing_before_serve_launch", parentToolExitCode: 1, serverClaimAbsent: true, serverOwnerAbsent: true,
    operatorDirectoryAbsent: true, deviceEffects: false }), "v2_install_successor_unclaimed_parent");
}
async function browserEvidence(root, hash, last) {
  const b = (await proof(root, "parent-observations/browser-closure.json")).value;
  object(b, ["schema", "source", "contextSha256", "closed", "lastSequence", "lastStateSha256", "observedAtUnixMs"]); uint(b.observedAtUnixMs);
  check(b.schema === "noise-serial-browser-closure-v2" && b.source === "parent-observed" && b.contextSha256 === hash && b.closed === true &&
    b.lastSequence === last.sequence && b.lastStateSha256 === sha256(JSON.stringify(last)) && last.phase === "before" &&
    last.state.status === "closed" && !last.state.connected && last.state.serialOwnershipReleased, "v2_install_successor_browser");
  const provenance = (await proof(root, "parent-observations/browser-provenance.json")).value;
  check(canonical(provenance) === canonical({ schema: "str005-v2-parent-browser-observation-v1", source: "CUA-owned-tab-close-and-parent-journal-join",
    ownedTabClosed: true, candidateFreshRestorationCollected: false, deviceControlAfterFailedReview: false }), "v2_install_successor_browser_provenance");
  // The old before-phase baseline booleans are retained observations, not a new candidate restoration.
  const failure = (await proof(root, "parent-cleanup-failure.json")).value;
  object(failure, ["schema", "source", "contextSha256", "stage", "code", "observedAtUnixMs"]); uint(failure.observedAtUnixMs);
  check(failure.schema === "str005-v2-parent-cleanup-failure-v1" && failure.source === "parent-observed" && failure.contextSha256 === hash &&
    failure.stage === "browser_witness" && failure.code === "v2_parent_cleanup_state" && failure.observedAtUnixMs >= b.observedAtUnixMs,
    "v2_install_successor_cleanup_failure");
  return { browser: b, failure };
}
async function supervisorEvidence(root, hash, browser) {
  const server = (await proof(root, "server-owner.json")).value;
  object(server, ["schema", "contextSha256", "owner", "origin", "port", "atHostMs"]); checkedOwner(server.owner); port(server.port); uint(server.atHostMs);
  check(server.schema === "str005-v2-server-owner-v1" && server.contextSha256 === hash && server.origin === `http://127.0.0.1:${server.port}` &&
    canonical((await proof(root, "server.claim.json")).value) === canonical({ schema: "str005-v2-server-claim-v1", contextSha256: hash }),
    "v2_install_successor_server");
  const s = (await proof(root, "parent-cleanup-supervisor.json")).value;
  object(s, ["schema", "source", "contextSha256", "owner", "code", "observedAtUnixMs", "clock", "stopRequestedAtMs", "exitedAtMs"]); checkedOwner(s.owner);
  for (const key of ["observedAtUnixMs", "stopRequestedAtMs", "exitedAtMs"]) uint(s[key]);
  check(s.schema === "noise-serial-process-exit-v2" && s.source === "parent-observed" && s.contextSha256 === hash && sameProcess(s.owner, server.owner) &&
    s.code === 0 && s.clock === "node-hrtime-ms-v1" && s.exitedAtMs >= s.stopRequestedAtMs && s.exitedAtMs - s.stopRequestedAtMs <= 5000 &&
    s.observedAtUnixMs >= browser.failure.observedAtUnixMs, "v2_install_successor_supervisor_exit");
  const raw = (await proof(root, "parent-cleanup-supervisor-observation.json")).value;
  object(raw, ["schema", "source", "contextSha256", "owner", "code", "signal", "clock", "stopRequestedAtMs", "exitedAtMs", "observedAtUnixMs"]);
  check(canonical(raw) === canonical({ ...s, schema: "str005-v2-parent-process-observation-v1", signal: null }), "v2_install_successor_supervisor_join");
  check(sameProcess((await proof(root, "parent-observations/operator/supervisor-root.json")).value, server.owner), "v2_install_successor_server_parent");
  return { server, exit: s };
}
async function installationOwners(root, installed) {
  const detected = (await proof(root, "install-0.detect.observation.json")).value; validateCommandObservation(detected);
  const owner = (await proof(root, "install-0.detect.host-root.json")).value;
  const armed = (await proof(root, "install-0.detect.observer-armed.json")).value;
  checkedOwner(owner); checkedOwner(armed);
  check(sameProcess(owner, armed) && detected.seen.some(row => sameProcess(row, owner)), "v2_install_successor_detect_owner");
  return [owner, (await proof(root, "install-0.host-root.json")).value, ...detected.seen, ...installed.owners];
}

/** Parent/CUA and process witnesses remain distinct from the missing device-restoration proof. */
export async function inspectInstallOwnershipEvidence(root, context, states, installed) {
  const hash = sha256(JSON.stringify(context)); await supportEvidence(root, hash);
  const browser = await browserEvidence(root, hash, states.at(-1));
  const { server, exit } = await supervisorEvidence(root, hash, browser);
  const parent = (await proof(root, "parent-observations/operator/parent-root.json")).value; checkedOwner(parent);
  check(parent.pid === parent.pgid && server.owner.ppid === parent.pid, "v2_install_successor_parent_identity");
  const owners = [], parents = new Set();
  for (const [index, owner] of [parent, server.owner, ...await installationOwners(root, installed)].entries()) {
    const identity = checkedOwner(owner);
    if (!owners.some(old => sameProcess(old, identity))) owners.push(identity);
    // The recorded detached parent is the campaign boundary. Its own launcher
    // is outside that boundary; worker-parent references remain fail-closed.
    if (index !== 0 && owner.ppid > 0) parents.add(owner.ppid);
  }
  for (const owner of owners) parents.delete(owner.pid);
  serialNodes(installed.claim.detector.port);
  return { owners, parentPids: [...parents].sort((a, b) => a - b), serialPorts: [installed.claim.detector.port], supervisorPort: server.port,
    minimumObservedAtUnixMs: exit.observedAtUnixMs };
}

export function validateInstallOwnershipObservation(value, evidence) {
  object(value, ["schema", "source", "observedAtUnixMs", "ownerCount", "processGroupsAndChildrenAbsent", "serialNodeCount", "serialHoldersAbsent",
    "supervisorListenerAbsent", "parentReferenceCount", "parentReferencesAbsent"]); uint(value.observedAtUnixMs);
  check(value.schema === "str005-v2-install-successor-ownership-v1" && value.source === "successor-collector" &&
    value.observedAtUnixMs >= evidence.minimumObservedAtUnixMs && value.ownerCount === evidence.owners.length &&
    value.serialNodeCount === new Set(evidence.serialPorts.flatMap(serialNodes)).size && value.parentReferenceCount === evidence.parentPids.length &&
    ["processGroupsAndChildrenAbsent", "serialHoldersAbsent", "supervisorListenerAbsent", "parentReferencesAbsent"].every(key => value[key] === true),
    "v2_install_successor_ownership");
}

/** Fresh kernel gate only; historical review never recollects a vanished device/port observation. */
export async function collectCurrentInstallOwnership(evidence, operations = {}) {
  const current = await (operations.processSnapshot ?? processSnapshot)();
  await requireGone(evidence.owners, { processSnapshot: async () => current });
  check(!current.some(row => evidence.parentPids.includes(row.pid)), "v2_install_successor_parent_present");
  requireLsofAbsent(["-nP", `-iTCP:${evidence.supervisorPort}`, "-sTCP:LISTEN", "-t"], operations);
  for (const path of evidence.serialPorts) requireNoHolders(path, operations);
  const value = { schema: "str005-v2-install-successor-ownership-v1", source: "successor-collector", observedAtUnixMs: (operations.now ?? Date.now)(),
    ownerCount: evidence.owners.length, processGroupsAndChildrenAbsent: true, serialNodeCount: new Set(evidence.serialPorts.flatMap(serialNodes)).size,
    serialHoldersAbsent: true, supervisorListenerAbsent: true, parentReferenceCount: evidence.parentPids.length, parentReferencesAbsent: true };
  validateInstallOwnershipObservation(value, evidence); return value;
}
