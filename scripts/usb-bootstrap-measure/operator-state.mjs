import { resolve } from "node:path";
import { link, unlink } from "node:fs/promises";
import { proof, writeNew, canonical } from "../str005-noise-serial/files.mjs";
import { check, object, sha256 } from "./values.mjs";
export const OPERATOR_CODES = ["bootstrap_operator_failed", "bootstrap_operator_initialization", "bootstrap_operator_conflict", "bootstrap_operator_busy", "bootstrap_operator_request", "bootstrap_operator_owner", "bootstrap_operator_protocol", "bootstrap_operator_stopped"];
export const operatorCode = error => OPERATOR_CODES.includes(error?.code) ? error.code : "bootstrap_operator_failed";
export function requestShape(input, contextSha256, indices) {
  object(input, ["schema", "contextSha256", "requestId", "action", "payload"]);
  check(input.schema === "usb-bootstrap-measure-operator-request-v1" && input.contextSha256 === contextSha256 && /^[a-f0-9]{32}$/u.test(input.requestId), "bootstrap_operator_request");
  check(["install", "finish-cleanup", "stop"].includes(input.action), "bootstrap_operator_request");
  object(input.payload, input.action === "install" ? ["index"] : input.action === "finish-cleanup" ? ["browserWitness"] : []);
  if (input.action === "install") check(indices.includes(input.payload.index), "bootstrap_operator_request");
  return input;
}
export function queryShape(input, contextSha256) {
  object(input, ["schema", "contextSha256", "maybeRequestId"]);
  check(input.schema === "usb-bootstrap-measure-operator-query-v1" && input.contextSha256 === contextSha256 &&
    (input.maybeRequestId === null || /^[a-f0-9]{32}$/u.test(input.maybeRequestId)), "bootstrap_operator_request");
  return input;
}
export function response(contextSha256, requestId, status, code = null) {
  return { schema: "usb-bootstrap-measure-operator-response-v1", contextSha256, requestId, status, code };
}
/** Exclusive claims survive client loss; absent results always remain pending. */
export function createRequestStore(directory, contextSha256, indices, operations = {}) {
  const accepted = new Map();
  async function get(id) {
    check(/^[a-f0-9]{32}$/u.test(id), "bootstrap_operator_request");
    const claim = (await proof(directory, `request-${id}.json`)).value;
    requestShape(claim, contextSha256, indices);
    try { return responseShape((await proof(directory, `result-${id}.json`)).value, contextSha256, id); }
    catch (error) { if (error.code !== "ENOENT") throw error; return response(contextSha256, id, "pending"); }
  }
  async function accept(input) {
    requestShape(input, contextSha256, indices);
    const key = input.requestId, bytes = canonical(input);
    let first = false;
    try { await writeNew(resolve(directory, `request-${key}.json`), JSON.parse(bytes)); first = true; }
    catch (error) {
      if (error.code !== "EEXIST") throw error;
      check(canonical((await proof(directory, `request-${key}.json`)).value) === bytes, "bootstrap_operator_conflict");
    }
    if (first) {
      const phase = input.action === "install" ? `install-${input.payload.index}` : input.action;
      try { await writeNew(resolve(directory, `phase-${phase}.json`), { requestId: key, requestSha256: sha256(bytes) }); }
      catch (error) {
        if (error.code !== "EEXIST") throw error;
        const prior = (await proof(directory, `phase-${phase}.json`)).value;
        check(prior.requestId === key && prior.requestSha256 === sha256(bytes), "bootstrap_operator_conflict");
      }
    }
    accepted.set(key, sha256(bytes));
    return { first, value: await get(key) };
  }
  async function finish(input, code = null) {
    check(accepted.has(input.requestId), "bootstrap_operator_request");
    const value = response(contextSha256, input.requestId, code ? "failed" : "succeeded", code);
    const pending = resolve(directory, `result-${input.requestId}.pending.json`);
    const target = resolve(directory, `result-${input.requestId}.json`);
    await writeNew(pending, value);
    await operations.beforeResultPublish?.();
    // Publish only complete protected bytes; link preserves exclusive final creation.
    await link(pending, target); await unlink(pending); return value;
  }
  return { get, accept, finish };
}
export function responseShape(value, contextSha256, requestId) {
  object(value, ["schema", "contextSha256", "requestId", "status", "code"]);
  check(value.schema === "usb-bootstrap-measure-operator-response-v1" && value.contextSha256 === contextSha256 && value.requestId === requestId &&
    ["pending", "succeeded", "failed"].includes(value.status) && (value.code === null || OPERATOR_CODES.includes(value.code)) &&
    (value.status === "failed" ? value.code !== null : value.code === null), "bootstrap_operator_protocol");
  return value;
}
export function statusShape(value, contextSha256) {
  object(value, ["schema", "contextSha256", "phase", "maybeRequestId", "maybeAction", "maybeIndex", "maybeCode", "maybeSupervisorOrigin", "lastSequence", "lastStateSha256"]);
  check(value.schema === "usb-bootstrap-measure-operator-status-v1" && value.contextSha256 === contextSha256 &&
    ["initializing", "ready", "busy", "failed", "stopping", "stopped"].includes(value.phase) &&
    (value.maybeRequestId === null || /^[a-f0-9]{32}$/u.test(value.maybeRequestId)) &&
    (value.maybeAction === null || ["install", "finish-cleanup", "stop"].includes(value.maybeAction)) &&
    (value.maybeIndex === null || Number.isInteger(value.maybeIndex) && value.maybeIndex >= 0 && value.maybeIndex === 0) &&
    (value.maybeCode === null || OPERATOR_CODES.includes(value.maybeCode)) &&
    (value.maybeSupervisorOrigin === null || /^http:\/\/127\.0\.0\.1:[1-9][0-9]{0,4}$/u.test(value.maybeSupervisorOrigin) && Number(value.maybeSupervisorOrigin.split(":").at(-1)) <= 65535) &&
    (value.lastSequence === null || Number.isSafeInteger(value.lastSequence) && value.lastSequence >= 0) &&
    (value.lastStateSha256 === null || /^[a-f0-9]{64}$/u.test(value.lastStateSha256)), "bootstrap_operator_protocol");
  return value;
}
