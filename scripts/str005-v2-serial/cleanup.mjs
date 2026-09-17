import { mkdir, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { loadContext, verifyCleanupInputs } from "./context.mjs";
import { restoredBaseline, readDeviceJournal, readJournal } from "./journal.mjs";
import { checkedOwner, installationResources, processSnapshot, requireGone, requireLsofAbsent, requireNoHolders,
  requirePoolListenerAbsent, sameProcess, signerExitProofs } from "./host-resources.mjs";
import { bytes, check, object, port, SCOPES, sha256, uint } from "./values.mjs";
const HERE = dirname(fileURLToPath(import.meta.url));
const HELPER_PATHS = ["scripts/str005-v2-serial/cleanup.mjs", "scripts/str005-v2-serial/host-resources.mjs"];
const contextHash = (context) => sha256(JSON.stringify(context));

async function helperSources(context) {
  const result = [];
  for (const path of HELPER_PATHS) {
    const digest = sha256(await readFile(resolve(HERE, path.split("/").at(-1))));
    check(context.evaluator?.some((entry) => entry.path === path && entry.sha256 === digest), "v2_cleanup_helper_changed");
    result.push({ path, sha256: digest });
  }
  return result;
}
function serverOwner(value, context) {
  object(value, ["schema", "contextSha256", "owner", "origin", "port", "atHostMs"]);
  check(value.schema === "str005-v2-server-owner-v1" && value.contextSha256 === contextHash(context), "v2_cleanup_server_context");
  port(value.port); uint(value.atHostMs); checkedOwner(value.owner);
  check(value.origin === `http://127.0.0.1:${value.port}`, "v2_cleanup_origin");
  return value;
}
function readyFacts(value, context) {
  object(value, ["schema", "contextSha256", "scope", "attemptId", "instanceId", "authorityPublicKeySha256", "owner", "readyAtMs"]);
  check(value.schema === "str005-v2-fixture-ready-facts-v1" && value.contextSha256 === contextHash(context) &&
    value.scope === context.scope && value.attemptId === context.attemptId, "v2_cleanup_readiness");
  bytes(value.instanceId, 16); checkedOwner(value.owner); uint(value.readyAtMs);
  check(/^[a-f0-9]{64}$/u.test(value.authorityPublicKeySha256), "v2_cleanup_readiness"); return value;
}
async function privateContext(server, context, operations) {
  try {
    const response = await (operations.fetch ?? fetch)(`${server.origin}/cleanup/private-context`, { method: "POST", redirect: "error",
      signal: AbortSignal.timeout(5000), headers: { Origin: server.origin, "Content-Type": "application/json" }, body: "{}" });
    check(response.ok && !response.redirected && response.body, "v2_cleanup_private_context");
    const reader = response.body.getReader(); const chunks = []; let length = 0;
    try {
      while (true) { const { value, done } = await reader.read(); if (done) break;
        length += value.length; check(length <= 4096, "v2_cleanup_private_context"); chunks.push(value); }
    } finally { await reader.cancel(); reader.releaseLock(); }
    const value = JSON.parse(Buffer.concat(chunks).toString("utf8"));
    object(value, ["schema", "contextSha256", "scope", "attemptId", "fixtureInstanceId", "fixturePort"]);
    check(value.schema === "str005-v2-cleanup-runtime-v1" && value.contextSha256 === contextHash(context) &&
      value.scope === context.scope && value.attemptId === context.attemptId, "v2_cleanup_private_context");
    port(value.fixturePort); bytes(value.fixtureInstanceId, 16); return value;
  } catch { check(false, "v2_cleanup_private_context_unproved"); }
}
/** Capture the real pool port in a closure before the supervisor exits. No private tuple leaves this API. */
export async function prepareCleanup(root, context, operations = {}) {
  root = await privateRoot(root);
  const validated = await loadContext(root, { historical: true, operations });
  check(canonical(validated) === canonical(context) && SCOPES.includes(context.scope), "v2_cleanup_context");
  for (const name of ["final-result.json", "sealed-inventory.json"]) await missing(resolve(root, name));
  await verifyCleanupInputs(context, operations);
  const sources = await helperSources(context), serverProof = await proof(root, "server-owner.json");
  const server = serverOwner(serverProof.value, context), readyProof = await proof(root, "fixture-ready.json"), ready = readyFacts(readyProof.value, context);
  const snapshot = operations.processSnapshot ?? processSnapshot;
  check((await snapshot()).some((row) => sameProcess(row, server.owner)), "v2_cleanup_server_not_live");
  const runtime = await privateContext(server, context, operations);
  check(runtime.fixtureInstanceId === ready.instanceId && (await snapshot()).some((row) => sameProcess(row, server.owner)), "v2_cleanup_instance_changed");
  let maybePrivatePort = runtime.fixturePort, used = false, busy = false;
  const prepared = { serverSha256: serverProof.sha256, readySha256: readyProof.sha256, sources, atUnixMs: Date.now() };
  return Object.freeze({
    async record(input) {
      check(!used && !busy && maybePrivatePort !== null, "v2_cleanup_already_recorded"); busy = true;
      try {
        const captured = await collectCleanup(root, context, input, prepared, maybePrivatePort, operations);
        const cleanupRoot = `${root}.cleanup`; await missing(cleanupRoot); await mkdir(cleanupRoot, { mode: 0o700 }); used = true;
        for (const key of ["browser", "supervisor", "fixture", "resources"]) await writeNew(resolve(cleanupRoot, `${key}.json`), captured[key]);
        const witnesses = {};
        for (const key of ["browser", "supervisor", "fixture", "resources"]) witnesses[key] = (await proof(cleanupRoot, `${key}.json`)).sha256;
        await writeNew(resolve(cleanupRoot, "receipt.json"), { ...captured.receipt, witnesses });
        return { cleanup_recorded: true, device_resources_released: captured.receipt.deviceResourcesReleased,
          device_baseline_confirmed: captured.receipt.deviceBaselineConfirmed };
      } finally { busy = false; if (used) maybePrivatePort = null; }
    },
  });
}
function browserWitness(value, context, last, now) {
  object(value, ["schema", "source", "contextSha256", "closed", "lastSequence", "lastStateSha256", "observedAtUnixMs"]);
  check(value.schema === "noise-serial-browser-closure-v2" && value.source === "parent-observed" && value.closed === true &&
    value.contextSha256 === contextHash(context) && last && value.lastSequence === last.sequence &&
    value.lastStateSha256 === sha256(JSON.stringify(last)), "v2_browser_closure_join");
  uint(value.observedAtUnixMs); check(value.observedAtUnixMs <= now, "v2_browser_closure_time");
}
function supervisorWitness(value, context, server, now) {
  object(value, ["schema", "source", "contextSha256", "owner", "code", "observedAtUnixMs", "clock", "stopRequestedAtMs", "exitedAtMs"]);
  checkedOwner(value.owner);
  check(value.schema === "noise-serial-process-exit-v2" && value.source === "parent-observed" && value.contextSha256 === contextHash(context) &&
    sameProcess(value.owner, server.owner) && value.clock === "node-hrtime-ms-v1" && (value.code === null || Number.isInteger(value.code)), "v2_supervisor_exit_join");
  for (const key of ["observedAtUnixMs", "stopRequestedAtMs", "exitedAtMs"]) uint(value[key]);
  check(value.observedAtUnixMs <= now && value.exitedAtMs >= value.stopRequestedAtMs && value.exitedAtMs - value.stopRequestedAtMs <= 5000,
    "v2_supervisor_exit_timing");
}
async function released(root, context) {
  const rows = await readJournal(root, context), last = rows.at(-1);
  const records = await readDeviceJournal(root, context), device = records.at(-1)?.record;
  let confirmed = false;
  try { restoredBaseline(last?.state, context, true); confirmed = last.phase === "candidate"; }
  catch (error) { if (!["v2_baseline", "v2_baseline_connection", "v2_authorization_restoration"].includes(error.code)) throw error; }
  return { last, confirmed, browserReleased: last?.state.status === "closed" && !last.state.connected && last.state.serialOwnershipReleased === true,
    deviceReleased: device?.resources.socketClosed === true && device.resources.workerQuiescent === true && device.resources.fenceRetained === false };
}
async function fixtureWitness(root, context) {
  const ownerProof = await proof(root, "fixture-owner.json"), owner = ownerProof.value;
  object(owner, ["schema", "contextSha256", "owner", "binarySha256", "atHostMs"]);
  check(owner.schema === "str005-v2-fixture-owner-v1" && owner.contextSha256 === contextHash(context) && owner.binarySha256 === context.fixture_sha256,
    "v2_cleanup_fixture_owner"); checkedOwner(owner.owner); uint(owner.atHostMs);
  const exit = await proof(root, "fixture-exit.json");
  object(exit.value, ["schema", "contextSha256", "code", "signal", "atHostMs", "owner", "stderrBytes", "lifetimeMs"]);
  checkedOwner(exit.value.owner);
  check(exit.value.schema === "str005-v2-fixture-exit-v1" && exit.value.contextSha256 === contextHash(context) &&
    sameProcess(exit.value.owner, owner.owner) && (exit.value.code === null || Number.isInteger(exit.value.code)) &&
    (exit.value.signal === null || /^SIG[A-Z0-9]+$/u.test(exit.value.signal)) && (exit.value.code !== null || exit.value.signal !== null), "v2_fixture_close_unproved");
  uint(exit.value.atHostMs); uint(exit.value.stderrBytes); if (exit.value.lifetimeMs !== null) uint(exit.value.lifetimeMs);
  const reapProof = await proof(root, "fixture-reap.json"), reap = reapProof.value;
  object(reap, ["schema", "contextSha256", "kind", "requestedAtHostMs", "completedAtHostMs", "durationMs"]);
  check(reap.schema === "str005-v2-fixture-reap-v1" && reap.contextSha256 === contextHash(context), "v2_fixture_reap_binding"); uint(reap.completedAtHostMs);
  if (reap.kind === "natural_exit") check(reap.requestedAtHostMs === null && reap.durationMs === null && reap.completedAtHostMs >= exit.value.atHostMs, "v2_fixture_natural_reap");
  else check(reap.kind === "requested_stop" && Number.isSafeInteger(reap.requestedAtHostMs) && reap.requestedAtHostMs >= 0 &&
    reap.completedAtHostMs >= exit.value.atHostMs && reap.durationMs === reap.completedAtHostMs - reap.requestedAtHostMs &&
    reap.durationMs >= 0 && reap.durationMs <= 5000, "v2_fixture_reap_timing");
  return { ownerSha256: ownerProof.sha256, ownerAtHostMs: owner.atHostMs, reapSha256: reapProof.sha256, maybeCleanupMs: reap.kind === "requested_stop" ? reap.durationMs : null,
    exit: exit.value, witness: { schema: "noise-serial-fixture-exit-witness-v2", source: "fixture-owner-observed",
    contextSha256: contextHash(context), owner: owner.owner, code: exit.value.code, exitReceiptSha256: exit.sha256 } };
}
async function collectCleanup(root, context, input, prepared, privatePort, operations) {
  object(input, ["browser", "supervisor"]);
  for (const name of ["final-result.json", "sealed-inventory.json"]) await missing(resolve(root, name));
  const now = Date.now(), serverProof = await proof(root, "server-owner.json"), server = serverOwner(serverProof.value, context);
  const ready = await proof(root, "fixture-ready.json"); readyFacts(ready.value, context);
  check(serverProof.sha256 === prepared.serverSha256 && ready.sha256 === prepared.readySha256, "v2_cleanup_source_changed");
  check(canonical(await helperSources(context)) === canonical(prepared.sources), "v2_cleanup_helper_changed");
  const device = await released(root, context);
  browserWitness(input.browser, context, device.last, now); supervisorWitness(input.supervisor, context, server, now);
  check(input.supervisor.observedAtUnixMs >= prepared.atUnixMs, "v2_cleanup_preparation_order");
  const fixture = await fixtureWitness(root, context), installs = await installationResources(root, context), signerExits = await signerExitProofs(root, context);
  check(sameProcess(ready.value.owner, fixture.witness.owner), "v2_cleanup_readiness_owner");
  const owners = [];
  for (const owner of [server.owner, fixture.witness.owner, ...installs.owners]) if (!owners.some((old) => sameProcess(old, owner))) owners.push(checkedOwner(owner));
  await requireGone(owners, operations);
  requireLsofAbsent(["-nP", `-iTCP:${server.port}`, "-sTCP:LISTEN", "-t"], operations);
  for (const path of installs.serialPorts) requireNoHolders(path, operations);
  requirePoolListenerAbsent(privatePort, operations);
  const observedAtUnixMs = Date.now();
  const resources = { schema: "str005-v2-resource-check-v1", contextSha256: contextHash(context), scope: context.scope, attemptId: context.attemptId,
    owners, supervisorPort: server.port, serialPorts: installs.serialPorts, observedAtUnixMs,
    serverOwnerSha256: serverProof.sha256, fixtureOwnerSha256: fixture.ownerSha256, fixtureReapSha256: fixture.reapSha256, fixtureReadySha256: ready.sha256,
    helperSources: prepared.sources, signerExits, poolListenerAbsent: true, poolListenerMethod: "lsof-tcp-listener-inventory-v1" };
  const receipt = { schema: "str005-v2-cleanup-v1", contextSha256: contextHash(context), scope: context.scope, attemptId: context.attemptId,
    browserClosed: true, browserOwnershipReleased: device.browserReleased === true, fixtureExited: true, fixtureExitCode: fixture.exit.code,
    supervisorExited: true, supervisorExitCode: input.supervisor.code, remainingOwnedProcesses: 0, supervisorListenerAbsent: true,
    poolListenerAbsent: true, serialHoldersAbsent: installs.serialPorts.length > 0, deviceResourcesReleased: device.deviceReleased,
    deviceBaselineConfirmed: device.confirmed, observedAtHostUnixMs: observedAtUnixMs };
  return { browser: input.browser, supervisor: input.supervisor, fixture: fixture.witness, resources, receipt };
}

/** Read-only proof check. Pool-port absence is a retained live-collector fact, not offline reconstruction. */
export async function inspectCleanup(root, context, receiptPath, operations = {}) {
  root = await privateRoot(root);
  const external = `${root}.cleanup`, snapshot = resolve(root, "final-inputs/cleanup");
  const cleanupRoot = operations.cleanupRoot ?? (operations.cleanupSnapshot ? snapshot : external);
  check([external, snapshot].includes(resolve(cleanupRoot)) && resolve(receiptPath) === resolve(external, "receipt.json"), "v2_cleanup_path");
  if (resolve(cleanupRoot) === snapshot) await privateRoot(resolve(root, "final-inputs"));
  await privateRoot(cleanupRoot);
  const proofReceipt = await proof(cleanupRoot, "receipt.json"), receipt = proofReceipt.value;
  object(receipt, ["schema", "contextSha256", "scope", "attemptId", "browserClosed", "browserOwnershipReleased", "fixtureExited", "fixtureExitCode",
    "supervisorExited", "supervisorExitCode", "remainingOwnedProcesses", "supervisorListenerAbsent", "poolListenerAbsent", "serialHoldersAbsent",
    "deviceResourcesReleased", "deviceBaselineConfirmed", "observedAtHostUnixMs", "witnesses"]);
  check(receipt.schema === "str005-v2-cleanup-v1" && receipt.contextSha256 === contextHash(context) && receipt.scope === context.scope &&
    receipt.attemptId === context.attemptId, "v2_cleanup_context"); uint(receipt.observedAtHostUnixMs);
  for (const key of ["browserClosed", "browserOwnershipReleased", "fixtureExited", "supervisorExited", "supervisorListenerAbsent",
    "poolListenerAbsent", "serialHoldersAbsent", "deviceResourcesReleased", "deviceBaselineConfirmed"]) check(receipt[key] === true, "v2_cleanup_incomplete");
  check(receipt.remainingOwnedProcesses === 0, "v2_cleanup_exit");
  if (operations.requireSuccessfulExit !== false)
    check(receipt.fixtureExitCode === 0 && receipt.supervisorExitCode === 0, "v2_cleanup_exit");
  object(receipt.witnesses, ["browser", "supervisor", "fixture", "resources"]);
  const witnesses = {};
  for (const key of Object.keys(receipt.witnesses)) {
    const item = await proof(cleanupRoot, `${key}.json`); check(item.sha256 === receipt.witnesses[key], "v2_cleanup_witness_changed"); witnesses[key] = item.value;
  }
  const device = await released(root, context);
  check(device.confirmed && device.browserReleased && device.deviceReleased, "v2_cleanup_device_unproved");
  const serverProof = await proof(root, "server-owner.json"), server = serverOwner(serverProof.value, context);
  browserWitness(witnesses.browser, context, device.last, receipt.observedAtHostUnixMs);
  supervisorWitness(witnesses.supervisor, context, server, receipt.observedAtHostUnixMs);
  check(receipt.supervisorExitCode === witnesses.supervisor.code, "v2_cleanup_exit_join");
  if (operations.requireSuccessfulExit !== false) check(witnesses.supervisor.code === 0, "v2_cleanup_exit");
  const fixture = await fixtureWitness(root, context);
  check(canonical(witnesses.fixture) === canonical(fixture.witness) && receipt.fixtureExitCode === fixture.exit.code, "v2_fixture_owner_join");
  if (operations.requireSuccessfulExit !== false) check(fixture.exit.code === 0 && fixture.exit.signal === null, "v2_fixture_owner_join");
  const readyProof = await proof(root, "fixture-ready.json"), ready = readyFacts(readyProof.value, context);
  check(sameProcess(ready.owner, fixture.witness.owner), "v2_cleanup_readiness_owner");
  check(ready.readyAtMs >= fixture.ownerAtHostMs && ready.readyAtMs - fixture.ownerAtHostMs <= 5000, "v2_fixture_readiness_deadline");
  check(fixture.exit.atHostMs >= ready.readyAtMs && fixture.exit.lifetimeMs === fixture.exit.atHostMs - ready.readyAtMs, "v2_fixture_exit_clock_join");
  if (operations.requireSuccessfulExit !== false) check(fixture.exit.stderrBytes === 0 &&
    fixture.exit.lifetimeMs <= (context.scope === "channel" ? 150000 : 300000), "v2_fixture_owner_deadline");
  const resource = witnesses.resources;
  object(resource, ["schema", "contextSha256", "scope", "attemptId", "owners", "supervisorPort", "serialPorts", "observedAtUnixMs",
    "serverOwnerSha256", "fixtureOwnerSha256", "fixtureReapSha256", "fixtureReadySha256", "helperSources", "signerExits", "poolListenerAbsent", "poolListenerMethod"]);
  check(resource.schema === "str005-v2-resource-check-v1" && resource.contextSha256 === contextHash(context) &&
    resource.scope === context.scope && resource.attemptId === context.attemptId && resource.supervisorPort === server.port &&
    resource.serverOwnerSha256 === serverProof.sha256 && resource.fixtureOwnerSha256 === fixture.ownerSha256 && resource.fixtureReapSha256 === fixture.reapSha256 && resource.fixtureReadySha256 === readyProof.sha256 &&
    resource.poolListenerAbsent === true && resource.poolListenerMethod === "lsof-tcp-listener-inventory-v1" &&
    resource.observedAtUnixMs === receipt.observedAtHostUnixMs, "v2_resource_binding");
  check(Array.isArray(resource.helperSources) && resource.helperSources.length === HELPER_PATHS.length && HELPER_PATHS.every((path) => {
    const matches = resource.helperSources.filter((entry) => entry.path === path);
    return matches.length === 1 && matches[0].sha256 === context.evaluator.find((entry) => entry.path === path)?.sha256;
  }), "v2_cleanup_collector_source");
  for (const item of resource.helperSources) object(item, ["path", "sha256"]);
  const installed = await installationResources(root, context);
  check(canonical(resource.serialPorts) === canonical(installed.serialPorts) && installed.serialPorts.length > 0 &&
    canonical(resource.signerExits) === canonical(await signerExitProofs(root, context)), "v2_resource_inventory");
  const expectedOwners = [];
  for (const owner of [server.owner, fixture.witness.owner, ...installed.owners]) if (!expectedOwners.some((old) => sameProcess(old, owner))) expectedOwners.push(checkedOwner(owner));
  check(canonical(resource.owners) === canonical(expectedOwners), "v2_cleanup_owners");
  if (operations.checkKernel !== false) {
    await requireGone(expectedOwners, operations);
    requireLsofAbsent(["-nP", `-iTCP:${server.port}`, "-sTCP:LISTEN", "-t"], operations);
    for (const path of installed.serialPorts) requireNoHolders(path, operations);
  }
  return { sha256: proofReceipt.sha256, value: receipt, root: cleanupRoot, witnesses: receipt.witnesses,
    cleanupMs: Math.max(witnesses.supervisor.exitedAtMs - witnesses.supervisor.stopRequestedAtMs, fixture.maybeCleanupMs ?? 0),
    fixtureCleanupMs: fixture.maybeCleanupMs,
    poolListenerEvidence: "source-bound-live-collector; private port intentionally not retained" };
}

/** Even an unverified finalization cannot seal while its admitted host writers remain alive. */
export async function requireHostStopped(root, context, operations = {}) {
  let maybeServer;
  try { maybeServer = (await proof(root, "server-owner.json")).value; }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  if (!maybeServer) {
    for (const name of ["server.claim.json", "fixture-owner.json", "fixture-start.claim.json"]) await missing(resolve(root, name));
    return;
  }
  const server = serverOwner(maybeServer, context), owners = [checkedOwner(server.owner)];
  try {
    const fixture = (await proof(root, "fixture-owner.json")).value;
    check(fixture.contextSha256 === contextHash(context), "v2_cleanup_fixture_owner"); owners.push(checkedOwner(fixture.owner));
  } catch (error) { if (error.code !== "ENOENT") throw error; await missing(resolve(root, "fixture-start.claim.json")); }
  const installed = await installationResources(root, context); owners.push(...installed.owners);
  await requireGone(owners, operations);
  requireLsofAbsent(["-nP", `-iTCP:${server.port}`, "-sTCP:LISTEN", "-t"], operations);
  for (const path of installed.serialPorts) requireNoHolders(path, operations);
  await signerExitProofs(root, context);
}
