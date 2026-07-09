#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";

const FIXTURE_NAME = "phase28-source-work-v1";
const VALID_SESSION_LABELS = new Set(["upstream", "rust"]);
const FIXTURE = Object.freeze({
  subscribeResult: {
    extranonce1: "4de05269",
    extranonce2Len: 4,
  },
  difficulty: 42,
  notify: {
    jobId: "phase28-source-work-v1",
    prevBlockHash: "00".repeat(32),
    coinbase1: "0200000001",
    coinbase2: "ffffffff",
    merkleBranches: [],
    version: "20000004",
    nbits: "1705ae3a",
    ntime: "647025b5",
    cleanJobs: false,
  },
  versionRollingMask: "1fffe000",
});

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.1-fake-stratum-pool.mjs --host <host> --port <port> --fixture phase28-source-work-v1 --session-label upstream|rust --ready-json <path> --report-json <path> --duration-seconds <n>

Runs a deterministic Stratum v1 fake pool for Phase 28.1.1.1 source-work alignment.
Reports contain redacted event labels and fixture fingerprints only.
`);
  process.exit(exitCode);
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      usage(0);
    }

    if (!arg.startsWith("--")) {
      usage(1);
    }

    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      usage(1);
    }

    args.set(arg.slice(2), value);
    index += 1;
  }

  for (const key of [
    "host",
    "port",
    "fixture",
    "session-label",
    "ready-json",
    "report-json",
    "duration-seconds",
  ]) {
    if (!args.get(key)) {
      usage(1);
    }
  }

  const fixture = args.get("fixture");
  if (fixture !== FIXTURE_NAME) {
    fail(`unsupported fixture: ${fixture}`);
  }

  const sessionLabel = args.get("session-label");
  if (!VALID_SESSION_LABELS.has(sessionLabel)) {
    fail(`unsupported session label: ${sessionLabel}`);
  }

  const port = Number(args.get("port"));
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    fail("--port must be an integer from 0 to 65535");
  }

  const durationSeconds = Number(args.get("duration-seconds"));
  if (!Number.isInteger(durationSeconds) || durationSeconds < 1) {
    fail("--duration-seconds must be a positive integer");
  }

  return {
    host: args.get("host"),
    port,
    fixture,
    sessionLabel,
    readyJson: args.get("ready-json"),
    reportJson: args.get("report-json"),
    durationSeconds,
  };
}

function sourceWorkFingerprint() {
  return crypto.createHash("sha256").update(JSON.stringify(FIXTURE)).digest("hex");
}

function writeJsonFile(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const tempPath = `${filePath}.tmp-${process.pid}`;
  fs.writeFileSync(tempPath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
  fs.renameSync(tempPath, filePath);
}

function fail(message) {
  console.error(`phase28_fake_stratum_pool_error: ${message}`);
  process.exit(1);
}

const options = parseArgs(process.argv.slice(2));
const fingerprint = sourceWorkFingerprint();
const sockets = new Set();
const events = [];
const methodCounts = new Map();
let notifySentCount = 0;
let submitObservedCount = 0;
let acceptedSubmitCount = 0;
let finalized = false;

function recordEvent(event) {
  events.push(event);
  if (events.length > 200) {
    events.shift();
  }
}

function recordMethod(method) {
  methodCounts.set(method, (methodCounts.get(method) ?? 0) + 1);
  recordEvent({ event: "client_method_observed", method });
}

function sendJson(socket, value, event) {
  socket.write(`${JSON.stringify(value)}\n`);
  recordEvent(event);
}

function response(id, result) {
  return {
    id,
    result,
    error: null,
  };
}

function handleMessage(socket, message) {
  if (!message || typeof message !== "object" || Array.isArray(message)) {
    sendJson(socket, { id: null, result: false, error: [20, "invalid request", null] }, {
      event: "invalid_request_rejected",
    });
    return;
  }

  const method = typeof message.method === "string" ? message.method : "";
  const maybeId = Number.isInteger(message.id) ? message.id : null;
  recordMethod(method || "unknown");

  switch (method) {
    case "mining.configure":
      sendJson(
        socket,
        response(maybeId, {
          "version-rolling": true,
          "version-rolling.mask": FIXTURE.versionRollingMask,
        }),
        { event: "configure_response_sent" },
      );
      break;
    case "mining.subscribe":
      sendJson(
        socket,
        response(maybeId, [
          [
            ["mining.set_difficulty", "1"],
            ["mining.notify", "1"],
          ],
          FIXTURE.subscribeResult.extranonce1,
          FIXTURE.subscribeResult.extranonce2Len,
        ]),
        { event: "subscribe_response_sent" },
      );
      break;
    case "mining.authorize":
      sendJson(socket, response(maybeId, true), { event: "authorize_response_sent" });
      setTimeout(() => sendSourceWork(socket), 100);
      break;
    case "mining.suggest_difficulty":
    case "mining.extranonce.subscribe":
      sendJson(socket, response(maybeId, true), { event: "setup_response_sent", method });
      break;
    case "mining.submit":
      submitObservedCount += 1;
      acceptedSubmitCount += 1;
      sendJson(socket, response(maybeId, true), { event: "submit_response_sent" });
      break;
    case "pong":
      recordEvent({ event: "pong_observed" });
      break;
    default:
      sendJson(socket, { id: maybeId, result: false, error: [20, "unsupported method", null] }, {
        event: "unsupported_method_rejected",
        method: method || "unknown",
      });
      break;
  }
}

function sendSourceWork(socket) {
  if (socket.destroyed || socket.phase28SourceWorkSent) {
    return;
  }

  socket.phase28SourceWorkSent = true;
  sendJson(
    socket,
    {
      id: null,
      method: "mining.set_difficulty",
      params: [FIXTURE.difficulty],
    },
    { event: "set_difficulty_sent" },
  );
  sendJson(
    socket,
    {
      id: null,
      method: "mining.notify",
      params: [
        FIXTURE.notify.jobId,
        FIXTURE.notify.prevBlockHash,
        FIXTURE.notify.coinbase1,
        FIXTURE.notify.coinbase2,
        FIXTURE.notify.merkleBranches,
        FIXTURE.notify.version,
        FIXTURE.notify.nbits,
        FIXTURE.notify.ntime,
        FIXTURE.notify.cleanJobs,
      ],
    },
    { event: "notify_sent", source_work_fingerprint: "sha256" },
  );
  notifySentCount += 1;
}

function report(status) {
  const methodCountsObject = Object.fromEntries([...methodCounts.entries()].sort());
  return {
    status,
    fixture: options.fixture,
    session_label: options.sessionLabel,
    source_work_fingerprint: fingerprint,
    source_work_fingerprint_kind: "sha256",
    connection_count: sockets.size,
    method_counts: methodCountsObject,
    configure_observed: (methodCounts.get("mining.configure") ?? 0) > 0,
    subscribe_observed: (methodCounts.get("mining.subscribe") ?? 0) > 0,
    authorize_observed: (methodCounts.get("mining.authorize") ?? 0) > 0,
    submit_observed: submitObservedCount > 0,
    notify_sent_count: notifySentCount,
    accepted_submit_count: acceptedSubmitCount,
    raw_messages_committed: false,
    credential_contents_read: false,
    events,
  };
}

const server = net.createServer((socket) => {
  sockets.add(socket);
  socket.setEncoding("utf8");
  socket.phase28SourceWorkSent = false;
  recordEvent({ event: "client_connected" });

  let buffer = "";
  socket.on("data", (chunk) => {
    buffer += chunk;
    let newlineIndex = buffer.indexOf("\n");
    while (newlineIndex !== -1) {
      const line = buffer.slice(0, newlineIndex).trim();
      buffer = buffer.slice(newlineIndex + 1);
      if (line) {
        try {
          handleMessage(socket, JSON.parse(line));
        } catch {
          sendJson(socket, { id: null, result: false, error: [20, "invalid json", null] }, {
            event: "invalid_json_rejected",
          });
        }
      }
      newlineIndex = buffer.indexOf("\n");
    }
  });
  socket.on("close", () => recordEvent({ event: "client_disconnected" }));
  socket.on("error", () => recordEvent({ event: "client_socket_error" }));
});

server.on("error", (error) => {
  fail(error.message);
});

function finish(status) {
  if (finalized) {
    return;
  }
  finalized = true;
  for (const socket of sockets) {
    socket.destroy();
  }
  server.close();
  writeJsonFile(options.reportJson, report(status));
}

process.on("SIGINT", () => {
  finish("stopped");
  process.exit(0);
});
process.on("SIGTERM", () => {
  finish("stopped");
  process.exit(0);
});
process.on("exit", () => {
  if (!finalized) {
    finish("process_exit");
  }
});

server.listen(options.port, options.host, () => {
  const address = server.address();
  writeJsonFile(options.readyJson, {
    status: "ready",
    fixture: options.fixture,
    session_label: options.sessionLabel,
    bound_host: options.host,
    bound_port: address.port,
    source_work_fingerprint: fingerprint,
    source_work_fingerprint_kind: "sha256",
    raw_messages_committed: false,
    credential_contents_read: false,
  });
});

setTimeout(() => {
  finish("duration_elapsed");
  process.exit(0);
}, options.durationSeconds * 1000);
