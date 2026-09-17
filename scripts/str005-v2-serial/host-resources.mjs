import { isIP } from "node:net";
import { spawnSync } from "node:child_process";
import { readdir } from "node:fs/promises";
import { proof } from "../str005-noise-serial/files.mjs";
import { processSnapshot, requireGone, requireLsofAbsent, requireNoHolders, sameProcess, serialNodes } from "../str005-noise-serial/host-resources.mjs";
import { check, object, port, sha256, uint } from "./values.mjs";
export { processSnapshot, requireGone, requireLsofAbsent, requireNoHolders, sameProcess };
export const LISTENER_ARGS = Object.freeze(["-nP", "-iTCP", "-sTCP:LISTEN", "-Fpn"]);

/** Runtime-only parsed values: never print, hash or persist the listener inventory. */
export function parseListenerInventory(output) {
  check(typeof output === "string" && Buffer.byteLength(output) <= 1048576, "v2_listener_inventory_bound");
  const rows = output.split("\n").filter(Boolean); check(rows.length > 0 && rows.length <= 16384, "v2_listener_inventory_shape");
  let maybePid = null, maybeFormat = null, needsName = false, groupHasName = false;
  const listeners = [], descriptors = new Set();
  const completeGroup = () => check(maybePid === null || (groupHasName && !needsName), "v2_listener_inventory_shape");
  for (const row of rows) {
    if (/^p[1-9][0-9]*$/u.test(row)) {
      completeGroup(); maybePid = uint(Number(row.slice(1)), 0x7fffffff); groupHasName = false;
      continue;
    }
    if (/^f(?:0|[1-9][0-9]*)$/u.test(row)) {
      check(maybePid !== null && maybeFormat !== "names" && !needsName, "v2_listener_inventory_shape");
      const descriptor = uint(Number(row.slice(1)), 0x7fffffff), key = `${maybePid}:${descriptor}`;
      check(!descriptors.has(key), "v2_listener_inventory_shape");
      descriptors.add(key); maybeFormat = "descriptors"; needsName = true;
      continue;
    }
    const match = /^n(\*|(?:[0-9]{1,3}\.){3}[0-9]{1,3}|\[[0-9A-Za-z:.%_-]+\]):([1-9][0-9]*)$/u.exec(row);
    check(maybePid !== null && match !== null && (maybeFormat !== "descriptors" || needsName), "v2_listener_inventory_shape");
    const address = match[1].replace(/^\[|\]$/gu, "").split("%")[0];
    check(address === "*" || isIP(address) !== 0, "v2_listener_inventory_shape");
    listeners.push({ pid: maybePid, port: port(Number(match[2])) });
    // Some supported inventories contain only PID/name fields. Once descriptor
    // records appear, every file must have its own name; never mix the grammars.
    maybeFormat ??= "names"; needsName = false; groupHasName = true;
  }
  completeGroup(); check(listeners.length > 0, "v2_listener_inventory_shape");
  return listeners;
}
/** The private port never enters argv, environment, diagnostics or the return value. */
export function requirePoolListenerAbsent(privatePort, operations = {}) {
  port(privatePort);
  let result;
  try {
    result = (operations.spawnSync ?? spawnSync)("/usr/sbin/lsof", [...LISTENER_ARGS], {
      encoding: "utf8", timeout: 5000, maxBuffer: 1048576, stdio: ["ignore", "pipe", "pipe"],
      env: { PATH: "/usr/bin:/bin", LANG: "C", LC_ALL: "C" },
    });
  } catch { check(false, "v2_pool_listener_unproved"); }
  check(result && !result.error && result.signal == null && String(result.stderr ?? "") === "", "v2_pool_listener_unproved");
  const output = String(result.stdout ?? "");
  if (result.status === 1 && output === "") return;
  check(result.status === 0, "v2_pool_listener_unproved");
  const listeners = parseListenerInventory(output);
  check(!listeners.some((entry) => entry.port === privatePort), "v2_pool_listener_present");
}
export function checkedOwner(value) {
  check(value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).every((key) =>
    ["pid", "pgid", "startedAt", "ppid", "state", "cpuPercent"].includes(key)), "v2_owner_shape");
  check(uint(value.pid, 0x7fffffff) > 0 && uint(value.pgid, 0x7fffffff) > 0 && typeof value.startedAt === "string" &&
    value.startedAt.length > 0 && value.startedAt.length <= 128, "v2_owner_identity");
  if (value.ppid !== undefined) uint(value.ppid, 0x7fffffff);
  if (value.state !== undefined) check(typeof value.state === "string" && /^[A-Za-z+<NsEsLWX-]{1,12}$/u.test(value.state), "v2_owner_state");
  if (value.cpuPercent !== undefined) check(Number.isFinite(value.cpuPercent) && value.cpuPercent >= 0, "v2_owner_cpu");
  return { pid: value.pid, pgid: value.pgid, startedAt: value.startedAt };
}
export async function installationResources(root, context) {
  const owners = [], paths = new Set(); const names = await readdir(root);
  for (const name of names.filter((value) => /^install-[0-4]\.claim\.json$/u.test(value)).sort()) {
    const claim = (await proof(root, name)).value;
    check(claim.contextSha256 === sha256(JSON.stringify(context)) && context.install_indices.includes(claim.index), "v2_cleanup_install_context");
    serialNodes(claim.detector.port); paths.add(claim.detector.port);
  }
  for (const name of names.filter((value) => /^install-[0-4](?:\.detect)?\.host-root\.json$/u.test(value)).sort()) owners.push(checkedOwner((await proof(root, name)).value));
  return { owners, serialPorts: [...paths].sort() };
}
export async function requireAllSerialHoldersAbsent(root, context, operations = {}) {
  const resources = await installationResources(root, context);
  for (const path of resources.serialPorts) requireNoHolders(path, operations);
  return resources;
}
/** Closed signer observations are source-produced process close events, not grant contents. */
export async function signerExitProofs(root, context) {
  const names = (await readdir(root)).filter((name) => /^signer-[0-9]{2}\.exit\.json$/u.test(name)).sort();
  check(names.length <= 11, "v2_signer_exit_bound"); const digests = [];
  for (const [index, name] of names.entries()) {
    const receipt = await proof(root, name), value = receipt.value;
    object(value, ["schema", "contextSha256", "index", "operation", "observation"]);
    check(value.schema === "str005-v2-signer-exit-v1" && value.contextSha256 === sha256(JSON.stringify(context)) &&
      value.index === index + 1 && name === `signer-${String(index + 1).padStart(2, "0")}.exit.json` &&
      ["public-trust", "start", "renew"].includes(value.operation), "v2_signer_exit_binding");
    const o = value.observation;
    object(o, ["pid", "code", "signal", "elapsedMs", "stdoutBytes", "stderrBytes", "overflow", "inputFailed"]);
    check((o.pid === null || uint(o.pid, 0x7fffffff) > 0) && (o.code === null || Number.isInteger(o.code)) &&
      (o.signal === null || (typeof o.signal === "string" && /^SIG[A-Z0-9]+$/u.test(o.signal))) &&
      (o.code !== null || o.signal !== null), "v2_signer_close_unproved");
    uint(o.elapsedMs); uint(o.stdoutBytes); uint(o.stderrBytes);
    check(typeof o.overflow === "boolean" && typeof o.inputFailed === "boolean", "v2_signer_exit_shape");
    check(o.overflow || (o.stdoutBytes <= 65536 && o.stderrBytes <= 65536), "v2_signer_exit_overflow");
    digests.push({ path: name, sha256: receipt.sha256 });
  }
  return digests;
}
