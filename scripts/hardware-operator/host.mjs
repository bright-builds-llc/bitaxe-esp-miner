// Imperative host helpers shared by the operator commands: private paths, process identity and bounded waits.
import { execFileSync } from "node:child_process";
import { appendFile, lstat, mkdir, realpath } from "node:fs/promises";
import { dirname, isAbsolute, resolve } from "node:path";
import { processSnapshot, sameProcess } from "../host-stalls/capture.mjs";
import { refuse, HardwareOperatorError } from "./errors.mjs";

export const pause = (milliseconds) => new Promise((done) => setTimeout(done, milliseconds));

/**
 * The real Node binary. Under aspect_rules_js `process.execPath` is a wrapper script that needs the
 * JS_BINARY__* environment, which detached children deliberately do not inherit.
 */
export function nodeBinary(environment = process.env) {
  return resolve(environment.JS_BINARY__NODE_BINARY ?? process.execPath);
}

/**
 * The caller's own environment for nested `just` and detached children: without this binary's
 * JS_BINARY__* launcher state and without its `_node_bin` wrapper directory on PATH. Nested launchers
 * would otherwise inherit fs-patch roots, and a changed PATH can invalidate Bazel's cached actions.
 */
export function hostEnvironment(environment = process.env) {
  const maybeWrapperDirectory = environment.JS_BINARY__NODE_WRAPPER ? dirname(resolve(environment.JS_BINARY__NODE_WRAPPER)) : null;
  const result = Object.fromEntries(Object.entries(environment).filter(([key]) => !key.startsWith("JS_BINARY__")));
  if (typeof result.PATH === "string") {
    result.PATH = result.PATH.split(":").filter((entry) => entry !== "" && resolve(entry) !== maybeWrapperDirectory && !/_node_bin$/u.test(entry)).join(":");
  }
  return result;
}

/** Append one step row to a private breadcrumb log; rows carry codes and counts, never device values. */
export async function breadcrumb(parent, action, row) {
  await appendFile(resolve(parent, "hardware-operator.jsonl"), `${JSON.stringify({ atUnixMs: Date.now(), action, ...row })}\n`, { mode: 0o600 });
}

/** Resolve a user path the way `bazel run` callers expect: relative to the directory they ran `just` from. */
export function userPath(path, environment = process.env) {
  refuse(typeof path === "string" && path.length > 0, "path_missing");
  return resolve(environment.BUILD_WORKING_DIRECTORY ?? process.cwd(), path);
}

export function workspace(environment = process.env) {
  const maybeWorkspace = environment.BUILD_WORKSPACE_DIRECTORY;
  refuse(typeof maybeWorkspace === "string" && isAbsolute(maybeWorkspace), "workspace_unknown");
  return maybeWorkspace;
}

/** Require an existing, non-aliased, owner-only (0700) directory. */
export async function privateDirectory(path, missingCode = "private_directory_missing") {
  let stat;
  try { stat = await lstat(path); } catch (error) {
    if (error.code === "ENOENT") throw new HardwareOperatorError(missingCode);
    throw error;
  }
  refuse(stat.isDirectory() && !stat.isSymbolicLink(), "private_directory_type");
  refuse((stat.mode & 0o777) === 0o700, "private_directory_mode");
  refuse(await realpath(path) === resolve(path), "private_directory_alias");
  return resolve(path);
}

/** Create a new owner-only directory; an existing one is refused with `existsCode`. */
export async function newPrivateDirectory(path, existsCode) {
  try { await mkdir(path, { mode: 0o700 }); } catch (error) {
    if (error.code === "EEXIST") throw new HardwareOperatorError(existsCode);
    throw error;
  }
  return privateDirectory(path);
}

export async function absent(path, code) {
  try { await lstat(path); } catch (error) {
    if (error.code === "ENOENT") return;
    throw error;
  }
  throw new HardwareOperatorError(code);
}

/** Identity of a live, non-zombie process; zombies have exited and only await reaping. */
export async function liveIdentity(pid, operations = {}) {
  const rows = await (operations.processSnapshot ?? processSnapshot)();
  const row = rows.find((candidate) => candidate.pid === pid && !String(candidate.state ?? "").startsWith("Z"));
  return row ? { pid: row.pid, pgid: row.pgid, startedAt: row.startedAt } : null;
}

export async function isLive(identity, operations = {}) {
  const maybeLive = await liveIdentity(identity.pid, operations);
  return maybeLive !== null && sameProcess(maybeLive, identity);
}

/** Poll `probe` until it returns a non-null value or the automated bound elapses (null). */
export async function waitFor(probe, { timeoutMs, intervalMs = 250 }) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const value = await probe();
    if (value !== null && value !== undefined) return value;
    if (Date.now() >= deadline) return null;
    await pause(intervalMs);
  }
}

/** SIGTERM one recorded process and wait for it to exit; refuses a pid now owned by another process. */
export async function terminate(identity, { timeoutMs, failureCode }, operations = {}) {
  if (!(await isLive(identity, operations))) return "already_stopped";
  process.kill(identity.pid, "SIGTERM");
  const stopped = await waitFor(async () => (await isLive(identity, operations)) ? null : true, { timeoutMs });
  refuse(stopped === true, failureCode);
  return "stopped";
}

/** Prove no process listens on a TCP port; lsof exits 1 with no output when nothing matches. */
export function requirePortFree(port, operations = {}) {
  let output;
  try {
    output = (operations.execFileSync ?? execFileSync)("/usr/sbin/lsof", ["-nP", `-iTCP:${port}`, "-sTCP:LISTEN", "-t"],
      { encoding: "utf8", timeout: 5000, stdio: ["ignore", "pipe", "pipe"] });
  } catch (error) {
    refuse(error.status === 1 && String(error.stdout ?? "").trim() === "", "port_unproved");
    return;
  }
  throw new HardwareOperatorError(String(output).trim() === "" ? "port_unproved" : "port_held");
}
