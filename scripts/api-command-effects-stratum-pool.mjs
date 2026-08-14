#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";

const FIXTURE_NAME = "api-command-effects-v1";
const FIXTURE = Object.freeze({
  difficulty: 1,
  extranonce1: "4de05269",
  extranonce2Length: 4,
  versionRollingMask: "1fffe000",
  notify: {
    jobId: FIXTURE_NAME,
    previousBlockHash: "00".repeat(32),
    coinbase1: "0200000001",
    coinbase2: "ffffffff",
    merkleBranches: [],
    version: "20000004",
    networkTarget: "207fffff",
    timestamp: "647025b5",
    cleanJobs: true,
  },
});

function fail(message) {
  process.stderr.write(`api_command_effects_fixture_error: ${message}\n`);
  process.exit(1);
}

function parseArgs(argv) {
  const values = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (!flag?.startsWith("--") || !value || value.startsWith("--")) {
      fail("arguments must be flag/value pairs");
    }
    values.set(flag.slice(2), value);
  }
  for (const required of [
    "host", "port", "fixture", "session-label", "ready-json", "report-json",
    "duration-seconds", "stop-file",
  ]) {
    if (!values.has(required)) fail(`missing --${required}`);
  }
  if (values.get("fixture") !== FIXTURE_NAME || values.get("session-label") !== "command-effects") {
    fail("closed fixture identity is invalid");
  }
  const port = Number(values.get("port"));
  const durationSeconds = Number(values.get("duration-seconds"));
  if (!Number.isInteger(port) || port < 0 || port > 65535) fail("port is invalid");
  if (!Number.isInteger(durationSeconds) || durationSeconds < 1 || durationSeconds > 7_800) {
    fail("duration is invalid");
  }
  return {
    host: values.get("host"),
    port,
    readyJson: values.get("ready-json"),
    reportJson: values.get("report-json"),
    stopFile: values.get("stop-file"),
    durationSeconds,
  };
}

function writePrivateJson(output, value) {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const temporary = `${output}.tmp-${process.pid}`;
  fs.writeFileSync(temporary, `${JSON.stringify(value, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
    mode: 0o600,
  });
  fs.renameSync(temporary, output);
  fs.chmodSync(output, 0o600);
}

const options = parseArgs(process.argv.slice(2));
const fingerprint = crypto.createHash("sha256").update(JSON.stringify(FIXTURE)).digest("hex");
const sockets = new Set();
const methodCounts = new Map();
let connectionCount = 0;
let notifySentCount = 0;
let acceptedSubmitCount = 0;
let finalized = false;

function methodObserved(method) {
  methodCounts.set(method, (methodCounts.get(method) ?? 0) + 1);
}

function send(socket, value) {
  socket.write(`${JSON.stringify(value)}\n`);
}

function response(id, result) {
  return { id, result, error: null };
}

function sendWork(socket) {
  if (socket.destroyed || socket.commandEffectsWorkSent) return;
  socket.commandEffectsWorkSent = true;
  send(socket, { id: null, method: "mining.set_difficulty", params: [FIXTURE.difficulty] });
  const work = FIXTURE.notify;
  send(socket, {
    id: null,
    method: "mining.notify",
    params: [
      work.jobId,
      work.previousBlockHash,
      work.coinbase1,
      work.coinbase2,
      work.merkleBranches,
      work.version,
      work.networkTarget,
      work.timestamp,
      work.cleanJobs,
    ],
  });
  notifySentCount += 1;
}

function handleMessage(socket, message) {
  if (!message || typeof message !== "object" || Array.isArray(message)) {
    send(socket, { id: null, result: false, error: [20, "invalid request", null] });
    return;
  }
  const method = typeof message.method === "string" ? message.method : "unknown";
  const id = Number.isInteger(message.id) ? message.id : null;
  methodObserved(method);
  switch (method) {
    case "mining.configure":
      send(socket, response(id, {
        "version-rolling": true,
        "version-rolling.mask": FIXTURE.versionRollingMask,
      }));
      break;
    case "mining.subscribe":
      send(socket, response(id, [
        [["mining.set_difficulty", "1"], ["mining.notify", "1"]],
        FIXTURE.extranonce1,
        FIXTURE.extranonce2Length,
      ]));
      break;
    case "mining.authorize":
      send(socket, response(id, true));
      setTimeout(() => sendWork(socket), 100);
      break;
    case "mining.suggest_difficulty":
    case "mining.extranonce.subscribe":
      send(socket, response(id, true));
      break;
    case "mining.submit":
      acceptedSubmitCount += 1;
      send(socket, response(id, true));
      break;
    case "pong":
      break;
    default:
      send(socket, { id, result: false, error: [20, "unsupported method", null] });
  }
}

const server = net.createServer((socket) => {
  sockets.add(socket);
  connectionCount += 1;
  socket.commandEffectsWorkSent = false;
  socket.setEncoding("utf8");
  let buffer = "";
  socket.on("data", (chunk) => {
    buffer += chunk;
    let newline = buffer.indexOf("\n");
    while (newline !== -1) {
      const line = buffer.slice(0, newline).trim();
      buffer = buffer.slice(newline + 1);
      if (line) {
        try {
          handleMessage(socket, JSON.parse(line));
        } catch {
          send(socket, { id: null, result: false, error: [20, "invalid json", null] });
        }
      }
      newline = buffer.indexOf("\n");
    }
  });
  socket.on("close", () => sockets.delete(socket));
  socket.on("error", () => sockets.delete(socket));
});

function report(status) {
  return {
    status,
    fixture: FIXTURE_NAME,
    source_work_fingerprint: fingerprint,
    source_work_fingerprint_kind: "sha256",
    connection_count: connectionCount,
    method_counts: Object.fromEntries([...methodCounts.entries()].sort()),
    configure_observed: (methodCounts.get("mining.configure") ?? 0) > 0,
    subscribe_observed: (methodCounts.get("mining.subscribe") ?? 0) > 0,
    authorize_observed: (methodCounts.get("mining.authorize") ?? 0) > 0,
    submit_observed: (methodCounts.get("mining.submit") ?? 0) > 0,
    notify_sent_count: notifySentCount,
    accepted_submit_count: acceptedSubmitCount,
    compact_network_target: FIXTURE.notify.networkTarget,
    raw_messages_committed: false,
    credential_contents_read: false,
  };
}

function finish(status) {
  if (finalized) return;
  finalized = true;
  for (const socket of sockets) socket.destroy();
  server.close();
  writePrivateJson(options.reportJson, report(status));
}

server.on("error", (error) => fail(error.message));
server.listen(options.port, options.host, () => {
  const address = server.address();
  writePrivateJson(options.readyJson, {
    status: "ready",
    fixture: FIXTURE_NAME,
    bound_port: address.port,
    source_work_fingerprint: fingerprint,
    raw_messages_committed: false,
    credential_contents_read: false,
  });
});

const stopWatcher = setInterval(() => {
  if (!fs.existsSync(options.stopFile)) return;
  clearInterval(stopWatcher);
  finish("stopped");
  process.exit(0);
}, 100);

setTimeout(() => {
  finish("duration_elapsed");
  process.exit(0);
}, options.durationSeconds * 1_000);

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => {
    finish("stopped");
    process.exit(0);
  });
}
