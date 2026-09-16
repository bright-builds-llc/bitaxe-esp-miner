import { mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { exactObject, missing } from "../fixed-usb-qualification/contract.mjs";
import { parseCleanupReceipt } from "./evidence.mjs";
import { readJournal, readNoiseJournal, baseline } from "./journal.mjs";
import { requireGone, requireLsofAbsent, requireNoHolders, sameProcess } from "./host-resources.mjs";
import { check, digest, privateRoot, proof, writeNew } from "./files.mjs";

/** Revalidate fixed parent witnesses and current kernel absence; no process is signalled. */
export async function inspectCleanup(root, context, receiptPath, operations = {}) {
  const cleanupRoot = operations.cleanupSnapshot ? resolve(root, "final-inputs/cleanup") : `${root}.cleanup`;
  check(resolve(receiptPath) === resolve(`${root}.cleanup`, "receipt.json"), "noise_cleanup_path");
  await privateRoot(cleanupRoot);
  const recorded = await proof(cleanupRoot, "receipt.json"), value = parseCleanupReceipt(recorded.value);
  check(value.contextSha256 === digest(JSON.stringify(context)), "noise_cleanup_context");
  for (const key of ["browserClosed", "browserOwnershipReleased", "fixtureExited", "supervisorExited", "listenerAbsent", "serialHoldersAbsent", "deviceResourcesReleased"])
    check(value[key] === true, "noise_cleanup_incomplete");
  check(value.fixtureExitCode === 0 && value.supervisorExitCode === 0 && value.remainingOwnedProcesses === 0, "noise_cleanup_exit");
  const witnesses = {};
  for (const key of ["browser", "fixture", "supervisor", "resources"]) {
    const p = await proof(cleanupRoot, `${key}.json`); check(p.sha256 === value.witnesses[key], "noise_cleanup_witness_changed"); witnesses[key] = p.value;
  }
  const rows = await readJournal(root, context), last = rows.at(-1); check(last, "noise_final_state_missing"); baseline(last.state, true);
  const browser = witnesses.browser;
  exactObject(browser, ["schema", "source", "contextSha256", "closed", "lastSequence", "lastStateSha256", "observedAtUnixMs"]);
  check(browser.schema === "noise-serial-browser-closure-v2" && browser.source === "parent-observed" && browser.closed === true &&
    browser.contextSha256 === value.contextSha256 && browser.lastSequence === last.sequence &&
    browser.lastStateSha256 === digest(JSON.stringify(last)) && browser.observedAtUnixMs <= value.observedAtHostUnixMs, "noise_browser_closure_join");
  const server = (await proof(root, "server-owner.json")).value, fixture = (await proof(root, "fixture-owner.json")).value;
  const fixtureExit = (await proof(root, "fixture-exit.json")).value;
  const supervisorExit = witnesses.supervisor;
  exactObject(supervisorExit, ["schema", "source", "contextSha256", "owner", "code", "observedAtUnixMs", "clock", "stopRequestedAtMs", "exitedAtMs"]);
  check(supervisorExit.schema === "noise-serial-process-exit-v2" && supervisorExit.source === "parent-observed" &&
    supervisorExit.contextSha256 === value.contextSha256 && supervisorExit.code === 0 && sameProcess(supervisorExit.owner, server.owner) &&
    supervisorExit.observedAtUnixMs <= value.observedAtHostUnixMs && supervisorExit.clock === "node-hrtime-ms-v1" &&
    Number.isSafeInteger(supervisorExit.stopRequestedAtMs) && Number.isSafeInteger(supervisorExit.exitedAtMs) &&
    supervisorExit.stopRequestedAtMs >= 0 && supervisorExit.exitedAtMs >= supervisorExit.stopRequestedAtMs &&
    supervisorExit.exitedAtMs - supervisorExit.stopRequestedAtMs <= 5000, "noise_process_exit_join");
  const fixtureWitness = witnesses.fixture;
  exactObject(fixtureWitness, ["schema", "source", "contextSha256", "owner", "code", "exitReceiptSha256"]);
  check(fixtureWitness.schema === "noise-serial-fixture-exit-witness-v2" && fixtureWitness.source === "fixture-owner-observed" &&
    fixtureWitness.contextSha256 === value.contextSha256 && sameProcess(fixtureWitness.owner, fixture.owner) &&
    fixtureWitness.code === 0 && fixtureExit.code === 0 &&
    fixtureWitness.exitReceiptSha256 === (await proof(root, "fixture-exit.json")).sha256, "noise_fixture_owner_join");
  const resources = witnesses.resources;
  exactObject(resources, ["schema", "contextSha256", "owners", "port", "serialPort", "observedAtUnixMs"]);
  check(resources.schema === "noise-serial-resource-check-v2" && resources.contextSha256 === value.contextSha256 &&
    resources.port === server.port && resources.serialPort === (await proof(root, "install-4.claim.json")).value.detector.port &&
    resources.owners.some((owner) => sameProcess(owner, server.owner)) && resources.owners.some((owner) => sameProcess(owner, witnesses.fixture.owner)), "noise_cleanup_owners");
  if (operations.checkKernel !== false) {
    await requireGone(resources.owners, operations);
    requireLsofAbsent(["-nP", `-iTCP:${server.port}`, "-sTCP:LISTEN", "-t"], operations);
    requireLsofAbsent(["-nP", `-iTCP:${(await proof(root, "fixture-run/ready.json")).value.listenPort}`, "-sTCP:LISTEN", "-t"], operations);
    await requireAllSerialHoldersAbsent(root, operations);
  }
  return { sha256: recorded.sha256, value, root: cleanupRoot, witnesses: value.witnesses,
    cleanupMs: supervisorExit.exitedAtMs - supervisorExit.stopRequestedAtMs };
}

/** After actual external exits, bind parent observations; never infer an unobserved exit. */
export async function recordCleanup(root, context, input, operations = {}) {
  exactObject(input, ["browser", "supervisor"]);
  const fixtureExit = await proof(root, "fixture-exit.json"), fixtureOwner = (await proof(root, "fixture-owner.json")).value;
  input = { ...input, fixture: { schema: "noise-serial-fixture-exit-witness-v2", source: "fixture-owner-observed",
    contextSha256: digest(JSON.stringify(context)), owner: fixtureOwner.owner, code: fixtureExit.value.code, exitReceiptSha256: fixtureExit.sha256 } };
  const cleanupRoot = `${root}.cleanup`; await missing(cleanupRoot); await mkdir(cleanupRoot, { mode: 0o700 });
  const owner = (await proof(root, "server-owner.json")).value;
  const serialPort = (await proof(root, "install-4.claim.json")).value.detector.port;
  const resources = { schema: "noise-serial-resource-check-v2", contextSha256: digest(JSON.stringify(context)),
    owners: [input.supervisor.owner, input.fixture.owner], port: owner.port, serialPort, observedAtUnixMs: Date.now() };
  await requireGone(resources.owners, operations);
  requireLsofAbsent(["-nP", `-iTCP:${owner.port}`, "-sTCP:LISTEN", "-t"], operations); await requireAllSerialHoldersAbsent(root, operations);
  for (const key of ["browser", "supervisor", "fixture"]) await writeNew(resolve(cleanupRoot, `${key}.json`), input[key]);
  await writeNew(resolve(cleanupRoot, "resources.json"), resources);
  const witnesses = {};
  for (const key of ["browser", "supervisor", "fixture", "resources"]) witnesses[key] = (await proof(cleanupRoot, `${key}.json`)).sha256;
  const states = await readNoiseJournal(root, context), resourcesReleased = states.at(-1)?.status.job?.resources;
  const released = resourcesReleased?.socketState === "closed" && resourcesReleased?.workerState === "quiescent" && resourcesReleased?.volatileInputsDisposed;
  const receipt = { schema: "noise-serial-cleanup-v1", contextSha256: resources.contextSha256, browserClosed: input.browser.closed,
    browserOwnershipReleased: true, fixtureExited: true, fixtureExitCode: input.fixture.code, supervisorExited: true, supervisorExitCode: input.supervisor.code,
    remainingOwnedProcesses: 0, listenerAbsent: true, serialHoldersAbsent: true, deviceResourcesReleased: released === true, observedAtHostUnixMs: Date.now(), witnesses };
  await writeNew(resolve(cleanupRoot, "receipt.json"), receipt);
  await inspectCleanup(root, context, resolve(cleanupRoot, "receipt.json"), operations);
  return { cleanup_recorded: true };
}

/** Seal only after all admitted host writers are actually gone, even for failure. */
export async function requireHostStopped(root, operations = {}) {
  let maybeServer;
  try { maybeServer = (await proof(root, "server-owner.json")).value; }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  if (!maybeServer) { await missing(resolve(root, "server.claim.json")); return; }
  const owners = [maybeServer.owner];
  try { owners.push((await proof(root, "fixture-owner.json")).value.owner); }
  catch (error) {
    if (error.code !== "ENOENT") throw error;
    await missing(resolve(root, "fixture-start.claim.json"));
  }
  await requireGone(owners, operations);
  requireLsofAbsent(["-nP", `-iTCP:${maybeServer.port}`, "-sTCP:LISTEN", "-t"], operations);
  await requireAllSerialHoldersAbsent(root, operations);
}

async function requireAllSerialHoldersAbsent(root, operations) {
  const ports = new Set();
  for (let index = 0; index <= 4; index++) {
    try { ports.add((await proof(root, `install-${index}.claim.json`)).value.detector.port); }
    catch (error) { if (error.code !== "ENOENT") throw error; }
  }
  for (const port of ports) requireNoHolders(port, operations);
}
