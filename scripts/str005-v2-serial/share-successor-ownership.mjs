import { proof, canonical } from "../str005-noise-serial/files.mjs";
import { validateCommandObservation } from "../str005-noise-serial/install.mjs";
import { checkedOwner, sameProcess } from "./host-resources.mjs";
import { check, sha256, uint } from "./values.mjs";
export { collectCurrentInstallOwnership as collectCurrentShareOwnership,
  validateInstallOwnershipObservation as validateShareOwnershipObservation } from "./install-successor-ownership.mjs";

/** Current ownership derives from actual recorded actors; unknown historical exits stay unknown. */
export async function inspectShareOwnershipEvidence(root, context, states, installed) {
  const hash = sha256(JSON.stringify(context));
  const interruption = (await proof(root, "parent-observations/interruption.json")).value;
  const cleanup = (await proof(root, "parent-observations/resumed-cleanup.json")).value;
  const support = (await proof(root, "parent-observations/support-inventory.json")).value;
  check(interruption.schema === "str005-v2-resumed-parent-absence-v1" && interruption.source === "resumed-operator" &&
    interruption.contextSha256 === hash && interruption.originalParentAbsent === true && interruption.originalParentExitCode === null &&
    interruption.supervisorObservedAlive === true && interruption.ownedBrowserTabClosed === true &&
    interruption.browserObservation === "native-ui-retained-state-closed-and-serial-released" &&
    interruption.freshPostInstallBaselineObserved === false && canonical(interruption.completedInstallationIndices) === "[1]" &&
    interruption.miningIssuanceObserved === false && interruption.fixtureStarted === false, "v2_share_successor_interruption");
  uint(interruption.observedAtUnixMs);
  check(cleanup.schema === "str005-v2-resumed-cleanup-v1" && cleanup.source === "resumed-operator" && cleanup.contextSha256 === hash &&
    cleanup.supervisorSignal === "SIGTERM" && cleanup.supervisorAbsent === true && cleanup.ownedResourcesAbsent === true &&
    cleanup.supervisorExitCode === null && cleanup.exitCodeObserved === false && cleanup.originalParentExitCode === null &&
    cleanup.completeHistoricalCleanupClaimed === false && cleanup.requestedAtUnixMs >= interruption.observedAtUnixMs &&
    cleanup.completedAtUnixMs >= cleanup.requestedAtUnixMs, "v2_share_successor_cleanup");
  uint(cleanup.completedAtUnixMs);
  check(support.schema === "str005-v2-interrupted-parent-support-v1" && support.source === "resumed-operator" && support.contextSha256 === hash &&
    support.originalParentExitCode === null && support.supervisorExitCode === null && support.freshPostInstallBaselineObserved === false &&
    Array.isArray(support.files) && support.files.length === 6, "v2_share_successor_support");
  const sealed = (await proof(root, "sealed-inventory.json")).value.files;
  for (const entry of [...support.files, ...interruption.originalFiles])
    check(sealed.some(row => canonical(row) === canonical(entry)), "v2_share_successor_support_bytes");
  const last = states.at(-1);
  check(last.state.status === "closed" && last.state.connected === false && last.state.serialOwnershipReleased === true,
    "v2_share_successor_browser");
  const parent = (await proof(root, "parent-observations/operator/parent-root.json")).value;
  const server = (await proof(root, "server-owner.json")).value;
  checkedOwner(parent); checkedOwner(server.owner);
  check(parent.pid === parent.pgid && server.owner.ppid === parent.pid && sameProcess(parent, interruption.originalParent) &&
    sameProcess(server.owner, interruption.supervisor) && server.contextSha256 === hash && server.origin === `http://127.0.0.1:${server.port}` &&
    sameProcess(server.owner, (await proof(root, "parent-observations/operator/supervisor-root.json")).value), "v2_share_successor_owner_join");
  const detected = (await proof(root, "install-1.detect.observation.json")).value; validateCommandObservation(detected);
  const detector = (await proof(root, "install-1.detect.host-root.json")).value;
  check(sameProcess(detector, (await proof(root, "install-1.detect.observer-armed.json")).value) &&
    detected.seen.some(owner => sameProcess(owner, detector)), "v2_share_successor_detector_owner");
  const owners = [], parentPids = new Set();
  for (const [index, owner] of [parent, server.owner, detector, ...detected.seen, ...installed.owners].entries()) {
    checkedOwner(owner);
    if (!owners.some(old => sameProcess(old, owner))) owners.push(owner);
    if (index !== 0 && owner.ppid > 0) parentPids.add(owner.ppid);
  }
  for (const owner of owners) parentPids.delete(owner.pid);
  // The public-trust child has no recorded start identity. An occupied PID must fail closed.
  parentPids.add((await proof(root, "signer-01.exit.json")).value.observation.pid);
  return { owners, parentPids: [...parentPids].sort((a, b) => a - b), serialPorts: [installed.claim.detector.port],
    supervisorPort: server.port, minimumObservedAtUnixMs: cleanup.completedAtUnixMs };
}
