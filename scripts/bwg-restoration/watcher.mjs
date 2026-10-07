// Owns one `flash usb-presence-watch` process for a physical checkpoint: verified binary, physical identity as
// a joined flag, ProtectedOperational JSON lines copied to a mode-0600 journal, and a bounded stop by stdin EOF.
import { spawn as spawnProcess } from "node:child_process";
import { appendFile, lstat, open } from "node:fs/promises";
import { createInterface } from "node:readline";
import { fileDigest, requireCondition } from "../fixed-usb-qualification/contract.mjs";

const STOP_GRACE_MS = 5000;
const EVENTS = new Set(["started", "present", "absent", "reappeared", "enumeration_changed", "stable", "stopped", "failed"]);
const DIGEST = /^[0-9a-f]{64}$/u;

/** One watcher line, closed: no device node, only the enumeration digest and counts. */
export function parseWatcherLine(text) {
  const value = JSON.parse(text);
  const allowed = new Set(["schema", "sequence", "elapsed_ms", "event", "enumeration_sha256", "enumeration_changed", "holder_count", "ready", "category"]);
  requireCondition(value && typeof value === "object" && Object.keys(value).every((key) => allowed.has(key)) &&
    value.schema === "bwg-usb-presence-watch-v1" && Number.isSafeInteger(value.sequence) && value.sequence > 0 &&
    Number.isSafeInteger(value.elapsed_ms) && value.elapsed_ms >= 0 && EVENTS.has(value.event) &&
    (value.enumeration_sha256 === undefined || DIGEST.test(value.enumeration_sha256)) &&
    (value.enumeration_changed === undefined || typeof value.enumeration_changed === "boolean") &&
    (value.ready === undefined || typeof value.ready === "boolean") &&
    (value.category === undefined || value.category === "probe_failed"), "watcher_line");
  return value;
}

export function createPresenceWatcher({ binary, physicalIdentity, journalPath, stderrPath, onEvent }, operations = {}) {
  const now = operations.now ?? Date.now;
  let child, exit, maybeStopping, expected = 1, lastEvent;
  let writes = Promise.resolve();

  function deliver(event) {
    writes = writes.then(() => appendFile(journalPath, `${JSON.stringify({ receivedAtUnixMs: now(), ...event })}\n`, { mode: 0o600 }));
    lastEvent = event;
    onEvent(event, now());
  }

  async function start() {
    requireCondition(child === undefined, "watcher_already_started");
    requireCondition(DIGEST.test(physicalIdentity), "watcher_identity");
    const info = await lstat(binary.path);
    requireCondition(info.isFile() && !info.isSymbolicLink() && (info.mode & 0o111) !== 0 && await fileDigest(binary.path) === binary.sha256, "watcher_binary");
    const stderr = await open(stderrPath, "a", 0o600);
    try {
      child = (operations.spawn ?? spawnProcess)(binary.path, ["usb-presence-watch", `--physical-identity=${physicalIdentity}`, "--interval-ms=250",
        "--stable-ms=3000"], { stdio: ["pipe", "pipe", stderr.fd], env: {} });
    } finally { await stderr.close(); }
    child.stdin.on("error", () => undefined);
    child.once("close", (code, signal) => {
      exit = { code, signal };
      if (!maybeStopping) deliver({ schema: "bwg-usb-presence-watch-v1", sequence: expected, elapsed_ms: 0, event: "failed", category: "probe_failed" });
    });
    child.once("error", () => { exit ??= { code: null, signal: null }; });
    createInterface({ input: child.stdout }).on("line", (line) => {
      let event;
      try { event = parseWatcherLine(line); } catch { child.kill("SIGKILL"); return; }
      if (event.sequence !== expected) { child.kill("SIGKILL"); return; }
      expected += 1;
      deliver(event);
    });
    return { watcher_started: true };
  }

  async function stopImpl() {
    if (!child) return null;
    if (!exit) {
      child.stdin.end();
      const deadline = now() + STOP_GRACE_MS;
      while (!exit && now() < deadline) await new Promise((done) => setTimeout(done, 50));
      if (!exit) { child.kill("SIGKILL"); while (!exit) await new Promise((done) => setTimeout(done, 25)); }
    }
    await writes;
    return { exit_code: exit.code, signal: exit.signal, stopped_on_request: exit.code === 0 && lastEvent?.event === "stopped" };
  }
  const stop = () => (maybeStopping ??= stopImpl());
  return { start, stop, running: () => child !== undefined && exit === undefined };
}
