import assert from "node:assert/strict";
import { access, mkdtemp, readFile, rm } from "node:fs/promises";
import net from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import test from "node:test";

import { sendInterruptedFirmwareUpload } from "./interrupted-upload.js";

test("real child receives the strict prefix and forced reset without EOF", async () => {
  // Arrange
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-interrupted-upload-"));
  const prefixPath = path.join(root, "prefix-received");
  const observationPath = path.join(root, "observation.json");
  const script = `
    const net = require("node:net");
    const fs = require("node:fs");
    const output = process.argv[1];
    const prefix = process.argv[2];
    const server = net.createServer({ allowHalfOpen: true }, (socket) => {
      const chunks = [];
      let endObserved = false;
      let resetObserved = false;
      socket.on("data", (chunk) => {
        chunks.push(chunk);
        const received = Buffer.concat(chunks);
        const split = received.indexOf(Buffer.from("\\r\\n\\r\\n"));
        if (split >= 0 && received.length - split - 4 === 4096) {
          fs.writeFileSync(prefix, "observed");
        }
      });
      socket.on("end", () => { endObserved = true; });
      socket.on("error", (error) => { resetObserved = error.code === "ECONNRESET"; });
      socket.on("close", () => {
        const received = Buffer.concat(chunks);
        const split = received.indexOf(Buffer.from("\\r\\n\\r\\n"));
        const head = received.subarray(0, split).toString("ascii");
        const body = received.subarray(split + 4);
        const match = /Content-Length: (\\d+)/i.exec(head);
        fs.writeFileSync(output, JSON.stringify({
          declared: Number(match[1]),
          body: body.length,
          end_observed: endObserved,
          reset_observed: resetObserved,
        }));
        server.close();
      });
    });
    server.listen(0, "127.0.0.1", () => process.stdout.write(String(server.address().port) + "\\n"));
  `;
  const child = spawn(process.execPath, ["-e", script, observationPath, prefixPath], {
    stdio: ["ignore", "pipe", "pipe"],
  });
  const childExit = new Promise<number | null>((resolve) => child.once("close", resolve));
  const port = await firstLine(child.stdout);

  try {
    // Act
    let uploadSettled = false;
    const upload = sendInterruptedFirmwareUpload(
      new URL(`http://127.0.0.1:${port}`),
      Buffer.alloc(16_384, 0x5a),
      4_096,
    ).finally(() => {
      uploadSettled = true;
    });
    await waitForFile(prefixPath);

    // Assert
    assert.equal(uploadSettled, false);
    const result = await upload;
    const exitCode = await childExit;
    const observed = JSON.parse(await readFile(observationPath, "utf8")) as {
      declared: number;
      body: number;
      end_observed: boolean;
      reset_observed: boolean;
    };

    assert.equal(exitCode, 0);
    assert.deepEqual(result, {
      declared_body_bytes: 16_384,
      transmitted_body_bytes: 4_096,
      connection_closed: true,
    });
    assert.deepEqual(observed, {
      declared: 16_384,
      body: 4_096,
      end_observed: false,
      reset_observed: true,
    });
  } finally {
    child.kill("SIGKILL");
    await rm(root, { recursive: true, force: true });
  }
});

test("timeout resets and closes an owned interrupted upload socket", async () => {
  // Arrange
  let connectionClosed = false;
  let maybeSocket: net.Socket | undefined;
  const server = net.createServer({ allowHalfOpen: true }, (socket) => {
    maybeSocket = socket;
    socket.on("error", () => {});
    socket.on("close", () => {
      connectionClosed = true;
    });
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (address === null || typeof address === "string") throw new Error("test server address is invalid");

  try {
    // Act
    await assert.rejects(
      sendInterruptedFirmwareUpload(
        new URL(`http://127.0.0.1:${address.port}`),
        Buffer.alloc(8_192),
        2_048,
        25,
      ),
      /interrupted upload timed out/,
    );
    await waitUntil(() => connectionClosed);

    // Assert
    assert.equal(connectionClosed, true);
  } finally {
    maybeSocket?.destroy();
    await closeServer(server);
  }
});

test("connection failure before prefix flush rejects", async () => {
  // Arrange
  const server = net.createServer();
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (address === null || typeof address === "string") throw new Error("test server address is invalid");
  await closeServer(server);

  // Act and Assert
  await assert.rejects(
    sendInterruptedFirmwareUpload(
      new URL(`http://127.0.0.1:${address.port}`),
      Buffer.alloc(4_096),
      1_024,
    ),
  );
});

test("non-origin HTTP target rejects before opening a socket", async () => {
  // Arrange
  const target = new URL("http://127.0.0.1/private");

  // Act and Assert
  await assert.rejects(
    sendInterruptedFirmwareUpload(target, Buffer.alloc(4_096), 1_024),
    /requires one origin-only HTTP target/,
  );
});

test("non-strict image prefix rejects before opening a socket", async () => {
  // Arrange
  const target = new URL("http://127.0.0.1");
  const image = Buffer.alloc(4_096);

  // Act and Assert
  await assert.rejects(
    sendInterruptedFirmwareUpload(target, image, image.length),
    /prefix must be a strict image prefix/,
  );
});

async function firstLine(stream: NodeJS.ReadableStream): Promise<number> {
  return await new Promise<number>((resolve, reject) => {
    let text = "";
    stream.on("data", (chunk: Buffer) => {
      text += chunk.toString("utf8");
      const newline = text.indexOf("\n");
      if (newline < 0) return;
      const port = Number(text.slice(0, newline));
      if (!Number.isSafeInteger(port)) reject(new Error("child server port is invalid"));
      else resolve(port);
    });
    stream.once("error", reject);
  });
}

async function closeServer(server: net.Server): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    server.close((error) => error === undefined ? resolve() : reject(error));
  });
}

async function waitForFile(filePath: string): Promise<void> {
  await waitUntil(async () => {
    try {
      await access(filePath);
      return true;
    } catch {
      return false;
    }
  });
}

async function waitUntil(predicate: () => boolean | Promise<boolean>): Promise<void> {
  const deadline = Date.now() + 2_000;
  while (Date.now() < deadline) {
    if (await predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  throw new Error("test condition timed out");
}
