import { lstat, readFile, readdir } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { inventory, privateRoot, proof, protectedPath, canonical } from "../str005-noise-serial/files.mjs";
import { serialNodes, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { checkedOwner } from "./host-resources.mjs";
import { readJournal } from "./journal.mjs";
import { verifySnapshot } from "./snapshot.mjs";
import { check, digest, object, port, sha256, uint } from "./values.mjs";

export const PERMISSION_PATH = "docs/hardware/str005-v2-serial-permission-amendment.md";
export const PERMISSION_SHA256 = "e0ec62fd7248d202f85868cac848294cd8261ced3a63b129d25fa32777d285cd";
export const FAILED_CONTEXT_SHA256 = "1eca9c340cec2786933552b2e549eff5536ca751c3192b17f431e98bdded1aae";
const PIN = { contextFile: "642358686f0238a7b90a692ffa67dd221d7b678a4cb270bc5f4de5cd1057d572",
  preflight: "2346447784cc52d71b4f4745258bf6ecf0543a172fa25689ca8052092c505c0b",
  attempt: "b6f8647534a3c564343594dd5cba34fdb34c1fa42b9af28193d6a0391d42a117",
  operator: "734c58e2ff69ad47ef14ecf9cf5e6cadf91a01b3c3c806cdf7d747079deadb30" };
const RUNTIME = ["server.claim.json", "server-owner.json", "failure.json", ...Array.from({ length: 6 }, (_, i) => `state-000${i + 1}.json`)];
const OPERATOR = ["browser.json", "initial-detection.log", "no-admission-resources.json", "parent.mjs", "supervisor-exit.json", "supervisor-root.json"];
const SIBLING_PINS = ["57a3473ff04ca3d9d60c26de252acc0f5522eef09045a9afc60b0ec86bdddee7",
  "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "e64256ea9b0bff4379c3abc8f2f0667ebec8576e5bbd7922026f074faf0627da"];

export function closurePath(root) { return `${root}.permission-closure.json`; }
async function raw(path) { await protectedPath(path); return readFile(path); }
async function requireLayout(root, files) {
  const allowedFiles = new Set(files), allowedDirectories = new Set();
  for (const file of files) {
    let parent = dirname(file);
    while (parent !== ".") { allowedDirectories.add(parent); const next = dirname(parent); check(next !== parent, "v2_permission_inventory_path"); parent = next; }
  }
  async function visit(path, prefix) {
    for (const name of await readdir(path)) {
      const relative = prefix ? `${prefix}/${name}` : name, absolute = resolve(path, name), metadata = await lstat(absolute);
      await protectedPath(absolute, metadata.isDirectory());
      check(metadata.isDirectory() ? allowedDirectories.has(relative) : allowedFiles.delete(relative), "v2_permission_inventory_membership");
      if (metadata.isDirectory()) await visit(absolute, relative);
    }
  }
  await visit(root, ""); check(allowedFiles.size === 0, "v2_permission_inventory_membership");
}
async function pinnedContext(root, inputs) {
  const stored = await proof(root, "context.json");
  check(stored.sha256 === PIN.contextFile && stored.value.sha256 === FAILED_CONTEXT_SHA256 &&
    inputs.attempt.length === 777 && sha256(JSON.stringify(inputs.attempt)) === PIN.attempt &&
    inputs.operator.length === 6 && sha256(JSON.stringify(inputs.operator)) === PIN.operator &&
    inputs.siblings.every((row, index) => row.sha256 === SIBLING_PINS[index]), "v2_permission_failure_anchor");
  // The pinned v1 branch has only its accepted Noise ancestor, never another permission closure.
  const { loadContext } = await import("./context.mjs");
  const context = await loadContext(root, { historical: true });
  check(context.firmware_commit === "979f7ba23881ba0921ebf726dd34f08cf4405acf" &&
    context.gate_commit === "9251a1f1fbacb35093db0dc0a75a1a87206abcb0", "v2_permission_pair");
  return context;
}
async function inputInventory(root) {
  const siblings = [];
  for (const path of [`${root}.serve.stdout.log`, `${root}.serve.stderr.log`, resolve(dirname(root), "channel-ordinal-1.json")]) {
    const bytes = await raw(path); siblings.push({ path: basename(path), sha256: sha256(bytes), length: bytes.length });
  }
  return { attempt: await inventory(root), operator: await inventory(`${root}.operator`), siblings };
}
function journalBoundary(rows, context, failure) {
  check(rows.length === 6, "v2_permission_journal_count");
  const statuses = ["configured", "configured", "configured", "failed", "closing", "closed"];
  const hash = sha256(JSON.stringify(context));
  for (const [index, row] of rows.entries()) {
    const state = row.state;
    object(state, ["schema", "gateCommit", "status", "connected", "running", "heartbeatSuppressed", "renewalsConfirmed", "serialOwnershipReleased",
      "deviceRestorationConfirmed", "deviceBaselineConfirmed", "deviceLeaseInactive", "expectedFirmwareSourceCommit", "expectedAppElfSha256",
      ...(index >= 2 ? ["serialFailureCategory", "admissionFailureStage"] : []), ...(index >= 3 ? ["failure"] : [])]);
    check(row.phase === "before" && row.contextSha256 === hash && state.status === statuses[index] &&
      state.connected === false && state.running === false && state.heartbeatSuppressed === false && state.renewalsConfirmed === 0 &&
      state.deviceRestorationConfirmed === false && state.deviceBaselineConfirmed === false && state.deviceLeaseInactive === false &&
      state.serialOwnershipReleased === (index !== 1) && (index < 2 || (state.admissionFailureStage === "permission" && state.serialFailureCategory === "operation_failed")) &&
      (index < 3 || state.failure === "connect_failed"), "v2_permission_before_open");
  }
  object(failure, ["schema", "contextSha256", "code", "atHostMs", "deviceCause", "sourceSequence"]); uint(failure.atHostMs);
  check(failure.schema === "str005-v2-first-failure-v1" && failure.contextSha256 === hash && failure.code === "v2_browser_failed" &&
    failure.deviceCause === null && failure.sourceSequence === 4 && failure.atHostMs >= rows[3].atHostMs && failure.atHostMs <= rows[4].atHostMs,
  "v2_permission_first_failure");
}
async function witnesses(root, context, rows) {
  const directory = `${root}.operator`, hash = sha256(JSON.stringify(context));
  const server = (await proof(root, "server-owner.json")).value, claim = (await proof(root, "server.claim.json")).value;
  object(claim, ["schema", "contextSha256"]);
  object(server, ["schema", "contextSha256", "owner", "origin", "port", "atHostMs"]);
  checkedOwner(server.owner); port(server.port); uint(server.atHostMs);
  check(claim.schema === "str005-v2-server-claim-v1" && claim.contextSha256 === hash && server.schema === "str005-v2-server-owner-v1" &&
    server.contextSha256 === hash && server.origin === `http://127.0.0.1:${server.port}` && server.atHostMs <= rows[0].atHostMs, "v2_permission_server");
  check((await raw(`${root}.serve.stderr.log`)).length === 0 &&
    (await raw(`${root}.serve.stdout.log`)).toString("utf8") === `qualification_url=${server.origin}/\n{"supervisor":"closed","hardware_qualified":false}\n`,
  "v2_permission_launch_log");
  const browser = (await proof(directory, "browser.json")).value, exited = (await proof(directory, "supervisor-exit.json")).value;
  const owner = (await proof(directory, "supervisor-root.json")).value; checkedOwner(owner);
  object(browser, ["schema", "source", "contextSha256", "closed", "lastSequence", "lastStateSha256", "observedAtUnixMs"]);
  check(browser.schema === "noise-serial-browser-closure-v2" && browser.source === "parent-observed" && browser.contextSha256 === hash &&
    browser.closed === true && browser.lastSequence === 6 && browser.lastStateSha256 === sha256(JSON.stringify(rows[5])), "v2_permission_browser_closure");
  uint(browser.observedAtUnixMs);
  object(exited, ["schema", "source", "contextSha256", "owner", "code", "observedAtUnixMs", "clock", "stopRequestedAtMs", "exitedAtMs"]);
  checkedOwner(exited.owner); for (const key of ["observedAtUnixMs", "stopRequestedAtMs", "exitedAtMs"]) uint(exited[key]);
  check(exited.schema === "noise-serial-process-exit-v2" && exited.source === "parent-observed" && exited.contextSha256 === hash &&
    sameProcess(server.owner, owner) && sameProcess(owner, exited.owner) && exited.code === 0 && exited.clock === "node-hrtime-ms-v1" &&
    exited.exitedAtMs >= exited.stopRequestedAtMs && exited.exitedAtMs - exited.stopRequestedAtMs <= 5000, "v2_permission_supervisor_exit");
  const detection = (await raw(resolve(directory, "initial-detection.log"))).toString("utf8");
  const ports = [...detection.matchAll(/^port:\s*(\S+)\s*$/gmu)], physical = [...detection.matchAll(/^physical_identity_sha256:\s*([a-f0-9]{64})\s*$/gmu)];
  check(ports.length === 1 && physical.length === 1 && /^usb_profile:\s*serial_jtag_runtime\s*$/mu.test(detection), "v2_permission_detector");
  const nodes = serialNodes(ports[0][1]), resources = (await proof(directory, "no-admission-resources.json")).value;
  object(resources, ["schema", "source", "contextSha256", "observedAtUnixMs", "observations"]); uint(resources.observedAtUnixMs);
  check(resources.schema === "str005-v2-no-admission-resources-v1" && resources.source === "parent-observed" && resources.contextSha256 === hash &&
    resources.observedAtUnixMs >= browser.observedAtUnixMs && resources.observedAtUnixMs >= exited.observedAtUnixMs &&
    Array.isArray(resources.observations) && resources.observations.length === 2, "v2_permission_resources");
  const expected = [["-nP", `-iTCP:${server.port}`, "-sTCP:LISTEN", "-t"], ["-Fpn", "--", ...nodes]];
  resources.observations.forEach((row, index) => {
    object(row, ["program", "args", "exitCode", "stdoutBytes", "stderrBytes"]);
    check(row.program === "lsof" && canonical(row.args) === canonical(expected[index]) && row.exitCode === 1 && row.stdoutBytes === 0 && row.stderrBytes === 0,
      "v2_permission_resource_observation");
  });
  return { server, browser, exited, resources, serialPort: nodes[0] };
}

/** Derive the only permitted class; no ordinary baseline/positive judge is relaxed. */
export async function inspectPermissionInputs(root, operations = {}) {
  check(typeof root === "string" && root === resolve(root) && basename(root) === "channel-001", "v2_permission_root");
  await privateRoot(root); await privateRoot(dirname(root)); await privateRoot(`${root}.operator`);
  const initialContext = await proof(root, "context.json"), initialPreflight = await proof(root, "preflight-inventory.json");
  if (!operations.inspectFailedContext) check(initialContext.sha256 === PIN.contextFile && initialPreflight.sha256 === PIN.preflight,
    "v2_permission_failure_anchor");
  object(initialPreflight.value, ["schema", "files"]);
  check(Array.isArray(initialPreflight.value.files) && initialPreflight.value.files.length <= 4096, "v2_permission_preflight_bound");
  for (const row of initialPreflight.value.files) { object(row, ["path", "sha256", "length"]); digest(row.sha256); uint(row.length); }
  const expectedFiles = [...initialPreflight.value.files.map(row => row.path), "preflight-inventory.json", ...RUNTIME];
  check(expectedFiles.every(path => typeof path === "string" && !path.startsWith("/") && !path.split("/").includes("..")), "v2_permission_inventory_path");
  await requireLayout(root, expectedFiles); await requireLayout(`${root}.operator`, OPERATOR);
  const inputs = await inputInventory(root);
  const context = operations.inspectFailedContext ? await operations.inspectFailedContext(root) : await pinnedContext(root, inputs);
  check(context.schema === "str005-v2-serial-context-v1" && context.scope === "channel" && context.hostOrdinal === 1 &&
    !Object.hasOwn(context, "permissionSupersession") && root === resolve(context.firmware_root, "scratch/str005-v2-serial/channel-001"), "v2_permission_context");
  const stored = (await proof(root, "context.json")).value;
  check(stored.sha256 === sha256(JSON.stringify(context)) && canonical(stored.context) === canonical(context), "v2_permission_context_changed");
  await verifySnapshot(root, context);
  const artifact = (await proof(root, "artifact-snapshot.json")).value;
  check(artifact.files.length === 13, "v2_permission_artifact_count");
  const preflight = (await proof(root, "preflight-inventory.json")).value;
  const expected = [...preflight.files.map(row => row.path), "preflight-inventory.json", ...RUNTIME].sort();
  check(canonical(inputs.attempt.map(row => row.path).sort()) === canonical(expected) &&
    canonical(inputs.operator.map(row => row.path)) === canonical(OPERATOR), "v2_permission_inventory_membership");
  const assignment = (await proof(dirname(root), "channel-ordinal-1.json")).value;
  object(assignment, ["schema", "root", "scope", "context_sha256"]);
  check(assignment.schema === "str005-v2-serial-assignment-v1" && assignment.root === root && assignment.scope === "channel" &&
    assignment.context_sha256 === stored.sha256, "v2_permission_assignment");
  const rows = await readJournal(root, context), failure = (await proof(root, "failure.json")).value;
  journalBoundary(rows, context, failure);
  const observed = await witnesses(root, context, rows);
  check(canonical(inputs) === canonical(await inputInventory(root)), "v2_permission_input_drift");
  return { root, context, contextSha256: stored.sha256, inputs, failure, ...observed };
}
