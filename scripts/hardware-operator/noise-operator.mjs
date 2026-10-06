// Launch, drive and release the detached Noise-serial operator parent through a held-open command FIFO.
import { execFileSync, spawn } from "node:child_process";
import { constants } from "node:fs";
import { appendFile, open, readFile, realpath, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import { writeNew } from "../fixed-usb-qualification/contract.mjs";
import { HardwareOperatorError, refuse } from "./errors.mjs";
import { absent, hostEnvironment, isLive, liveIdentity, nodeBinary, newPrivateDirectory, privateDirectory, terminate, waitFor } from "./host.mjs";
import { operatorDirectory, parseCommand, readinessOf, replyAfter, completeLines } from "./noise-protocol.mjs";

const LAUNCH_SCHEMA = "hardware-operator-noise-launch-v1";
const READY_TIMEOUT_MS = 60_000;
export const DEFAULT_REPLY_TIMEOUT_MS = 900_000;
const HOLDER_STOP_MS = 5000;
// The holder only keeps the FIFO's write end open, so writes between commands never deliver EOF.
const HOLDER_SECONDS = "2147483647";

const files = (operator) => ({
  fifo: resolve(operator, "commands.fifo"), launch: resolve(operator, "launch.json"), stdout: resolve(operator, "parent.stdout.log"),
  stderr: resolve(operator, "parent.stderr.log"), lock: resolve(operator, "send.lock"), commands: resolve(operator, "commands.jsonl"),
  holderStopped: resolve(operator, "holder-stopped.json"),
});

async function identityOf(pid, operations) {
  const maybeIdentity = await waitFor(() => liveIdentity(pid, operations), { timeoutMs: 3000, intervalMs: 100 });
  refuse(maybeIdentity !== null && maybeIdentity.pgid === pid, "noise_operator_process_identity");
  return maybeIdentity;
}

async function spawnParent(node, program, root, paths) {
  const stdout = await open(paths.stdout, "wx", 0o600), stderr = await open(paths.stderr, "wx", 0o600);
  try {
    // `exec` keeps the recorded pid; the shell only performs the blocking FIFO open for stdin.
    const child = spawn("/bin/sh", ["-c", 'exec "$0" "$1" "$2" < "$3"', node, program, `--private-root=${root}`, paths.fifo], {
      detached: true, stdio: ["ignore", stdout.fd, stderr.fd],
      env: { PATH: hostEnvironment().PATH ?? "/usr/bin:/bin", HOME: process.env.HOME ?? "/", LANG: "C", LC_ALL: "C",
        ...(process.env.TZ ? { TZ: process.env.TZ } : {}) },
    });
    child.unref();
    return child;
  } finally { await stdout.close(); await stderr.close(); }
}

function spawnHolder(fifo) {
  const child = spawn("/bin/sh", ["-c", 'exec /bin/sleep "$0" > "$1"', HOLDER_SECONDS, fifo], { detached: true, stdio: "ignore" });
  child.unref();
  return child;
}

async function readLaunch(root) {
  const operator = await privateDirectory(operatorDirectory(root), "noise_operator_not_launched");
  const paths = files(operator);
  let launch;
  try { launch = JSON.parse(await readFile(paths.launch, "utf8")); } catch (error) {
    if (error.code === "ENOENT") throw new HardwareOperatorError("noise_operator_launch_record_missing");
    throw new HardwareOperatorError("noise_operator_launch_record_invalid");
  }
  refuse(launch?.schema === LAUNCH_SCHEMA && launch.root === root, "noise_operator_launch_record_invalid");
  return { paths, launch };
}

/**
 * Start one detached parent for an attempt root, then wait for its first line.
 * A second launch for the same root is refused because the operator directory already exists.
 */
export async function launch(root, operations = {}) {
  process.umask(0o077);
  root = await privateDirectory(root, "noise_attempt_root_missing");
  const operator = await newPrivateDirectory(operatorDirectory(root), "noise_operator_already_launched");
  const paths = files(operator);
  (operations.execFileSync ?? execFileSync)("/usr/bin/mkfifo", ["-m", "600", paths.fifo], { stdio: "ignore", timeout: 5000 });
  const node = await realpath(nodeBinary());
  const parent = await spawnParent(node, operations.parentProgram, root, paths);
  const holder = spawnHolder(paths.fifo);
  const record = { schema: LAUNCH_SCHEMA, root, parent: await identityOf(parent.pid, operations),
    holder: await identityOf(holder.pid, operations), launchedAtUnixMs: Date.now() };
  await writeNew(paths.launch, record);
  const maybeReadiness = await waitFor(async () => {
    const maybeReadiness = readinessOf(await readFile(paths.stdout, "utf8"));
    if (maybeReadiness) return maybeReadiness;
    return (await isLive(record.parent, operations)) ? null : { ready: false, reply: null };
  }, { timeoutMs: operations.readyTimeoutMs ?? READY_TIMEOUT_MS });
  if (maybeReadiness?.ready) return { event: "noise_operator_ready", reply: maybeReadiness.reply };
  // A parent that failed or exited before readiness never reads commands, so its holder is stopped. After a
  // timeout the parent may still become ready, and stopping the holder would hand it an EOF.
  if (maybeReadiness !== null) await terminate(record.holder, { timeoutMs: HOLDER_STOP_MS, failureCode: "noise_operator_holder_stop_failed" }, operations);
  const error = new HardwareOperatorError(maybeReadiness === null ? "noise_operator_ready_timeout"
    : maybeReadiness.reply === null ? "noise_operator_parent_exited" : "noise_operator_not_ready");
  error.reply = maybeReadiness?.reply ?? null;
  throw error;
}

async function acquireLock(path, operations) {
  const owner = await liveIdentity(process.pid, operations);
  refuse(owner !== null, "noise_operator_process_identity");
  for (let attempt = 0; attempt < 2; attempt++) {
    try { await writeNew(path, owner); return; } catch (error) { if (error.code !== "EEXIST") throw error; }
    let holder;
    try { holder = JSON.parse(await readFile(path, "utf8")); } catch { throw new HardwareOperatorError("noise_operator_send_busy"); }
    refuse(!(holder?.startedAt && (await isLive(holder, operations))), "noise_operator_send_busy");
    await unlink(path);
  }
  throw new HardwareOperatorError("noise_operator_send_busy");
}

async function writeCommand(fifo, line) {
  let handle;
  try { handle = await open(fifo, constants.O_WRONLY | constants.O_NONBLOCK); } catch (error) {
    if (error.code === "ENXIO") throw new HardwareOperatorError("noise_operator_parent_not_reading");
    throw error;
  }
  try { await handle.write(`${line}\n`); } finally { await handle.close(); }
}

async function ledger(paths) {
  let text;
  try { text = await readFile(paths.commands, "utf8"); } catch (error) { if (error.code === "ENOENT") return []; throw error; }
  return completeLines(text).map((line) => JSON.parse(line));
}

/**
 * The parent answers commands strictly in order with one line each after its readiness line, so the
 * reply to the n-th sent command is stdout line n. Waiting ends early only when the parent exits.
 */
async function awaitReply(paths, record, sequence, operations) {
  let checks = 0;
  const maybeReply = await waitFor(async () => {
    const maybeLine = replyAfter(await readFile(paths.stdout, "utf8"), sequence);
    if (maybeLine) return maybeLine;
    if (++checks % 8 === 0 && !(await isLive(record.parent, operations))) {
      return replyAfter(await readFile(paths.stdout, "utf8"), sequence) ?? { missing: true };
    }
    return null;
  }, { timeoutMs: operations.replyTimeoutMs ?? DEFAULT_REPLY_TIMEOUT_MS });
  refuse(maybeReply !== null, "noise_operator_reply_timeout");
  refuse(maybeReply.missing !== true, "noise_operator_parent_exited");
  await appendFile(paths.commands, `${JSON.stringify({ kind: "reply", sequence, replyEvent: maybeReply.event, repliedAtUnixMs: Date.now() })}\n`, { mode: 0o600 });
  return maybeReply;
}

async function admittedOperator(root, operations) {
  process.umask(0o077);
  root = await privateDirectory(root, "noise_attempt_root_missing");
  const { paths, launch: record } = await readLaunch(root);
  await acquireLock(paths.lock, operations);
  return { paths, record };
}

/**
 * Send one validated command and return the parent's reply to it. While an earlier command's reply is
 * still outstanding (its sender timed out or was killed) a new send is refused; use `awaitPending`.
 */
export async function send(root, line, operations = {}) {
  const command = parseCommand(line);
  const { paths, record } = await admittedOperator(root, operations);
  try {
    refuse(await isLive(record.parent, operations), "noise_operator_parent_not_running");
    const sent = (await ledger(paths)).filter((row) => row.kind === "sent").length;
    refuse(completeLines(await readFile(paths.stdout, "utf8")).length - 1 >= sent, "noise_operator_reply_pending");
    const sequence = sent + 1;
    // Recorded before the write, so a killed sender still leaves the outstanding command visible.
    await appendFile(paths.commands, `${JSON.stringify({ kind: "sent", sequence, command, sentAtUnixMs: Date.now() })}\n`, { mode: 0o600 });
    await writeCommand(paths.fifo, JSON.stringify(command));
    return await awaitReply(paths, record, sequence, operations);
  } finally { await unlink(paths.lock); }
}

/** Wait for the reply to the most recently sent command, after its own sender timed out or was killed. */
export async function awaitPending(root, operations = {}) {
  const { paths, record } = await admittedOperator(root, operations);
  try {
    const rows = await ledger(paths), sent = rows.filter((row) => row.kind === "sent").length;
    refuse(sent > 0 && !rows.some((row) => row.kind === "reply" && row.sequence === sent), "noise_operator_no_pending_reply");
    return await awaitReply(paths, record, sent, operations);
  } finally { await unlink(paths.lock); }
}

/** Stop the FIFO holder once the parent has exited; refuses while the parent still reads commands. */
export async function stopHolder(root, operations = {}) {
  process.umask(0o077);
  root = await privateDirectory(root, "noise_attempt_root_missing");
  const { paths, launch: record } = await readLaunch(root);
  refuse(!(await isLive(record.parent, operations)), "noise_operator_parent_live");
  await absent(paths.holderStopped, "noise_operator_holder_stop_recorded");
  const outcome = await terminate(record.holder, { timeoutMs: HOLDER_STOP_MS, failureCode: "noise_operator_holder_stop_failed" }, operations);
  await writeNew(paths.holderStopped, { holder: record.holder, outcome, stoppedAtUnixMs: Date.now() });
  return { event: outcome === "stopped" ? "holder_stopped" : "holder_already_stopped" };
}
