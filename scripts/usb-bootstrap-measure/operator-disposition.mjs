import { checkedOwner } from "../str005-v2-serial/host-resources.mjs";
import { resolve } from "node:path";
import { canonical, inventory, privateRoot, proof } from "../str005-noise-serial/files.mjs";
import { operatorAbsent } from "./cleanup.mjs";
import { readJournal } from "./journal.mjs";
import { contextHash } from "./context.mjs";
import { responseShape, requestShape, statusShape } from "./operator-state.mjs";
import { check, object, schema, sha256 } from "./values.mjs";
export async function readStoppedOperator(root, context, operations = {}) {
  const directory = operations.directory ?? `${root}.operator`; await privateRoot(directory);
  const disposition = await proof(directory, "disposition.json"), stopped = await proof(directory, "stopped.json"), locator = await proof(directory, "locator.json");
  const hash = contextHash(context), d = disposition.value;
  object(locator.value, ["schema", "contextSha256", "owner", "socketPath"]); checkedOwner(locator.value.owner);
  check(locator.value.schema === schema("operator-locator") && locator.value.contextSha256 === hash && locator.value.owner.pid === locator.value.owner.pgid &&
    typeof locator.value.socketPath === "string" && resolve(locator.value.socketPath) === locator.value.socketPath && Buffer.byteLength(locator.value.socketPath) <= 103 &&
    /\/ubmop-[A-Za-z0-9]+\/s$/u.test(locator.value.socketPath), "bootstrap_operator_locator");
  object(d, ["schema", "contextSha256", "owner", "sourceSha256", "hostStopped", "cleanupRecorded", "supervisorObservationSha256"]);
  check(d.schema === schema("operator-disposition") && d.contextSha256 === hash && d.hostStopped === true && typeof d.cleanupRecorded === "boolean" &&
    d.sourceSha256 === context.sourceInventory.find(row => row.path === "scripts/usb-bootstrap-measure/operator-daemon.mjs")?.sha256 &&
    JSON.stringify(d.owner) === JSON.stringify(locator.value.owner), "bootstrap_operator_disposition");
  statusShape(stopped.value, hash); check(stopped.value.phase === "stopped", "bootstrap_operator_disposition");
  const files = await inventory(directory), requests = files.filter(row => /^request-[a-f0-9]{32}\.json$/u.test(row.path));
  for (const file of requests) {
    const request = requestShape((await proof(directory, file.path)).value, hash, [0]);
    responseShape((await proof(directory, `result-${request.requestId}.json`)).value, hash, request.requestId);
  }
  if (operations.checkKernel !== false) await operatorAbsent(root, context, operations);
  return { directory, files, disposition, stopped, locator };
}
export async function judgeOperator(root, context, directory) {
  const observed = await readStoppedOperator(root, context, { directory, checkKernel: false }), expected = new Set(["install", "finish-cleanup", "stop"]), hash = contextHash(context);
  check(observed.disposition.value.cleanupRecorded === true && observed.stopped.value.maybeCode === null, "bootstrap_operator_incomplete");
  for (const file of observed.files.filter(row => /^request-[a-f0-9]{32}\.json$/u.test(row.path))) {
    const request = (await proof(directory, file.path)).value, response = (await proof(directory, `result-${request.requestId}.json`)).value;
    const phase = (await proof(directory, `phase-${request.action === "install" ? "install-0" : request.action}.json`)).value;
    object(phase, ["requestId", "requestSha256"]);
    check(phase.requestId === request.requestId && phase.requestSha256 === sha256(canonical(request)), "bootstrap_operator_phase");
    if (request.action === "stop") check(observed.stopped.value.maybeRequestId === request.requestId && observed.stopped.value.maybeAction === "stop", "bootstrap_operator_journal");
    check(expected.delete(request.action) && response.status === "succeeded" && response.code === null, "bootstrap_operator_incomplete");
  }
  check(expected.size === 0, "bootstrap_operator_incomplete");
  const server = (await proof(root, "server-owner.json")).value;
  check(server.contextSha256 === hash && server.owner.ppid === observed.disposition.value.owner.pid, "bootstrap_operator_parent");
  const last = (await readJournal(root, context)).at(-1);
  check(last && observed.stopped.value.lastSequence === last.sequence && observed.stopped.value.lastStateSha256 === sha256(JSON.stringify(last)) &&
    observed.stopped.value.maybeSupervisorOrigin === server.origin, "bootstrap_operator_journal");
  return observed;
}
