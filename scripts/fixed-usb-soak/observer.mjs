// Owns one soak observer process: private endpoint handoff on stdin, journal straight to a 0600 file,
// bounded stop. The journal itself carries no endpoint, pool or credential values.
import { spawn } from "node:child_process";
import { lstat, open, readFile } from "node:fs/promises";
import { isIP } from "node:net";
import { resolve } from "node:path";
import { exactObject, fileDigest, QualificationError, requireCondition, writeNew } from "../fixed-usb-qualification/contract.mjs";
import { IDLE_MINIMUM_HTTP_SAMPLES, IDLE_PROOF_MS } from "./contract.mjs";

const STOP_GRACE_MS = 10000;
const PRIVATE_FIRST_OCTETS = (octets) => octets[0] === 10 || (octets[0] === 172 && octets[1] >= 16 && octets[1] <= 31) || (octets[0] === 192 && octets[1] === 168);
const BROKEN_EVENTS = new Set(["closed", "failed", "connect_failed", "projection_invalid", "http_failed", "error"]);

/** Complete journal lines only; the observer may be mid-write on the last one. */
export function journalLines(text) {
  const parts = text.split("\n");
  parts.pop();
  return parts.filter(Boolean).map((line) => JSON.parse(line));
}

/**
 * Idle pre-phase proof over the journal written since the observer started: at least 60 s, at least
 * 20 HTTP samples and 10 WebSocket samples, one WebSocket connection and no reconnect or failure.
 */
export function idleProof(lines, startedAtUnixMs, nowUnixMs) {
  const samples = (transport) => lines.filter((line) => line.event === "sample" && line.transport === transport);
  const http = samples("http"), websocket = samples("websocket");
  const connections = lines.filter((line) => line.event === "connected");
  const broken = lines.filter((line) => BROKEN_EVENTS.has(line.event)).length;
  const mining = [...http, ...websocket].some((line) => line.sample?.miningActive);
  const passed = nowUnixMs - startedAtUnixMs >= IDLE_PROOF_MS && http.length >= IDLE_MINIMUM_HTTP_SAMPLES && websocket.length >= 10 &&
    connections.length === 1 && connections[0].detail?.reconnect === false && broken === 0 && !mining;
  return { schema: "soak-idle-proof-v1", passed, elapsed_ms: nowUnixMs - startedAtUnixMs, http_samples: http.length,
    websocket_samples: websocket.length, websocket_connections: connections.length, broken_events: broken, mining_observed: mining };
}

export function createSoakObserver(root, context, operations = {}) {
  const now = operations.now ?? Date.now;
  let child, startedAtUnixMs, exit, maybeStopping, used = false;
  const journalPath = resolve(root, "soak-observer.jsonl");

  async function start(endpoint) {
    requireCondition(!used, "observer_already_used"); used = true;
    exactObject(endpoint, ["ipv4", "httpPort"]);
    requireCondition(isIP(endpoint.ipv4) === 4 && PRIVATE_FIRST_OCTETS(endpoint.ipv4.split(".").map(Number)) &&
      Number.isInteger(endpoint.httpPort) && endpoint.httpPort > 0 && endpoint.httpPort <= 65535, "observer_endpoint");
    const binary = context.soak_observer, info = await lstat(binary.path);
    requireCondition(info.isFile() && !info.isSymbolicLink() && (info.mode & 0o111) !== 0 && await fileDigest(binary.path) === binary.sha256, "observer_binary");
    const stdout = await open(journalPath, "wx", 0o600), stderr = await open(resolve(root, "soak-observer.stderr.log"), "wx", 0o600);
    try {
      startedAtUnixMs = now();
      child = (operations.spawn ?? spawn)(binary.path, [], { stdio: ["pipe", stdout.fd, stderr.fd], env: {} });
    } finally { await stdout.close(); await stderr.close(); }
    child.once("close", (code, signal) => { exit = { code, signal, closedAtUnixMs: now() }; });
    child.once("error", () => { exit ??= { code: null, signal: null, closedAtUnixMs: now(), spawnFailed: true }; });
    const handoff = Buffer.from(JSON.stringify({ schema: "soak-observer-input-v1", ipv4: endpoint.ipv4, port: endpoint.httpPort,
      observedUnixMs: startedAtUnixMs, expiresUnixMs: startedAtUnixMs + 5000, bindingVerified: true }) + "\n");
    child.stdin.on("error", () => undefined);
    child.stdin.write(handoff, () => handoff.fill(0));
    return { observer_started: true };
  }

  async function lines() {
    try { return journalLines(await readFile(journalPath, "utf8")); } catch (error) { if (error.code === "ENOENT") return []; throw error; }
  }

  async function idle() {
    requireCondition(child && !exit, "observer_not_running");
    return idleProof(await lines(), startedAtUnixMs, now());
  }

  async function stopImpl() {
    if (!child) return null;
    if (!exit) {
      child.stdin.end(JSON.stringify({ op: "stop" }) + "\n");
      const deadline = now() + STOP_GRACE_MS;
      while (!exit && now() < deadline) await new Promise((done) => setTimeout(done, 100));
      if (!exit) { child.kill("SIGKILL"); while (!exit) await new Promise((done) => setTimeout(done, 50)); }
    }
    const last = (await lines()).at(-1);
    const result = { schema: "soak-observer-result-v1", started_at_unix_ms: startedAtUnixMs, exit_code: exit.code, signal: exit.signal,
      closed_at_unix_ms: exit.closedAtUnixMs, stopped_on_request: last?.event === "stopped" && last.detail?.reason === "requested" && exit.code === 0 };
    await writeNew(resolve(root, "soak-observer-result.json"), result);
    if (!result.stopped_on_request) throw new QualificationError("observer_exit");
    return result;
  }
  const stop = () => (maybeStopping ??= stopImpl());
  return { start, idle, stop, started: () => startedAtUnixMs, journalPath };
}
