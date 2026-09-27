import { resolve } from "node:path";
import { checkedOwner } from "./host-resources.mjs";
import { lstat } from "node:fs/promises";
import { inventory, proof, privateRoot } from "../str005-noise-serial/files.mjs";
import { requireGone } from "../str005-noise-serial/host-resources.mjs";
import { check, object, sha256 } from "./values.mjs";
import { requestShape, statusShape, OPERATOR_CODES } from "./operator-state.mjs";
/** Read-only input collection. Kernel absence supplies no daemon exit code. */
export async function readStoppedOperator(root, context, operations = {}) {
  const selected = operations.directory ?? `${root}.operator`;
  check(selected === `${root}.operator` || selected === resolve(root, "final-inputs/operator"), "v2_operator_owner");
  const directory = await privateRoot(selected), contextSha256 = sha256(JSON.stringify(context));
  const files = await inventory(directory), byName = new Map(files.map(item => [item.path, item]));
  check(!files.some(item => /^result-[a-f0-9]{32}\.pending\.json$/u.test(item.path)), "v2_operator_request");
  const disposition = await proof(directory, "disposition.json"), locator = await proof(directory, "locator.json"), stopped = await proof(directory, "stopped.json");
  object(disposition.value, ["schema", "contextSha256", "owner", "sourceSha256", "hostStopped", "cleanupRecorded", "supervisorObservationSha256"]);
  check(disposition.value.schema === "str005-v2-operator-disposition-v1" && disposition.value.contextSha256 === contextSha256 &&
    disposition.value.hostStopped === true && typeof disposition.value.cleanupRecorded === "boolean" &&
    disposition.value.sourceSha256 === context.evaluator.find(item => item.path === "scripts/str005-v2-serial/operator-daemon.mjs")?.sha256,
  "v2_operator_owner");
  locatorShape(locator.value, contextSha256); statusShape(stopped.value, contextSha256); checkedOwner(disposition.value.owner);
  check(locator.value.contextSha256 === contextSha256 && JSON.stringify(locator.value.owner) === JSON.stringify(disposition.value.owner) &&
    stopped.value.contextSha256 === contextSha256 && stopped.value.phase === "stopped", "v2_operator_owner");
  const claims = files.filter(item => /^request-[a-f0-9]{32}\.json$/u.test(item.path));
  const results = files.filter(item => /^result-[a-f0-9]{32}\.json$/u.test(item.path));
  check(claims.length === results.length, "v2_operator_request");
  for (const claim of claims) {
    const request = (await proof(directory, claim.path)).value; requestShape(request, contextSha256, context.install_indices);
    check(claim.path === `request-${request.requestId}.json` && byName.has(`result-${request.requestId}.json`), "v2_operator_request");
    const result = (await proof(directory, `result-${request.requestId}.json`)).value;
    object(result, ["schema", "contextSha256", "requestId", "status", "code"]);
    check(result.schema === "str005-v2-operator-response-v1" && result.contextSha256 === contextSha256 && result.requestId === request.requestId &&
      ((result.status === "succeeded" && result.code === null) || (result.status === "failed" && OPERATOR_CODES.includes(result.code))), "v2_operator_request");
  }
  if (disposition.value.supervisorObservationSha256 !== null) {
    check(disposition.value.supervisorObservationSha256 === (await proof(root, "parent-cleanup-supervisor-observation.json")).sha256, "v2_operator_owner");
  } else check(disposition.value.cleanupRecorded === false, "v2_operator_owner");
  if (operations.checkKernel !== false) {
    await requireGone([disposition.value.owner], operations);
    try { await lstat(locator.value.socketPath); check(false, "v2_operator_owner"); }
    catch (error) { if (error.code !== "ENOENT") throw error; }
  }
  return { directory, files, disposition, locator, stopped, owners: [disposition.value.owner], socketPath: locator.value.socketPath };
}

function locatorShape(value, contextSha256) {
  object(value, ["schema", "contextSha256", "owner", "socketPath"]); checkedOwner(value.owner);
  check(value.schema === "str005-v2-operator-locator-v1" && value.contextSha256 === contextSha256 && value.owner.pid === value.owner.pgid &&
    typeof value.socketPath === "string" && resolve(value.socketPath) === value.socketPath && Buffer.byteLength(value.socketPath) <= 103 && /\/v2op-[A-Za-z0-9]+\/s$/u.test(value.socketPath), "v2_operator_owner");
}
/** Absence is independently required even when no positive disposition exists. */
export async function requireOperatorAbsent(root, context, operations = {}) {
  const directory = `${root}.operator`, contextSha256 = sha256(JSON.stringify(context));
  let claim;
  try { claim = (await proof(directory, "start.claim.json")).value; }
  catch (error) {
    if (error.code !== "ENOENT") throw error;
    try { await lstat(directory); check(false, "v2_operator_owner"); } catch (missing) { if (missing.code !== "ENOENT") throw missing; }
    return { neverStarted: true, owners: [] };
  }
  object(claim, ["schema", "contextSha256"]);
  check(claim.schema === "str005-v2-operator-start-v1" && claim.contextSha256 === contextSha256, "v2_operator_owner");
  const locator = (await proof(directory, "locator.json")).value; locatorShape(locator, contextSha256);
  await requireGone([locator.owner], operations);
  try { await lstat(locator.socketPath); check(false, "v2_operator_owner"); } catch (error) { if (error.code !== "ENOENT") throw error; }
  return { neverStarted: false, owners: [locator.owner], socketPath: locator.socketPath };
}
