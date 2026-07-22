#!/usr/bin/env node
import { createHash } from "node:crypto";
import { chmodSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";

function parseArgs(argv) {
  const args = { readyPath: "", closedPath: "" };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];
    if (arg === "--ready" && next) {
      args.readyPath = next;
      index += 1;
    } else if (arg === "--closed" && next) {
      args.closedPath = next;
      index += 1;
    } else {
      throw new Error(`unsupported argument: ${arg}`);
    }
  }

  if (!args.readyPath || !args.closedPath) {
    throw new Error("--ready and --closed are required");
  }
  return args;
}

function writePrivate(path, value) {
  writeFileSync(path, value, { encoding: "utf8", mode: 0o600, flag: "wx" });
  chmodSync(path, 0o600);
}

function textFrame(value) {
  const payload = Buffer.from(value, "utf8");
  if (payload.length > 125) {
    throw new Error("test payload exceeds one-byte frame limit");
  }
  return Buffer.concat([Buffer.from([0x81, payload.length]), payload]);
}

function acceptValue(key) {
  return createHash("sha1")
    .update(`${key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`)
    .digest("base64");
}

function maybeCloseOpcode(buffer) {
  return buffer.length >= 2 && (buffer[0] & 0x0f) === 0x08;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  let completed = false;
  const server = createServer((socket) => {
    let upgraded = false;
    let input = Buffer.alloc(0);

    socket.on("data", (chunk) => {
      input = Buffer.concat([input, chunk]);
      if (!upgraded) {
        const headerEnd = input.indexOf("\r\n\r\n");
        if (headerEnd < 0) {
          return;
        }
        const headers = input.subarray(0, headerEnd).toString("utf8");
        const keyLine = headers
          .split("\r\n")
          .find((line) => line.toLowerCase().startsWith("sec-websocket-key:"));
        if (!keyLine) {
          socket.destroy();
          return;
        }
        const key = keyLine.slice(keyLine.indexOf(":") + 1).trim();
        socket.write(
          "HTTP/1.1 101 Switching Protocols\r\n" +
            "Upgrade: websocket\r\n" +
            "Connection: Upgrade\r\n" +
            `Sec-WebSocket-Accept: ${acceptValue(key)}\r\n\r\n`,
        );
        socket.write(
          textFrame(
            JSON.stringify({
              event: "system_info",
              data: { bootSession: "synthetic", operatorSnapshotRevision: 2 },
            }),
          ),
        );
        upgraded = true;
        input = input.subarray(headerEnd + 4);
      }

      if (!completed && maybeCloseOpcode(input)) {
        completed = true;
        setTimeout(() => {
          writePrivate(args.closedPath, "peer_observed_close\n");
          socket.end(Buffer.from([0x88, 0x02, 0x03, 0xe8]));
          server.close();
        }, 200);
      }
    });
  });

  const failTimer = setTimeout(() => {
    server.close();
    process.exitCode = 1;
  }, 10_000);

  server.listen(0, "127.0.0.1", () => {
    const address = server.address();
    if (!address || typeof address === "string") {
      throw new Error("loopback server did not expose a TCP port");
    }
    writePrivate(args.readyPath, `${address.port}\n`);
  });

  server.on("close", () => {
    clearTimeout(failTimer);
  });
}

main().catch(() => {
  process.exitCode = 1;
});
