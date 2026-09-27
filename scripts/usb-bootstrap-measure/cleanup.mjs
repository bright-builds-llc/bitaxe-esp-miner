import { lstat, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { canonical, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { processSnapshot, requireGone, requireNoHolders, requireLsofAbsent, sameProcess, serialNodes } from "../str005-noise-serial/host-resources.mjs";
import { checkedOwner } from "../str005-v2-serial/host-resources.mjs";
import { contextHash } from "./context.mjs";
import { verifyAccounting, baseline, readJournal } from "./journal.mjs";
import { check, object, schema, sha256, uint } from "./values.mjs";
export async function maybeProof(root, name) { try { return await proof(root, name); } catch (e) { if (e.code !== "ENOENT") throw e; return null; } }
export async function browserWitness(root, context, value) {
  object(value, ["schema", "source", "contextSha256", "closed", "lastSequence", "lastStateSha256", "observedAtUnixMs"]);
  const last = (await readJournal(root, context)).at(-1); uint(value.observedAtUnixMs);
  check(value.schema === schema("browser-closure") && value.source === "native-ui-observer" && value.contextSha256 === contextHash(context) && value.closed === true &&
    value.observedAtUnixMs <= Date.now() && last && last.sequence === value.lastSequence && sha256(JSON.stringify(last)) === value.lastStateSha256 &&
    last.state.status === "closed" && !last.state.connected && last.state.serialOwnershipReleased === true, "bootstrap_browser_unproved");
  return value;
}
export async function ownedFacts(root, context) {
  const hash = contextHash(context), owners = [], observedOwners = [], serial = new Set();
  const server = await maybeProof(root, "server-owner.json");
  if (server) { check(server.value.contextSha256 === hash && server.value.origin === `http://127.0.0.1:${server.value.port}`, "bootstrap_owner"); owners.push({ role: "supervisor", identity: server.value.owner, exitObservationSha256: (await maybeProof(root, "parent-cleanup-supervisor-observation.json"))?.sha256 ?? null }); }
  for (const [role, prefix] of [["installation", "install-0"], ["detector", "install-0.detect"]]) {
    const observed = await maybeProof(root, `${prefix}.observation.json`), host = await maybeProof(root, `${prefix}.host-root.json`), exit = await maybeProof(root, `${prefix}.exit.json`);
    if (host) owners.push({ role, identity: host.value, exitObservationSha256: exit?.sha256 ?? null });
    if (observed) for (const owner of observed.value.seen) if (!observedOwners.some(row => sameProcess(row, owner))) observedOwners.push(owner);
  }
  const claim = await maybeProof(root, "install-0.claim.json"); if (claim) serial.add(claim.value.detector.port);
  for (const row of owners) checkedOwner(row.identity);
  for (const owner of observedOwners) checkedOwner(owner);
  return { owners, observedOwners, serial: [...serial], server };
}
export async function requireHostStopped(root, context, operations = {}) {
  const facts = await ownedFacts(root, context);
  const startup = await maybeProof(`${root}.operator`, "supervisor-owner.json");
  if (startup) {
    object(startup.value, ["schema", "contextSha256", "owner"]); checkedOwner(startup.value.owner);
    check(startup.value.schema === schema("operator-supervisor-owner") && startup.value.contextSha256 === contextHash(context), "bootstrap_owner");
    await requireGone([startup.value.owner], operations);
  }
  await requireGone([...facts.owners.map(row => row.identity), ...facts.observedOwners], operations);
  for (const port of facts.serial) requireNoHolders(port, operations);
  if (facts.server) requireLsofAbsent(["-nP", `-iTCP:${facts.server.value.port}`, "-sTCP:LISTEN", "-t"], operations);
  return facts;
}
export async function operatorAbsent(root, context, operations = {}) {
  const locator = await maybeProof(`${root}.operator`, "locator.json");
  if (!locator) {
    check(!(await maybeProof(`${root}.operator`, "start.claim.json")), "bootstrap_operator_unlocated"); return null;
  }
  object(locator.value, ["schema", "contextSha256", "owner", "socketPath"]); checkedOwner(locator.value.owner);
  check(locator.value.schema === schema("operator-locator") && locator.value.contextSha256 === contextHash(context) &&
    /\/ubmop-[A-Za-z0-9]+\/s$/u.test(locator.value.socketPath), "bootstrap_operator_owner");
  await requireGone([locator.value.owner], operations);
  try { await lstat(locator.value.socketPath); check(false, "bootstrap_operator_socket_present"); } catch (e) { if (e.code !== "ENOENT") throw e; }
  return locator;
}
export async function deriveCleanup(root, context, operations = {}) {
  const facts = await requireHostStopped(root, context, operations), locator = await operatorAbsent(root, context, operations);
  const disposition = await maybeProof(`${root}.operator`, "disposition.json"), preliminary = await maybeProof(root, "cleanup-facts.json");
  let complete = false;
  if (disposition && preliminary && locator) {
    const d = disposition.value;
    check(d.contextSha256 === contextHash(context) && d.sourceSha256 === context.sourceInventory.find(row => row.path === "scripts/usb-bootstrap-measure/operator-daemon.mjs")?.sha256 &&
      sameProcess(d.owner, locator.value.owner) && d.hostStopped === true, "bootstrap_operator_disposition");
    try {
      const accounting = await verifyAccounting(root, context); baseline(accounting.last.state, true);
      await browserWitness(root, context, preliminary.value.browserWitness);
      const exit = (await proof(root, "parent-cleanup-supervisor-observation.json")).value;
      check(exit.code === 0 && exit.signal === null && preliminary.value.contextSha256 === contextHash(context) && preliminary.value.complete === true &&
        d.cleanupRecorded === true && d.supervisorObservationSha256 === (await proof(root, "parent-cleanup-supervisor-observation.json")).sha256,
        "bootstrap_cleanup_incomplete");
      complete = true;
    } catch (e) { if (e.code === "EACCES") throw e; }
  }
  if (locator) facts.owners.push({ role: "daemon", identity: locator.value.owner, exitObservationSha256: null });
  const hash = async name => (await maybeProof(root, name))?.sha256 ?? null;
  const value = { schema: schema("cleanup"), contextSha256: contextHash(context), browserWitnessSha256: await hash("browser-closure.json"),
    baselineReceiptSha256: await hash("restoration.json"), accountingBeforeSha256: await hash("accounting-before.json"), accountingAfterSha256: await hash("accounting-after.json"),
    owners: facts.owners, operatorDispositionSha256: disposition?.sha256 ?? null,
    absence: { observedAtUnixMs: Date.now(), serialNodeCount: new Set(facts.serial.flatMap(serialNodes)).size, serialHoldersAbsent: true,
      supervisorListenerAbsent: true, operatorSocketAbsent: true, ownedGroupsAbsent: true }, complete };
  if (complete) { try { await validateCleanup(root, context, value); } catch { value.complete = false; } }
  return value;
}

/** Pure historical joins. Kernel absence is an attested observation, never recollected by review. */
export async function validateCleanup(root, context, value, operatorDirectory) {
  object(value, ["schema", "contextSha256", "browserWitnessSha256", "baselineReceiptSha256", "accountingBeforeSha256", "accountingAfterSha256", "owners", "operatorDispositionSha256", "absence", "complete"]);
  const hash = contextHash(context); check(value.schema === schema("cleanup") && value.contextSha256 === hash && typeof value.complete === "boolean", "bootstrap_cleanup_changed");
  object(value.absence, ["observedAtUnixMs", "serialNodeCount", "serialHoldersAbsent", "supervisorListenerAbsent", "operatorSocketAbsent", "ownedGroupsAbsent"]);
  uint(value.absence.observedAtUnixMs); uint(value.absence.serialNodeCount);
  check(["serialHoldersAbsent", "supervisorListenerAbsent", "operatorSocketAbsent", "ownedGroupsAbsent"].every(key => value.absence[key] === true), "bootstrap_cleanup_absence");
  for (const owner of value.owners) { object(owner, ["role", "identity", "exitObservationSha256"]); checkedOwner(owner.identity); check(["daemon", "supervisor", "installation", "detector"].includes(owner.role), "bootstrap_cleanup_owner"); }
  if (!value.complete) return false;
  await validateResourceCleanup(root, context, value, operatorDirectory);
  await verifyAccounting(root, context);
  for (const [key, name] of [["baselineReceiptSha256", "restoration.json"], ["accountingBeforeSha256", "accounting-before.json"], ["accountingAfterSha256", "accounting-after.json"]])
    check(value[key] === (await proof(root, name)).sha256, "bootstrap_cleanup_restoration");
  return true;
}

/** Resource observations stand independently of missing accounting; never promotes a receipt. */
export async function validateResourceCleanup(root, context, value, operatorDirectory) {
  const hash = contextHash(context);
  const facts = await ownedFacts(root, context), directory = operatorDirectory ?? `${root}.operator`;
  const disposition = await proof(directory, "disposition.json"), locator = await proof(directory, "locator.json"), stopped = await proof(directory, "stopped.json");
  check(disposition.sha256 === value.operatorDispositionSha256 && disposition.value.contextSha256 === hash && disposition.value.hostStopped === true && disposition.value.cleanupRecorded === true &&
    disposition.value.sourceSha256 === context.sourceInventory.find(row => row.path === "scripts/usb-bootstrap-measure/operator-daemon.mjs")?.sha256 &&
    sameProcess(disposition.value.owner, locator.value.owner) && stopped.value.phase === "stopped" && stopped.value.contextSha256 === hash, "bootstrap_cleanup_disposition");
  const expected = [...facts.owners, { role: "daemon", identity: locator.value.owner, exitObservationSha256: null }];
  check(canonical(value.owners) === canonical(expected) && value.owners.length === 4 && new Set(value.owners.map(row => row.role)).size === 4 &&
    value.absence.serialNodeCount === new Set(facts.serial.flatMap(serialNodes)).size, "bootstrap_cleanup_owner");
  const preliminary = (await proof(root, "cleanup-facts.json")).value;
  object(preliminary, ["schema", "contextSha256", "browserWitness", "supervisorObservationSha256", "complete"]);
  const browser = await proof(root, "browser-closure.json"), supervisor = await proof(root, "parent-cleanup-supervisor-observation.json"), s = supervisor.value;
  check(preliminary.schema === schema("cleanup-facts") && preliminary.contextSha256 === hash && preliminary.complete === true &&
    canonical(preliminary.browserWitness) === canonical(browser.value) && preliminary.supervisorObservationSha256 === supervisor.sha256 &&
    disposition.value.supervisorObservationSha256 === supervisor.sha256 && browser.sha256 === value.browserWitnessSha256, "bootstrap_cleanup_facts");
  await browserWitness(root, context, browser.value);
  object(s, ["schema", "source", "contextSha256", "owner", "code", "signal", "observedAtUnixMs", "clock", "stopRequestedAtMs", "exitedAtMs"]);
  check(s.schema === schema("supervisor-exit") && s.source === "parent-observed" && s.contextSha256 === hash && s.clock === "node-hrtime-ms-v1" &&
    sameProcess(s.owner, facts.server.value.owner) && s.owner.ppid === locator.value.owner.pid && s.code === 0 && s.signal === null &&
    Number.isSafeInteger(s.stopRequestedAtMs) && Number.isSafeInteger(s.exitedAtMs) && s.exitedAtMs >= s.stopRequestedAtMs && s.exitedAtMs - s.stopRequestedAtMs <= 5000,
    "bootstrap_cleanup_supervisor");
  const child = (await proof(directory, "supervisor-close.json")).value;
  object(child, ["schema", "source", "contextSha256", "parent", "childPid", "code", "signal", "observedAtUnixMs"]);
  checkedOwner(child.parent); uint(child.observedAtUnixMs);
  check(child.source === "parent-observed" && child.schema === schema("operator-child-exit") && child.contextSha256 === hash && child.childPid === s.owner.pid && child.code === s.code && child.signal === s.signal &&
    sameProcess(child.parent, locator.value.owner), "bootstrap_cleanup_supervisor");
  const detector = (await proof(root, "install-0.detect.exit.json")).value, detectorOwner = (await proof(root, "install-0.detect.host-root.json")).value;
  object(detector, ["schema", "contextSha256", "pid", "code", "signal"]);
  check(detector.schema === schema("detector-exit") && detector.contextSha256 === hash && detector.pid === detectorOwner.pid && detector.code === 0 && detector.signal === null,
    "bootstrap_cleanup_detector");
  await (await import("./capture-exit.mjs")).verifyCaptureExit(root, context, { requireComplete: false });
  return true;
}
