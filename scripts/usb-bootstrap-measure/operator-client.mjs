import { spawn } from "node:child_process";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
import { mkdir, open, realpath } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { processSnapshot, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { check, object, sha256 } from "./values.mjs";
import { exchange } from "../str005-v2-serial/operator-ipc.mjs";
import { checkedOwner } from "../str005-v2-serial/host-resources.mjs";
import { responseShape, statusShape } from "./operator-state.mjs";
async function locate(root, allowStopped = false) {
  await privateRoot(root); await privateRoot(`${root}.operator`);
  const locator = (await proof(`${root}.operator`, "locator.json")).value;
  object(locator, ["schema", "contextSha256", "owner", "socketPath"]);
  const stored = (await proof(root, "context.json")).value; object(stored, ["context", "sha256"]);
  check(stored.sha256 === sha256(JSON.stringify(stored.context)) && stored.context.schema === "usb-bootstrap-measure-context-v1" &&
    root === resolve(stored.context.firmwareRoot, "scratch/usb-bootstrap-measure/attempt-001") &&
    locator.schema === "usb-bootstrap-measure-operator-locator-v1" && locator.contextSha256 === stored.sha256, "bootstrap_operator_owner");
  checkedOwner(locator.owner);
  const current = (await processSnapshot()).some(row => sameProcess(row, locator.owner) && !/[ZT]/u.test(row.state));
  if (!current && allowStopped) {
    const { readStoppedOperator } = await import("./operator-disposition.mjs");
    const stopped = await readStoppedOperator(root, stored.context);
    return { locator, stopped: stopped.stopped.value };
  }
  check(current && locator.owner.pid === locator.owner.pgid, "bootstrap_operator_owner");
  return { locator };
}
export async function operatorStart(options, operations = {}) {
  const root = await privateRoot(resolve(options.privateRoot));
  const context = await (await import("./context.mjs")).loadOperatorContext(root, operations);
  check(options.authorityDirectory === undefined, "bootstrap_operator_request");
  const directory = `${root}.operator`; await mkdir(directory, { mode: 0o700 });
  await writeNew(resolve(directory, "start.claim.json"), { schema: "usb-bootstrap-measure-operator-start-v1", contextSha256: sha256(JSON.stringify(context)) });
  const output = await open(resolve(directory, "daemon.stdout.log"), "wx", 0o600);
  const errors = await open(resolve(directory, "daemon.stderr.log"), "wx", 0o600);
  const child = spawn(await realpath(process.execPath), [fileURLToPath(new URL("./operator-daemon.mjs", import.meta.url)), root],
    { detached: true, stdio: ["ignore", output.fd, errors.fd, "ipc"], env: { PATH: process.env.PATH, HOME: process.env.HOME, LANG: "C", LC_ALL: "C", ...(process.env.TZ ? { TZ: process.env.TZ } : {}), ...nodeRuntimeEnvironment() } });
  await output.close(); await errors.close();
  // The short launcher never owns supervisor exit evidence; the detached daemon does.
  const acknowledgment = new Promise((done, reject) => {
    const timer = setTimeout(() => reject(Object.assign(Error("bootstrap_operator_initialization"), { code: "bootstrap_operator_initialization" })), 9000);
    child.once("message", value => { clearTimeout(timer); done(value); });
    child.once("error", error => { clearTimeout(timer); reject(error); });
    child.once("exit", () => { clearTimeout(timer); reject(Object.assign(Error("bootstrap_operator_initialization"), { code: "bootstrap_operator_initialization" })); });
  });
  child.send({ schema: "usb-bootstrap-measure-operator-bootstrap-v1" });
  try { return statusShape(await acknowledgment, sha256(JSON.stringify(context))); }
  finally { if (child.connected) child.disconnect(); child.unref(); }
}
export async function operatorRequest(options) {
  const root = resolve(options.privateRoot), { locator } = await locate(root);
  const requestPath = resolve(options.requestFile);
  const request = (await proof(await privateRoot(resolve(requestPath, "..")), requestPath.split("/").at(-1))).value;
  check(request.contextSha256 === locator.contextSha256, "bootstrap_operator_request");
  return responseShape(await exchange(locator.socketPath, request), locator.contextSha256, request.requestId);
}
export async function operatorStatus(options) {
  const root = resolve(options.privateRoot), { locator, stopped } = await locate(root, true);
  const maybeRequestId = options.requestId ?? null;
  check(maybeRequestId === null || /^[a-f0-9]{32}$/u.test(maybeRequestId), "bootstrap_operator_request");
  if (stopped) {
    check(stopped.contextSha256 === locator.contextSha256 && stopped.phase === "stopped", "bootstrap_operator_owner");
    return maybeRequestId === null ? statusShape(stopped, locator.contextSha256) : responseShape((await proof(`${root}.operator`, `result-${maybeRequestId}.json`)).value, locator.contextSha256, maybeRequestId);
  }
  const result = await exchange(locator.socketPath, { schema: "usb-bootstrap-measure-operator-query-v1", contextSha256: locator.contextSha256, maybeRequestId });
  return maybeRequestId === null ? statusShape(result, locator.contextSha256) : responseShape(result, locator.contextSha256, maybeRequestId);
}
